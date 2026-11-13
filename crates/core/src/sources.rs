//! Quellen-Plugins: externe Programme, die Musik aus dem Netz finden und abspielbar machen
//! (z. B. ein Online-Musikdienst). Jedes Plugin liegt in einem eigenen Ordner unter
//! `~/.local/share/amber-player/sources/<name>/` mit einer `plugin.json`:
//!
//! ```json
//! { "id": "meine-quelle", "name": "Meine Quelle", "icon": "♪", "command": ["python3", "plugin.py"],
//!   "features": ["search", "playlist"] }
//! ```
//!
//! Amber Player startet das Programm für jede Anfrage im Plugin-Ordner, schreibt eine Zeile JSON
//! auf stdin und liest die Antwort (JSON) von stdout. Anfragen:
//! - `{"action":"search","query":"…","limit":25}` → `{"items":[Eintrag, …]}`
//! - `{"action":"playlist","url":"…"}` → `{"name":"…","items":[…]}`
//! - `{"action":"resolve","id":"…"}` → `{"url":"…","headers":{…}}` (direkte Audio-Adresse)
//!
//! Eintrag: `{"id","title","artist","duration_ms","thumbnail"}`. Fehler: `{"error":"…"}`.

use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use anyhow::{anyhow, bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

const TIMEOUT: Duration = Duration::from_secs(120);

/// Präfix der Pfade von Online-Titeln in der Bibliothek: `plugin://<plugin>/<id>`.
pub const PATH_PREFIX: &str = "plugin://";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceInfo {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub icon: String,
    #[serde(default)]
    pub features: Vec<String>,
    #[serde(default, skip_serializing)]
    pub command: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct SourceItem {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub duration_ms: u64,
    pub thumbnail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct SourcePlaylist {
    pub name: String,
    pub items: Vec<SourceItem>,
}

/// Direkt abspielbare Adresse eines Eintrags (meist nur einige Stunden gültig).
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct Resolved {
    pub url: String,
    pub headers: std::collections::HashMap<String, String>,
}

pub struct SourcePlugin {
    pub info: SourceInfo,
    dir: PathBuf,
}

impl SourcePlugin {
    /// Eine Anfrage an das Plugin schicken und die Antwort lesen.
    fn call(&self, request: Value) -> Result<Value> {
        let (prog, args) = self.info.command.split_first().ok_or_else(|| anyhow!("Plugin ohne „command“"))?;
        let mut child = Command::new(prog)
            .args(args)
            .current_dir(&self.dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .with_context(|| format!("Plugin {} lässt sich nicht starten ({prog})", self.info.name))?;
        {
            let mut stdin = child.stdin.take().unwrap();
            writeln!(stdin, "{request}")?;
        }
        let mut stdout = child.stdout.take().unwrap();
        let mut stderr = child.stderr.take().unwrap();
        let out_thread = std::thread::spawn(move || {
            let mut s = String::new();
            let _ = stdout.read_to_string(&mut s);
            s
        });
        let err_thread = std::thread::spawn(move || {
            let mut s = String::new();
            let _ = stderr.read_to_string(&mut s);
            s
        });
        let start = Instant::now();
        loop {
            if child.try_wait()?.is_some() {
                break;
            }
            if start.elapsed() > TIMEOUT {
                let _ = child.kill();
                bail!("Plugin {} antwortet nicht", self.info.name);
            }
            std::thread::sleep(Duration::from_millis(30));
        }
        let out = out_thread.join().unwrap_or_default();
        let err = err_thread.join().unwrap_or_default();
        if !err.trim().is_empty() {
            log::debug!("Plugin {}: {}", self.info.id, err.trim());
        }
        let line = out.lines().rev().find(|l| l.trim_start().starts_with('{')).ok_or_else(|| {
            anyhow!("Plugin {} hat keine Antwort geliefert {}", self.info.name, err.lines().last().unwrap_or(""))
        })?;
        let v: Value = serde_json::from_str(line).context("Antwort des Plugins ist kein JSON")?;
        if let Some(e) = v.get("error").and_then(Value::as_str) {
            bail!("{e}");
        }
        Ok(v)
    }

    pub fn search(&self, query: &str, limit: u32) -> Result<Vec<SourceItem>> {
        let v = self.call(json!({ "action": "search", "query": query, "limit": limit }))?;
        Ok(serde_json::from_value(v["items"].clone()).unwrap_or_default())
    }

    pub fn playlist(&self, url: &str) -> Result<SourcePlaylist> {
        let v = self.call(json!({ "action": "playlist", "url": url }))?;
        Ok(serde_json::from_value(v).unwrap_or_default())
    }

    pub fn resolve(&self, id: &str) -> Result<Resolved> {
        let v = self.call(json!({ "action": "resolve", "id": id }))?;
        let r: Resolved = serde_json::from_value(v)?;
        if !(r.url.starts_with("http://") || r.url.starts_with("https://")) {
            bail!("Plugin {} hat keine Abspiel-Adresse geliefert", self.info.name);
        }
        Ok(r)
    }
}

pub struct Sources {
    plugins: Vec<SourcePlugin>,
    pub errors: Vec<String>,
}

impl Sources {
    /// Lädt alle Plugins (Unterordner mit `plugin.json`) aus `dir`.
    pub fn load(dir: &Path) -> Self {
        let mut plugins = Vec::new();
        let mut errors = Vec::new();
        let mut dirs: Vec<PathBuf> = std::fs::read_dir(dir)
            .map(|rd| rd.filter_map(Result::ok).map(|e| e.path()).filter(|p| p.is_dir()).collect())
            .unwrap_or_default();
        dirs.sort();
        for d in dirs {
            let manifest = d.join("plugin.json");
            if !manifest.is_file() {
                continue;
            }
            let result = std::fs::read_to_string(&manifest)
                .map_err(anyhow::Error::from)
                .and_then(|t| serde_json::from_str::<SourceInfo>(&t).map_err(Into::into))
                .and_then(|info| {
                    if info.id.is_empty() || !info.id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
                        bail!("„id“ fehlt oder enthält ungültige Zeichen");
                    }
                    if info.command.is_empty() {
                        bail!("„command“ fehlt");
                    }
                    Ok(info)
                });
            match result {
                Ok(info) if plugins.iter().any(|p: &SourcePlugin| p.info.id == info.id) => {
                    errors.push(format!("{}: Die ID „{}“ gibt es schon", d.display(), info.id))
                }
                Ok(info) => plugins.push(SourcePlugin { info, dir: d }),
                Err(e) => errors.push(format!("{}: {e:#}", d.display())),
            }
        }
        for e in &errors {
            log::warn!("Quellen-Plugin: {e}");
        }
        Self { plugins, errors }
    }

    pub fn list(&self) -> Vec<SourceInfo> {
        self.plugins.iter().map(|p| p.info.clone()).collect()
    }

    pub fn get(&self, id: &str) -> Result<&SourcePlugin> {
        self.plugins
            .iter()
            .find(|p| p.info.id == id)
            .ok_or_else(|| anyhow!("Quellen-Plugin „{id}“ ist nicht installiert"))
    }
}

/// `plugin://meine-quelle/abc` → („meine-quelle“, „abc“)
pub fn parse_path(path: &str) -> Option<(&str, &str)> {
    path.strip_prefix(PATH_PREFIX)?.split_once('/')
}

pub fn make_path(plugin: &str, id: &str) -> String {
    format!("{PATH_PREFIX}{plugin}/{id}")
}

// ---------- Gemeinsame Abläufe für Oberfläche, HTTP-API und MCP ----------

impl Sources {
    /// Plugin nach ID, ohne ID das erste installierte.
    pub fn pick(&self, id: Option<&str>) -> Result<&SourcePlugin> {
        match id.filter(|s| !s.is_empty()) {
            Some(id) => self.get(id),
            None => self.plugins.first().ok_or_else(|| anyhow!("Kein Quellen-Plugin installiert")),
        }
    }
}

/// Online suchen; die Treffer werden als (zunächst unsichtbare) Titel angelegt, damit sie
/// sich wie eigene Titel abspielen, einreihen und in Playlisten legen lassen.
pub fn search_tracks(lib: &crate::Library, sources: &Sources, source: Option<&str>, query: &str, limit: u32) -> Result<Vec<crate::Track>> {
    let plugin = sources.pick(source)?;
    let items = plugin.search(query, limit)?;
    let ids = lib.upsert_online(&plugin.info.id, &plugin.info.name, &items)?;
    lib.tracks_by_ids(&ids)
}

/// Online-Playlist lesen → (Name, Titel).
pub fn playlist_tracks(lib: &crate::Library, sources: &Sources, source: Option<&str>, url: &str) -> Result<(String, Vec<crate::Track>)> {
    let plugin = sources.pick(source)?;
    let pl = plugin.playlist(url)?;
    if pl.items.is_empty() {
        bail!("Die Playlist ist leer oder nicht öffentlich");
    }
    let ids = lib.upsert_online(&plugin.info.id, &plugin.info.name, &pl.items)?;
    Ok((pl.name, lib.tracks_by_ids(&ids)?))
}

/// Online-Playlist als Amber-Playlist übernehmen; liefert die neue Playlist-ID.
pub fn import_playlist(lib: &crate::Library, sources: &Sources, source: Option<&str>, url: &str) -> Result<i64> {
    let (name, tracks) = playlist_tracks(lib, sources, source, url)?;
    let id = lib.create_playlist(&name)?;
    lib.add_to_playlist(id, &tracks.iter().map(|t| t.id).collect::<Vec<_>>())?;
    Ok(id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths() {
        assert_eq!(parse_path("plugin://meine-quelle/abc_D-1"), Some(("meine-quelle", "abc_D-1")));
        assert_eq!(parse_path("/home/x.mp3"), None);
        assert_eq!(make_path("meine-quelle", "x"), "plugin://meine-quelle/x");
    }

    #[test]
    fn calls_plugin_process() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("echo");
        std::fs::create_dir(&p).unwrap();
        std::fs::write(
            p.join("plugin.json"),
            r#"{"id":"echo","name":"Echo","command":["sh","run.sh"],"features":["search"]}"#,
        )
        .unwrap();
        // Antwortet auf jede Anfrage mit einem Treffer; Fehler bei „resolve“.
        std::fs::write(
            p.join("run.sh"),
            "read line\ncase \"$line\" in *resolve*) echo '{\"error\":\"geht nicht\"}';; \
             *) echo 'Logzeile'; echo '{\"items\":[{\"id\":\"1\",\"title\":\"Lied\",\"artist\":\"A\",\"duration_ms\":1000}]}';; esac\n",
        )
        .unwrap();
        let s = Sources::load(dir.path());
        assert!(s.errors.is_empty(), "{:?}", s.errors);
        let plugin = s.get("echo").unwrap();
        let items = plugin.search("x", 5).unwrap();
        assert_eq!(items[0].title, "Lied");
        assert_eq!(plugin.resolve("1").unwrap_err().to_string(), "geht nicht");
    }
}
