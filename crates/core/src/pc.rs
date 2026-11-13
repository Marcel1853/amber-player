//! Verbindung vom Handy zum Amber Player am PC (über dessen HTTP-API im Heimnetz).
//!
//! Das Handy koppelt sich mit Adresse und Token des PCs. Danach kann es den PC fernsteuern
//! und Titel vom PC aufs Handy streamen: Solche Titel liegen in der Handy-Bibliothek als
//! `plugin://pc/<id>.<endung>` und werden beim Abspielen über `/stream/track/…` geladen.

use std::time::Duration;

use anyhow::{anyhow, bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::library::Library;
use crate::model::Track;
use crate::sources::SourceItem;

/// Kennung der eingebauten „Quelle“ für Titel vom PC.
pub const SOURCE_ID: &str = "pc";

const KEY: &str = "pc_link";
const TIMEOUT: Duration = Duration::from_secs(6);

/// Gespeicherte Kopplung mit einem PC.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct PcLink {
    /// z. B. `http://192.168.178.35:7878`
    pub url: String,
    pub token: String,
}

impl PcLink {
    pub fn load(lib: &Library) -> Option<Self> {
        lib.setting_as::<Self>(KEY).filter(|l| !l.url.is_empty())
    }

    pub fn save(&self, lib: &Library) -> Result<()> {
        lib.set_setting_json(KEY, self)
    }

    pub fn remove(lib: &Library) -> Result<()> {
        lib.set_setting_json(KEY, &Self::default())
    }

    /// Liest einen Kopplungs-Link: `amberplayer://pair?url=…&token=…` (aus dem QR-Code am PC)
    /// oder eine API-Adresse wie `http://192.168.178.35:7878/api?token=…`.
    pub fn parse(link: &str) -> Result<Self> {
        let link = link.trim();
        let (base, query) = link.split_once('?').unwrap_or((link, ""));
        let param = |name: &str| {
            query.split('&').find_map(|kv| {
                let (k, v) = kv.split_once('=')?;
                (k == name).then(|| percent_decode(v))
            })
        };
        let token = param("token").unwrap_or_default();
        let url = if base.starts_with("amberplayer://") {
            param("url").ok_or_else(|| anyhow!("Im Kopplungs-Link fehlt die Adresse des PCs"))?
        } else {
            base.to_string()
        };
        let url = url.trim().trim_end_matches('/');
        let url = url.strip_suffix("/api").unwrap_or(url).trim_end_matches('/');
        let url = if url.contains("://") { url.to_string() } else { format!("http://{url}") };
        if !(url.starts_with("http://") || url.starts_with("https://")) || url.len() < 10 {
            bail!("Das ist keine gültige Adresse eines Amber Players");
        }
        Ok(Self { url, token })
    }

    fn with_token(&self, path: &str) -> String {
        let sep = if path.contains('?') { '&' } else { '?' };
        if self.token.is_empty() {
            format!("{}{path}", self.url)
        } else {
            format!("{}{path}{sep}token={}", self.url, self.token)
        }
    }

    fn request(&self, method: &str, path: &str) -> Result<Value> {
        let resp = ureq::request(method, &self.with_token(path)).timeout(TIMEOUT).call();
        match resp {
            Ok(r) => {
                let text = r.into_string()?;
                Ok(serde_json::from_str(&text).unwrap_or(Value::Null))
            }
            Err(ureq::Error::Status(401 | 403, _)) => {
                bail!("Der PC hat die Verbindung abgelehnt – bitte am PC neu koppeln (QR-Code in den Einstellungen)")
            }
            Err(ureq::Error::Status(code, r)) => {
                let msg = r.into_string().unwrap_or_default();
                let msg = serde_json::from_str::<Value>(&msg)
                    .ok()
                    .and_then(|v| v["error"].as_str().map(str::to_string))
                    .unwrap_or(msg);
                bail!("PC meldet Fehler {code}: {msg}")
            }
            Err(e) => Err(anyhow!(e)).context(
                "PC nicht erreichbar – läuft Amber Player am PC, ist er im Heimnetz freigegeben und ist das Handy im selben WLAN?",
            ),
        }
    }

    /// GET an die API des PCs, z. B. `/api/status`.
    pub fn get(&self, path: &str) -> Result<Value> {
        self.request("GET", path)
    }

    /// POST (Befehle wie `/api/toggle`).
    pub fn post(&self, path: &str) -> Result<Value> {
        self.request("POST", path)
    }

    /// Bild (Cover) vom PC laden.
    pub fn bytes(&self, path: &str) -> Result<(Vec<u8>, String)> {
        let r = ureq::get(&self.with_token(path)).timeout(TIMEOUT).call()?;
        let mime = r.content_type().to_string();
        let mut data = Vec::new();
        std::io::Read::read_to_end(&mut std::io::Read::take(r.into_reader(), 4 * 1024 * 1024), &mut data)?;
        Ok((data, mime))
    }

    /// Stream-Adresse eines PC-Titels (`file` = `<id>.<endung>`).
    pub fn stream_url(&self, file: &str) -> String {
        self.with_token(&format!("/stream/track/{file}"))
    }

    /// Titel vom PC in die Handy-Bibliothek übernehmen (als Stream, ohne Download).
    /// Liefert die IDs auf dem Handy in derselben Reihenfolge.
    pub fn import_tracks(&self, lib: &Library, tracks: &[Track], album: &str) -> Result<Vec<i64>> {
        let items: Vec<SourceItem> = tracks
            .iter()
            .map(|t| SourceItem {
                id: format!("{}.{}", t.id, ext(&t.path)),
                title: t.title.clone(),
                artist: t.artist.clone(),
                duration_ms: t.duration_ms,
                thumbnail: if t.has_cover { self.with_token(&format!("/api/cover/{}", t.id)) } else { String::new() },
            })
            .collect();
        lib.upsert_online(SOURCE_ID, album, &items)
    }
}

fn ext(path: &str) -> String {
    std::path::Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .filter(|e| e.len() <= 5 && e.chars().all(|c| c.is_ascii_alphanumeric()))
        .unwrap_or_else(|| "mp3".into())
}

fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let hex = bytes.get(i + 1..i + 3).and_then(|h| std::str::from_utf8(h).ok()).and_then(|h| u8::from_str_radix(h, 16).ok());
        match (bytes[i], hex) {
            (b'%', Some(b)) => {
                out.push(b);
                i += 3;
                continue;
            }
            (b'+', _) => out.push(b' '),
            (b, _) => out.push(b),
        }
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_links() {
        let l = PcLink::parse("amberplayer://pair?url=http%3A%2F%2F192.168.178.35%3A7878&token=abc").unwrap();
        assert_eq!(l, PcLink { url: "http://192.168.178.35:7878".into(), token: "abc".into() });
        let l = PcLink::parse("http://192.168.178.35:7878/api?token=xyz").unwrap();
        assert_eq!(l, PcLink { url: "http://192.168.178.35:7878".into(), token: "xyz".into() });
        let l = PcLink::parse("192.168.178.35:7878").unwrap();
        assert_eq!(l.url, "http://192.168.178.35:7878");
        assert!(PcLink::parse("amberplayer://pair?token=abc").is_err());
    }

    #[test]
    fn urls() {
        let l = PcLink { url: "http://pc:7878".into(), token: "t".into() };
        assert_eq!(l.stream_url("5.flac"), "http://pc:7878/stream/track/5.flac?token=t");
        assert_eq!(l.with_token("/api/search?q=a"), "http://pc:7878/api/search?q=a&token=t");
        assert_eq!(percent_decode("a%20b+c"), "a b c");
        assert_eq!(percent_decode("100%"), "100%");
    }
}
