//! Plugin-Pakete: Ein Klangeffekt oder eine Visualisierung als eigener Ordner mit Beschreibung,
//! eigener Lizenz und optionaler Anleitung – weitergegeben als ZIP-Datei.
//!
//! ```text
//! telefon/                ← liegt in effects/ bzw. visualizers/
//! ├── plugin.json         ← { "id", "name", "version", "author", "license", "type", "main", … }
//! ├── effect.json         ← das Plugin selbst (bei Visualisierungen: visualizer.js)
//! ├── LICENSE             ← Lizenz dieses Plugins
//! └── README.md           ← optional
//! ```
//!
//! Einzelne `.json`- bzw. `.js`-Dateien direkt im Plugin-Ordner gehen weiterhin.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, ensure, Context, Result};
use serde::{Deserialize, Serialize};

use crate::effects::custom::CustomPlugin;
use crate::library::Library;

/// Art eines Plugin-Pakets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Effect,
    Visualizer,
}

impl Kind {
    /// Unterordner im Datenordner.
    pub fn dir_name(self) -> &'static str {
        match self {
            Kind::Effect => "effects",
            Kind::Visualizer => "visualizers",
        }
    }

    fn default_main(self) -> &'static str {
        match self {
            Kind::Effect => "effect.json",
            Kind::Visualizer => "visualizer.js",
        }
    }

    fn main_ext(self) -> &'static str {
        match self {
            Kind::Effect => "json",
            Kind::Visualizer => "js",
        }
    }

    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "effect" | "effects" => Ok(Kind::Effect),
            "visualizer" | "visualizers" => Ok(Kind::Visualizer),
            other => bail!("Unbekannte Plugin-Art „{other}“ (erlaubt: effect, visualizer)"),
        }
    }
}

/// Inhalt von `plugin.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub kind: Kind,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub description: String,
    /// Kurzname der Lizenz, z. B. „MIT“ oder „Alle Rechte vorbehalten“.
    #[serde(default)]
    pub license: String,
    /// Datei mit dem Plugin; Standard `effect.json` bzw. `visualizer.js`.
    #[serde(default)]
    pub main: String,
    #[serde(default)]
    pub homepage: String,
}

impl Manifest {
    pub fn parse(text: &str) -> Result<Self> {
        let mut m: Manifest = serde_json::from_str(text).context("plugin.json ist kein gültiges JSON")?;
        ensure!(valid_id(&m.id), "„id“ in plugin.json: 1–32 Zeichen aus Buchstaben, Ziffern, - und _");
        ensure!(!m.name.trim().is_empty(), "„name“ in plugin.json fehlt");
        if m.main.is_empty() {
            m.main = m.kind.default_main().to_string();
        }
        ensure!(safe_file_name(&m.main), "„main“ in plugin.json muss ein einfacher Dateiname sein");
        ensure!(
            m.main.to_ascii_lowercase().ends_with(&format!(".{}", m.kind.main_ext())),
            "„main“ muss bei dieser Plugin-Art auf .{} enden",
            m.kind.main_ext()
        );
        m.name = m.name.chars().take(40).collect();
        m.version = m.version.chars().take(20).collect();
        m.author = m.author.chars().take(60).collect();
        m.license = m.license.chars().take(60).collect();
        m.description = m.description.chars().take(240).collect();
        Ok(m)
    }
}

/// Ein installiertes Paket.
#[derive(Debug, Clone, Serialize)]
pub struct Installed {
    #[serde(flatten)]
    pub manifest: Manifest,
    /// Liegt eine Lizenzdatei bei?
    pub license_file: bool,
    pub readme: bool,
    #[serde(skip)]
    pub dir: PathBuf,
}

fn valid_id(id: &str) -> bool {
    (1..=32).contains(&id.len()) && id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

fn safe_file_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 64
        && !name.starts_with('.')
        && name.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
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
            manifest,
            license_file: names.iter().any(|n| is_license(n)),
            readme: names.iter().any(|n| is_readme(n)),
            dir: dir.to_path_buf(),
        }
    }))
}

/// Alle Pakete einer Art (Unterordner mit `plugin.json`), sortiert nach Ordnername.
/// Fehlerhafte Pakete kommen mit Meldung in die zweite Liste.
pub fn list_in(base: &Path, kind: Kind) -> (Vec<Installed>, Vec<String>) {
    let mut ok = Vec::new();
    let mut errors = Vec::new();
    let Ok(rd) = std::fs::read_dir(base) else {
        return (ok, errors);
    };
    let mut dirs: Vec<PathBuf> = rd.filter_map(Result::ok).map(|e| e.path()).filter(|p| p.is_dir()).collect();
    dirs.sort();
    for dir in dirs {
        let folder = dir.file_name().unwrap_or_default().to_string_lossy().into_owned();
        match read(&dir) {
            None => {}
            Some(Ok(p)) if p.manifest.kind == kind => ok.push(p),
            Some(Ok(_)) => errors.push(format!("{folder}/: Plugin-Art passt nicht in diesen Ordner")),
            Some(Err(e)) => errors.push(format!("{folder}/: {e:#}")),
        }
    }
    (ok, errors)
}

/// Pakete einer Art im Datenordner von Amber Player.
pub fn list(kind: Kind) -> (Vec<Installed>, Vec<String>) {
    list_in(&Library::plugin_dir(kind.dir_name()), kind)
}

/// Text der Plugin-Datei eines Pakets.
pub fn main_text(p: &Installed) -> Result<String> {
    let path = p.dir.join(&p.manifest.main);
    ensure!(
        std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0) <= 512 * 1024,
        "{} ist größer als 512 KB",
        p.manifest.main
    );
    std::fs::read_to_string(&path).with_context(|| format!("{} fehlt oder ist nicht lesbar", p.manifest.main))
}

/// Paket aus seinen Dateien installieren (Name → Inhalt, alles Text). Ein vorhandenes Paket mit
/// derselben `id` wird ersetzt. Gibt das Manifest zurück.
pub fn install_into(base_root: &Path, files: &HashMap<String, String>) -> Result<Manifest> {
    let manifest_text = files.get("plugin.json").ok_or_else(|| anyhow!("Im Paket fehlt die Datei plugin.json"))?;
    let manifest = Manifest::parse(manifest_text)?;
    let main = files
        .get(&manifest.main)
        .ok_or_else(|| anyhow!("Im Paket fehlt die Plugin-Datei {}", manifest.main))?;
    let total: usize = files.values().map(String::len).sum();
    ensure!(total <= 1024 * 1024, "Das Paket ist zu groß (höchstens 1 MB)");
    for name in files.keys() {
        ensure!(safe_file_name(name), "Ungültiger Dateiname im Paket: {name}");
        let allowed = name == "plugin.json" || *name == manifest.main || is_license(name) || is_readme(name);
        ensure!(allowed, "Unerwartete Datei im Paket: {name} (erlaubt: plugin.json, {}, LICENSE, README)", manifest.main);
    }
    if manifest.kind == Kind::Effect {
        CustomPlugin::parse(main, &format!("{}/{}", manifest.id, manifest.main)).context("Der Klangeffekt ist fehlerhaft")?;
    }

    let base = base_root.join(manifest.kind.dir_name());
    std::fs::create_dir_all(&base)?;
    let target = base.join(&manifest.id);
    // Erst vollständig in einen Nachbarordner schreiben, dann austauschen – so bleibt bei einem
    // Fehler das alte Paket heil.
    let tmp = base.join(format!(".{}-neu", manifest.id));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp)?;
    for (name, text) in files {
        std::fs::write(tmp.join(name), text)?;
    }
    if target.exists() {
        std::fs::remove_dir_all(&target)?;
    }
    std::fs::rename(&tmp, &target)?;
    Ok(manifest)
}

/// Paket in den Datenordner von Amber Player installieren.
pub fn install(files: &HashMap<String, String>) -> Result<Manifest> {
    install_into(&Library::data_dir(), files)
}

/// Paket entfernen.
pub fn remove(kind: Kind, id: &str) -> Result<()> {
    ensure!(valid_id(id), "Ungültige Plugin-ID");
    let dir = Library::plugin_dir(kind.dir_name()).join(id);
    ensure!(dir.join("plugin.json").exists(), "Plugin „{id}“ ist nicht installiert");
    std::fs::remove_dir_all(&dir)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn files(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs.iter().map(|(a, b)| (a.to_string(), b.to_string())).collect()
    }

    const EFFECT: &str = r#"{"id":"telefon","name":"Telefon","chain":[{"type":"lowpass","freq":3000}]}"#;

    #[test]
    fn install_list_remove() {
        let root = tempfile::tempdir().unwrap();
        let manifest = r#"{"id":"telefon","name":"Telefon","type":"effect","version":"1.0","license":"MIT"}"#;
        let m = install_into(root.path(), &files(&[("plugin.json", manifest), ("effect.json", EFFECT), ("LICENSE", "MIT …")])).unwrap();
        assert_eq!(m.main, "effect.json");
        let (found, errors) = list_in(&root.path().join("effects"), Kind::Effect);
        assert!(errors.is_empty());
        assert_eq!(found.len(), 1);
        assert!(found[0].license_file);
        assert_eq!(found[0].manifest.license, "MIT");
        assert!(main_text(&found[0]).unwrap().contains("lowpass"));
        // nochmal installieren ersetzt
        install_into(root.path(), &files(&[("plugin.json", manifest), ("effect.json", EFFECT)])).unwrap();
        let (found, _) = list_in(&root.path().join("effects"), Kind::Effect);
        assert!(!found[0].license_file);
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
        let bad_id = r#"{"id":"a b","name":"X","type":"effect"}"#;
        assert!(install_into(root.path(), &files(&[("plugin.json", bad_id), ("effect.json", EFFECT)])).is_err(), "ID");
        let vis = r#"{"id":"v","name":"V","type":"visualizer","main":"v.json"}"#;
        assert!(install_into(root.path(), &files(&[("plugin.json", vis), ("v.json", "x")])).is_err(), "falsche Endung");
    }
}
