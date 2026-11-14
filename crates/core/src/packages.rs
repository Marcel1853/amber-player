//! Plugin-Pakete: ein oder mehrere Klangeffekte und Visualisierungen in einer Datei, mit
//! Beschreibung, eigener Lizenz und optionaler Anleitung. Die Datei heißt `<id>.amberplugin`
//! (innen eine ZIP-Datei; `.zip` geht auch).
//!
//! ```text
//! nachtfahrt/                 ← liegt in packages/
//! ├── plugin.json             ← { "id", "name", "version", "author", "license", "contents": [ … ] }
//! ├── effects/telefon.json
//! ├── effects/stadion.json
//! ├── visualizers/stern.js
//! ├── LICENSE                 ← Lizenz dieses Pakets
//! └── README.md               ← optional
//! ```
//!
//! Ein Paket mit nur einem Teil darf statt `contents` auch `"type"` und `"main"` angeben (so
//! sahen die ersten Pakete aus; sie liegen noch in `effects/<id>/` bzw. `visualizers/<id>/`).
//! Einzelne `.json`- bzw. `.js`-Dateien direkt im Plugin-Ordner gehen weiterhin.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, ensure, Context, Result};
use serde::{Deserialize, Serialize};

use crate::effects::custom::CustomPlugin;
use crate::library::Library;

/// Ordner für Pakete im Datenordner.
pub const PACKAGES_DIR: &str = "packages";
/// Einstellung „Erweiterte Plugins erlauben“ (Online-Quellen mit Internetzugriff).
pub const EXTENDED_SETTING: &str = "extended_plugins";
const MAX_CONTENTS: usize = 32;
const MAX_FILE: u64 = 512 * 1024;
const MAX_TOTAL: usize = 2 * 1024 * 1024;

/// Art eines Teils.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Effect,
    Visualizer,
    /// Online-Quelle (JavaScript, „erweitertes Plugin“ mit Internetzugriff auf freigegebene Adressen)
    Source,
}

impl Kind {
    /// Ordner der alten Ein-Teil-Pakete und einzelnen Dateien.
    pub fn dir_name(self) -> &'static str {
        match self {
            Kind::Effect => "effects",
            Kind::Visualizer => "visualizers",
            Kind::Source => "sources",
        }
    }

    fn default_main(self) -> &'static str {
        match self {
            Kind::Effect => "effect.json",
            Kind::Visualizer => "visualizer.js",
            Kind::Source => "source.js",
        }
    }

    fn ext(self) -> &'static str {
        match self {
            Kind::Effect => ".json",
            Kind::Visualizer | Kind::Source => ".js",
        }
    }
}

/// Ein Teil eines Pakets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Content {
    #[serde(rename = "type")]
    pub kind: Kind,
    /// Pfad im Paket, z. B. `effects/telefon.json`.
    pub file: String,
}

/// Rechte eines Pakets.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Permissions {
    /// Adressen, die Online-Quellen des Pakets erreichen dürfen, z. B. `"example.org"` oder
    /// `"*.example.org"` (auch alle Unteradressen). Nur https.
    pub network: Vec<String>,
}

/// Inhalt von `plugin.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub description: String,
    /// Kurzname der Lizenz, z. B. „MIT“ oder „Alle Rechte vorbehalten“.
    #[serde(default)]
    pub license: String,
    #[serde(default)]
    pub homepage: String,
    /// Die Teile des Pakets.
    #[serde(default)]
    pub contents: Vec<Content>,
    /// Rechte (nur für Online-Quellen nötig).
    #[serde(default)]
    pub permissions: Permissions,
    /// Übersetzungen der Texte im Paket: Sprachcode → { Originaltext → Übersetzung }.
    /// Gilt für Paketname und -beschreibung sowie Namen, Beschreibungen, Regler und Presets
    /// der Klangeffekte und Visualisierungen.
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub translations: std::collections::BTreeMap<String, std::collections::BTreeMap<String, String>>,
    /// Kurzform für Pakete mit einem Teil: Art …
    #[serde(rename = "type", default, skip_serializing)]
    pub kind: Option<Kind>,
    /// … und Datei (Standard `effect.json` bzw. `visualizer.js`).
    #[serde(default, skip_serializing)]
    pub main: String,
}

impl Manifest {
    pub fn parse(text: &str) -> Result<Self> {
        let mut m: Manifest = serde_json::from_str(text).context("plugin.json ist kein gültiges JSON")?;
        ensure!(valid_id(&m.id), "„id“ in plugin.json: 1–32 Zeichen aus Buchstaben, Ziffern, - und _");
        ensure!(!m.name.trim().is_empty(), "„name“ in plugin.json fehlt");
        if m.contents.is_empty() {
            let kind = m.kind.ok_or_else(|| anyhow!("plugin.json braucht „contents“ (Liste der Teile) oder „type“"))?;
            let file = if m.main.is_empty() { kind.default_main().to_string() } else { m.main.clone() };
            m.contents.push(Content { kind, file });
        }
        ensure!(m.contents.len() <= MAX_CONTENTS, "Höchstens {MAX_CONTENTS} Teile pro Paket");
        for c in &m.contents {
            ensure!(safe_path(&c.file), "Ungültiger Dateipfad in „contents“: {}", c.file);
            ensure!(
                c.file.to_ascii_lowercase().ends_with(c.kind.ext()),
                "{} muss bei dieser Art auf {} enden",
                c.file,
                c.kind.ext()
            );
        }
        ensure!(m.translations.len() <= 40, "Höchstens 40 Sprachen in „translations“");
        let mut size = 0;
        for (lang, table) in &m.translations {
            ensure!(
                (2..=5).contains(&lang.len()) && lang.chars().all(|c| c.is_ascii_alphabetic() || c == '-'),
                "Ungültiger Sprachcode in „translations“: {lang} (z. B. en, fr, pt-BR)"
            );
            size += table.iter().map(|(k, v)| k.len() + v.len()).sum::<usize>();
        }
        ensure!(size <= 64 * 1024, "„translations“ ist zu groß (höchstens 64 KB)");
        ensure!(m.permissions.network.len() <= 32, "Höchstens 32 Adressen in „permissions.network“");
        for host in &m.permissions.network {
            ensure!(valid_host_pattern(host), "Ungültige Adresse in „permissions.network“: {host} (z. B. example.org oder *.example.org)");
        }
        if m.count(Kind::Source) > 0 {
            ensure!(!m.permissions.network.is_empty(), "Online-Quellen brauchen „permissions.network“ mit den Adressen, die sie erreichen dürfen");
        }
        m.name = m.name.chars().take(40).collect();
        m.version = m.version.chars().take(20).collect();
        m.author = m.author.chars().take(60).collect();
        m.license = m.license.chars().take(60).collect();
        m.description = m.description.chars().take(240).collect();
        Ok(m)
    }

    pub fn count(&self, kind: Kind) -> usize {
        self.contents.iter().filter(|c| c.kind == kind).count()
    }
}

/// Ein installiertes Paket.
#[derive(Debug, Clone, Serialize)]
pub struct Installed {
    #[serde(flatten)]
    pub manifest: Manifest,
    pub effects: usize,
    pub visualizers: usize,
    pub sources: usize,
    /// Liegt eine Lizenzdatei bei?
    pub license_file: bool,
    pub readme: bool,
    /// Wo das Paket läuft: `pc`, `phone`. Alle Teile eines Pakets (Klangeffekte,
    /// Visualisierungen, Online-Quellen in JavaScript) laufen auf beiden.
    pub platforms: Vec<&'static str>,
    #[serde(skip)]
    pub dir: PathBuf,
}

fn valid_id(id: &str) -> bool {
    (1..=32).contains(&id.len()) && id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// `example.org` oder `*.example.org` – nur Buchstaben, Ziffern, Punkte und Bindestriche.
fn valid_host_pattern(p: &str) -> bool {
    let host = p.strip_prefix("*.").unwrap_or(p);
    host.contains('.')
        && host.len() <= 253
        && host.split('.').all(|l| !l.is_empty() && l.chars().all(|c| c.is_ascii_alphanumeric() || c == '-'))
}

fn safe_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 64
        && !name.starts_with('.')
        && name.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
}

/// Dateiname oder ein Unterordner + Dateiname (`effects/telefon.json`).
fn safe_path(path: &str) -> bool {
    let parts: Vec<&str> = path.split('/').collect();
    (1..=2).contains(&parts.len()) && parts.iter().all(|p| safe_name(p))
}

fn is_license(name: &str) -> bool {
    let n = name.to_ascii_uppercase();
    n.starts_with("LICENSE") || n.starts_with("LICENCE") || n.starts_with("COPYING")
}

fn is_readme(name: &str) -> bool {
    name.to_ascii_uppercase().starts_with("README")
}

/// Liest ein Paket aus einem Ordner (`None`, wenn dort keine `plugin.json` liegt).
pub fn read(dir: &Path) -> Option<Result<Installed>> {
    let text = std::fs::read_to_string(dir.join("plugin.json")).ok()?;
    Some(Manifest::parse(&text).map(|manifest| {
        let names: Vec<String> = std::fs::read_dir(dir)
            .map(|rd| rd.filter_map(Result::ok).map(|e| e.file_name().to_string_lossy().into_owned()).collect())
            .unwrap_or_default();
        Installed {
            effects: manifest.count(Kind::Effect),
            visualizers: manifest.count(Kind::Visualizer),
            sources: manifest.count(Kind::Source),
            license_file: names.iter().any(|n| is_license(n)),
            readme: names.iter().any(|n| is_readme(n)),
            platforms: vec!["pc", "phone"],
            manifest,
            dir: dir.to_path_buf(),
        }
    }))
}

/// Alle Pakete in den angegebenen Ordnern (Unterordner mit `plugin.json`). Gleiche `id` zählt nur
/// einmal (der erste Ordner gewinnt). Fehlerhafte Pakete kommen mit Meldung in die zweite Liste.
pub fn list_in(bases: &[PathBuf]) -> (Vec<Installed>, Vec<String>) {
    let mut ok: Vec<Installed> = Vec::new();
    let mut errors = Vec::new();
    for base in bases {
        let Ok(rd) = std::fs::read_dir(base) else { continue };
        let mut dirs: Vec<PathBuf> = rd
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|p| p.is_dir() && !p.file_name().is_some_and(|n| n.to_string_lossy().starts_with('.')))
            .collect();
        dirs.sort();
        for dir in dirs {
            let folder = dir.file_name().unwrap_or_default().to_string_lossy().into_owned();
            match read(&dir) {
                None => {}
                Some(Ok(p)) if ok.iter().any(|x| x.manifest.id == p.manifest.id) => {}
                Some(Ok(p)) => ok.push(p),
                Some(Err(e)) => errors.push(format!("{folder}/: {e:#}")),
            }
        }
    }
    (ok, errors)
}

/// Wo Pakete liegen: der Paket-Ordner und die Ordner der alten Ein-Teil-Pakete.
pub fn bases_in(root: &Path) -> Vec<PathBuf> {
    vec![root.join(PACKAGES_DIR), root.join(Kind::Effect.dir_name()), root.join(Kind::Visualizer.dir_name())]
}

/// Alle installierten Pakete im Datenordner von Amber Player.
pub fn list() -> (Vec<Installed>, Vec<String>) {
    list_in(&bases_in(&Library::data_dir()))
}

/// Alle Teile einer Art aus den Paketen: (Quelle für Meldungen, Text oder Fehler).
pub fn contents_in(bases: &[PathBuf], kind: Kind) -> (Vec<(String, Result<String>)>, Vec<String>) {
    let (packages, errors) = list_in(bases);
    let mut out = Vec::new();
    for p in packages {
        for c in p.manifest.contents.iter().filter(|c| c.kind == kind) {
            let source = format!("{}/{}", p.manifest.id, c.file);
            let path = p.dir.join(&c.file);
            let text = if std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0) > MAX_FILE {
                Err(anyhow!("größer als 512 KB"))
            } else {
                std::fs::read_to_string(&path).map_err(|_| anyhow!("Datei fehlt oder ist nicht lesbar"))
            };
            out.push((source, text));
        }
    }
    (out, errors)
}

/// Online-Quellen aus den Paketen: (Quelle für Meldungen, Code, erlaubte Adressen).
pub fn source_parts_in(bases: &[PathBuf]) -> (Vec<(String, Result<String>, Vec<String>)>, Vec<String>) {
    let (packages, errors) = list_in(bases);
    let mut out = Vec::new();
    for p in packages {
        for c in p.manifest.contents.iter().filter(|c| c.kind == Kind::Source) {
            let source = format!("{}/{}", p.manifest.id, c.file);
            let text = std::fs::read_to_string(p.dir.join(&c.file)).map_err(|_| anyhow!("Datei fehlt oder ist nicht lesbar"));
            out.push((source, text, p.manifest.permissions.network.clone()));
        }
    }
    (out, errors)
}

pub fn source_parts() -> (Vec<(String, Result<String>, Vec<String>)>, Vec<String>) {
    source_parts_in(&bases_in(&Library::data_dir()))
}

/// Teile einer Art aus dem Datenordner von Amber Player.
pub fn contents(kind: Kind) -> (Vec<(String, Result<String>)>, Vec<String>) {
    contents_in(&bases_in(&Library::data_dir()), kind)
}

/// Paket aus seinen Dateien installieren (Pfad im Paket → Inhalt, alles Text). Ein vorhandenes
/// Paket mit derselben `id` wird ersetzt. Gibt das Manifest zurück.
pub fn install_into(root: &Path, files: &HashMap<String, String>) -> Result<Manifest> {
    let manifest_text = files.get("plugin.json").ok_or_else(|| anyhow!("Im Paket fehlt die Datei plugin.json"))?;
    let manifest = Manifest::parse(manifest_text)?;
    let total: usize = files.values().map(String::len).sum();
    ensure!(total <= MAX_TOTAL, "Das Paket ist zu groß (höchstens 2 MB)");
    for c in &manifest.contents {
        let text = files.get(&c.file).ok_or_else(|| anyhow!("Im Paket fehlt die Datei {}", c.file))?;
        ensure!(text.len() as u64 <= MAX_FILE, "{} ist größer als 512 KB", c.file);
        if c.kind == Kind::Effect {
            CustomPlugin::parse(text, &format!("{}/{}", manifest.id, c.file))
                .with_context(|| format!("Der Klangeffekt {} ist fehlerhaft", c.file))?;
        }
    }
    for name in files.keys() {
        ensure!(safe_path(name), "Ungültiger Dateiname im Paket: {name}");
        let listed = manifest.contents.iter().any(|c| &c.file == name);
        let top = !name.contains('/');
        let allowed = name == "plugin.json" || listed || (top && (is_license(name) || is_readme(name)));
        ensure!(allowed, "Unerwartete Datei im Paket: {name} (erlaubt: plugin.json, die Teile aus „contents“, LICENSE, README)");
    }

    let base = root.join(PACKAGES_DIR);
    std::fs::create_dir_all(&base)?;
    let target = base.join(&manifest.id);
    // Erst vollständig in einen Nachbarordner schreiben, dann austauschen – so bleibt bei einem
    // Fehler das alte Paket heil.
    let tmp = base.join(format!(".{}-neu", manifest.id));
    let _ = std::fs::remove_dir_all(&tmp);
    for (name, text) in files {
        let path = tmp.join(name);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, text)?;
    }
    if target.exists() {
        std::fs::remove_dir_all(&target)?;
    }
    std::fs::rename(&tmp, &target)?;
    // Alte Ein-Teil-Fassung desselben Pakets entfernen, sonst gäbe es den Teil doppelt.
    for kind in [Kind::Effect, Kind::Visualizer] {
        let old = root.join(kind.dir_name()).join(&manifest.id);
        if old.join("plugin.json").exists() {
            let _ = std::fs::remove_dir_all(old);
        }
    }
    Ok(manifest)
}

/// Alle Dateien eines installierten Pakets (zum Weitergeben, z. B. vom PC aufs Handy) –
/// genau die Dateien, die [`install_into`] annimmt.
pub fn files_in(bases: &[PathBuf], id: &str) -> Result<HashMap<String, String>> {
    ensure!(valid_id(id), "Ungültige Plugin-ID");
    let (all, _) = list_in(bases);
    let p = all.into_iter().find(|p| p.manifest.id == id).ok_or_else(|| anyhow!("Plugin-Paket {id} nicht gefunden"))?;
    let mut files = HashMap::new();
    files.insert("plugin.json".to_string(), std::fs::read_to_string(p.dir.join("plugin.json"))?);
    for c in &p.manifest.contents {
        files.insert(c.file.clone(), std::fs::read_to_string(p.dir.join(&c.file)).with_context(|| format!("{} fehlt", c.file))?);
    }
    for e in std::fs::read_dir(&p.dir)?.filter_map(Result::ok) {
        let name = e.file_name().to_string_lossy().into_owned();
        if (is_license(&name) || is_readme(&name)) && e.path().is_file() {
            if let Ok(text) = std::fs::read_to_string(e.path()) {
                files.insert(name, text);
            }
        }
    }
    Ok(files)
}

pub fn files(id: &str) -> Result<HashMap<String, String>> {
    files_in(&bases_in(&Library::data_dir()), id)
}

/// Installiertes Paket wieder als `.amberplugin` (ZIP) – zum Teilen mit anderen.
pub fn export_in(bases: &[PathBuf], id: &str) -> Result<Vec<u8>> {
    Ok(zip_store(&files_in(bases, id)?))
}

pub fn export(id: &str) -> Result<Vec<u8>> {
    export_in(&bases_in(&Library::data_dir()), id)
}

fn crc32(data: &[u8]) -> u32 {
    let mut crc = !0u32;
    for &b in data {
        crc ^= b as u32;
        for _ in 0..8 {
            crc = if crc & 1 != 0 { (crc >> 1) ^ 0xEDB8_8320 } else { crc >> 1 };
        }
    }
    !crc
}

/// Einfaches ZIP ohne Kompression (Pakete sind klein) – jedes Entpackprogramm liest das.
fn zip_store(files: &HashMap<String, String>) -> Vec<u8> {
    let mut names: Vec<&String> = files.keys().collect();
    names.sort();
    let mut out = Vec::new();
    let mut central = Vec::new();
    const DATE: u16 = (1 << 5) | 1; // 1.1.1980
    for name in &names {
        let data = files[*name].as_bytes();
        let crc = crc32(data);
        let offset = out.len() as u32;
        let header = |sig: u32, central_entry: bool| {
            let mut h = Vec::new();
            h.extend_from_slice(&sig.to_le_bytes());
            if central_entry {
                h.extend_from_slice(&20u16.to_le_bytes()); // erstellt mit
            }
            h.extend_from_slice(&20u16.to_le_bytes()); // benötigte Version
            h.extend_from_slice(&0x0800u16.to_le_bytes()); // Namen in UTF-8
            h.extend_from_slice(&0u16.to_le_bytes()); // ohne Kompression
            h.extend_from_slice(&0u16.to_le_bytes()); // Uhrzeit
            h.extend_from_slice(&DATE.to_le_bytes());
            h.extend_from_slice(&crc.to_le_bytes());
            h.extend_from_slice(&(data.len() as u32).to_le_bytes());
            h.extend_from_slice(&(data.len() as u32).to_le_bytes());
            h.extend_from_slice(&(name.len() as u16).to_le_bytes());
            h.extend_from_slice(&0u16.to_le_bytes()); // Extrafeld
            if central_entry {
                h.extend_from_slice(&[0; 8]); // Kommentar, Disk, Attribute
                h.extend_from_slice(&0u32.to_le_bytes()[..2]);
                h.extend_from_slice(&offset.to_le_bytes());
            }
            h.extend_from_slice(name.as_bytes());
            h
        };
        out.extend(header(0x0403_4b50, false));
        out.extend_from_slice(data);
        central.extend(header(0x0201_4b50, true));
    }
    let cd_offset = out.len() as u32;
    let cd_size = central.len() as u32;
    out.extend(central);
    out.extend_from_slice(&0x0605_4b50u32.to_le_bytes());
    out.extend_from_slice(&[0; 4]);
    out.extend_from_slice(&(names.len() as u16).to_le_bytes());
    out.extend_from_slice(&(names.len() as u16).to_le_bytes());
    out.extend_from_slice(&cd_size.to_le_bytes());
    out.extend_from_slice(&cd_offset.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out
}

/// Paket in den Datenordner von Amber Player installieren.
pub fn install(files: &HashMap<String, String>) -> Result<Manifest> {
    install_into(&Library::data_dir(), files)
}

/// Paket entfernen (mit allen Teilen).
pub fn remove_in(root: &Path, id: &str) -> Result<()> {
    ensure!(valid_id(id), "Ungültige Plugin-ID");
    let mut found = false;
    for base in bases_in(root) {
        let dir = base.join(id);
        if dir.join("plugin.json").exists() {
            std::fs::remove_dir_all(&dir)?;
            found = true;
        }
    }
    if !found {
        bail!("Plugin „{id}“ ist nicht installiert");
    }
    Ok(())
}

pub fn remove(id: &str) -> Result<()> {
    remove_in(&Library::data_dir(), id)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn files(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs.iter().map(|(a, b)| (a.to_string(), b.to_string())).collect()
    }

    const EFFECT: &str = r#"{"id":"telefon","name":"Telefon","chain":[{"type":"lowpass","freq":3000}]}"#;
    const EFFECT2: &str = r#"{"id":"stadion","name":"Stadion","chain":[{"type":"echo","time":120}]}"#;

    #[test]
    fn bundle_with_several_parts() {
        let root = tempfile::tempdir().unwrap();
        let manifest = r#"{"id":"nachtfahrt","name":"Nachtfahrt","version":"1.0","license":"MIT","contents":[
            {"type":"effect","file":"effects/telefon.json"},
            {"type":"effect","file":"effects/stadion.json"},
            {"type":"visualizer","file":"visualizers/stern.js"}]}"#;
        let m = install_into(
            root.path(),
            &files(&[
                ("plugin.json", manifest),
                ("effects/telefon.json", EFFECT),
                ("effects/stadion.json", EFFECT2),
                ("visualizers/stern.js", "amber.visualizer({id:'s',name:'S',draw(){}});"),
                ("LICENSE", "MIT …"),
            ]),
        )
        .unwrap();
        assert_eq!(m.contents.len(), 3);
        let (found, errors) = list_in(&bases_in(root.path()));
        assert!(errors.is_empty(), "{errors:?}");
        assert_eq!((found[0].effects, found[0].visualizers), (2, 1));
        assert!(found[0].license_file);
        let (effects, _) = contents_in(&bases_in(root.path()), Kind::Effect);
        assert_eq!(effects.len(), 2);
        assert!(effects.iter().all(|(_, t)| t.is_ok()));
        let (vis, _) = contents_in(&bases_in(root.path()), Kind::Visualizer);
        assert_eq!(vis[0].0, "nachtfahrt/visualizers/stern.js");
        remove_in(root.path(), "nachtfahrt").unwrap();
        assert!(list_in(&bases_in(root.path())).0.is_empty());
    }

    #[test]
    fn single_part_shorthand_and_replace() {
        let root = tempfile::tempdir().unwrap();
        let manifest = r#"{"id":"telefon","name":"Telefon","type":"effect","version":"1.0","license":"MIT"}"#;
        let m = install_into(root.path(), &files(&[("plugin.json", manifest), ("effect.json", EFFECT), ("LICENSE", "x")])).unwrap();
        assert_eq!(m.contents[0].file, "effect.json");
        // nochmal installieren ersetzt
        install_into(root.path(), &files(&[("plugin.json", manifest), ("effect.json", EFFECT)])).unwrap();
        let (found, _) = list_in(&bases_in(root.path()));
        assert_eq!(found.len(), 1);
        assert!(!found[0].license_file);
    }

    #[test]
    fn old_single_packages_still_load() {
        let root = tempfile::tempdir().unwrap();
        let old = root.path().join("effects").join("telefon");
        std::fs::create_dir_all(&old).unwrap();
        std::fs::write(old.join("plugin.json"), r#"{"id":"telefon","name":"Telefon","type":"effect"}"#).unwrap();
        std::fs::write(old.join("effect.json"), EFFECT).unwrap();
        let (effects, errors) = contents_in(&bases_in(root.path()), Kind::Effect);
        assert!(errors.is_empty());
        assert_eq!(effects.len(), 1);
        // Neue Fassung ersetzt die alte, statt sie zu verdoppeln
        let m = r#"{"id":"telefon","name":"Telefon","contents":[{"type":"effect","file":"effects/telefon.json"}]}"#;
        install_into(root.path(), &files(&[("plugin.json", m), ("effects/telefon.json", EFFECT)])).unwrap();
        assert!(!old.exists());
        assert_eq!(contents_in(&bases_in(root.path()), Kind::Effect).0.len(), 1);
    }

    /// Die Vorlagen in examples/plugins/ müssen sich installieren lassen.
    #[test]
    fn files_roundtrip() {
        // Ein installiertes Paket lässt sich vollständig auslesen und anderswo wieder installieren.
        let src = tempfile::tempdir().unwrap();
        let dst = tempfile::tempdir().unwrap();
        let ex = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/plugins/nachtfahrt");
        let mut files = HashMap::new();
        for rel in ["plugin.json", "LICENSE", "README.md", "effects/kassette.json", "effects/stadion.json", "visualizers/polarlicht.js"] {
            files.insert(rel.to_string(), std::fs::read_to_string(ex.join(rel)).unwrap());
        }
        install_into(src.path(), &files).unwrap();
        let out = files_in(&bases_in(src.path()), "nachtfahrt").unwrap();
        assert_eq!(out.len(), files.len());
        install_into(dst.path(), &out).unwrap();
        assert!(files_in(&bases_in(dst.path()), "nachtfahrt").is_ok());
        assert!(files_in(&bases_in(src.path()), "../x").is_err());
    }

    #[test]
    fn export_is_a_valid_zip() {
        assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
        let mut files = HashMap::new();
        files.insert("plugin.json".to_string(), "{}".to_string());
        files.insert("effects/ä.json".to_string(), "x".repeat(100));
        let zip = zip_store(&files);
        assert_eq!(&zip[..4], b"PK\x03\x04");
        let eocd = zip.len() - 22;
        assert_eq!(&zip[eocd..eocd + 4], &0x0605_4b50u32.to_le_bytes());
        assert_eq!(u16::from_le_bytes([zip[eocd + 10], zip[eocd + 11]]), 2);
        // Prüfung mit Python, wenn vorhanden: entpackt sich sauber
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("t.amberplugin");
        std::fs::write(&path, &zip).unwrap();
        if let Ok(out) = std::process::Command::new("python3")
            .args(["-c", "import sys,zipfile; z=zipfile.ZipFile(sys.argv[1]); assert z.testzip() is None; print(len(z.read('effects/ä.json')))"])
            .arg(&path)
            .output()
        {
            if out.status.success() {
                assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "100");
            }
        }
    }

    #[test]
    fn translations_are_checked() {
        let base = r#"{"id":"t","name":"Test","type":"effect"#;
        let ok = Manifest::parse(&format!(r#"{base}","translations":{{"en":{{"Test":"Test EN"}},"pt-BR":{{"Test":"Teste"}}}}}}"#)).unwrap();
        assert_eq!(ok.translations["en"]["Test"], "Test EN");
        assert!(Manifest::parse(&format!(r#"{base}","translations":{{"../x":{{}}}}}}"#)).is_err());
        assert!(Manifest::parse(&format!(r#"{base}"}}"#)).unwrap().translations.is_empty());
    }

    #[test]
    fn examples_install() {
        let examples = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/plugins");
        let mut count = 0;
        for entry in std::fs::read_dir(&examples).unwrap() {
            let dir = entry.unwrap().path();
            if !dir.join("plugin.json").exists() {
                continue;
            }
            let mut files = HashMap::new();
            for f in walk(&dir) {
                let rel = f.strip_prefix(&dir).unwrap().to_string_lossy().replace('\\', "/");
                files.insert(rel, std::fs::read_to_string(&f).unwrap());
            }
            let root = tempfile::tempdir().unwrap();
            let m = install_into(root.path(), &files).unwrap_or_else(|e| panic!("{}: {e:#}", dir.display()));
            assert!(files.keys().any(|k| is_license(k)), "{}: LICENSE fehlt", m.id);
            count += 1;
        }
        assert!(count >= 3, "zu wenige Beispiele gefunden: {count}");
    }

    fn walk(dir: &Path) -> Vec<PathBuf> {
        let mut out = Vec::new();
        for e in std::fs::read_dir(dir).unwrap() {
            let p = e.unwrap().path();
            if p.is_dir() {
                out.extend(walk(&p));
            } else {
                out.push(p);
            }
        }
        out
    }

    #[test]
    fn rejects_bad_packages() {
        let root = tempfile::tempdir().unwrap();
        let m = r#"{"id":"x","name":"X","type":"effect"}"#;
        assert!(install_into(root.path(), &files(&[("effect.json", EFFECT)])).is_err(), "ohne plugin.json");
        assert!(install_into(root.path(), &files(&[("plugin.json", m)])).is_err(), "ohne effect.json");
        assert!(install_into(root.path(), &files(&[("plugin.json", m), ("effect.json", "{kaputt")])).is_err(), "kaputter Effekt");
        assert!(install_into(root.path(), &files(&[("plugin.json", m), ("effect.json", EFFECT), ("../boese", "x")])).is_err(), "Pfad");
        assert!(install_into(root.path(), &files(&[("plugin.json", m), ("effect.json", EFFECT), ("run.sh", "x")])).is_err(), "fremde Datei");
        assert!(install_into(root.path(), &files(&[("plugin.json", m), ("effect.json", EFFECT), ("a/b/c.json", "x")])).is_err(), "zu tief");
        let none = r#"{"id":"x","name":"X"}"#;
        assert!(install_into(root.path(), &files(&[("plugin.json", none)])).is_err(), "ohne Teile");
        let bad_id = r#"{"id":"a b","name":"X","type":"effect"}"#;
        assert!(install_into(root.path(), &files(&[("plugin.json", bad_id), ("effect.json", EFFECT)])).is_err(), "ID");
        let vis = r#"{"id":"v","name":"V","contents":[{"type":"visualizer","file":"v.json"}]}"#;
        assert!(install_into(root.path(), &files(&[("plugin.json", vis), ("v.json", "x")])).is_err(), "falsche Endung");
    }
}
