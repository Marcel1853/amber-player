//! Fehlende Cover aus dem Internet laden (Deezer, ersatzweise MusicBrainz/Cover Art Archive).
//!
//! Gesendet werden nur Interpret, Titel und Album. Ein Treffer wird nur übernommen, wenn
//! Interpret und Titel bzw. Album wirklich passen – lieber kein Cover als ein falsches.
//! Jede Datei bekommt ihr eigenes Cover; nur Titel mit gleichem Album-Tag teilen sich eins.

use std::collections::HashMap;
use std::io::Read;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::{anyhow, bail, Result};
use serde_json::Value;

use crate::library::{CoverJob, Library};

const USER_AGENT: &str = concat!("AmberPlayer/", env!("CARGO_PKG_VERSION"), " ( lokaler Musik-Player )");
const TIMEOUT: Duration = Duration::from_secs(12);
const MAX_IMAGE: u64 = 6 * 1024 * 1024;
/// Pause zwischen Anfragen, damit die Dienste nicht überlastet werden.
const PAUSE: Duration = Duration::from_millis(350);

pub fn cover_dir() -> PathBuf {
    Library::plugin_dir("covers")
}

/// Kleinbuchstaben, ohne Klammerzusätze und Satzzeichen.
fn norm(s: &str) -> String {
    let mut out = String::new();
    let mut depth = 0;
    for c in s.chars() {
        match c {
            '(' | '[' => depth += 1,
            ')' | ']' => depth = (depth - 1).max(0),
            _ if depth > 0 => {}
            c if c.is_alphanumeric() => out.extend(c.to_lowercase()),
            _ => out.push(' '),
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn artist_matches(want: &str, got: &str) -> bool {
    let (w, g) = (norm(want), norm(got));
    !w.is_empty() && !g.is_empty() && (w == g || g.contains(&w) || w.contains(&g))
}

fn title_score(want: &str, got: &str) -> Option<i32> {
    let (w, g) = (norm(want), norm(got));
    if w.len() < 2 || g.is_empty() {
        return None;
    }
    let mut score = if w == g {
        3
    } else if g.starts_with(&w) || w.starts_with(&g) {
        1
    } else {
        return None;
    };
    // Live-/Remix-Fassungen nur, wenn die eigene Datei auch so heißt.
    let special = ["live", "remix", "edit", "mix", "acoustic", "akustik", "karaoke", "instrumental"];
    let (wl, gl) = (want.to_lowercase(), got.to_lowercase());
    if special.iter().any(|s| gl.contains(s) && !wl.contains(s)) {
        score -= 2;
    }
    (score > 0).then_some(score)
}

fn get_json(url: &str, query: &[(&str, &str)]) -> Result<Value> {
    let mut req = ureq::get(url).set("User-Agent", USER_AGENT).timeout(TIMEOUT);
    for (k, v) in query {
        req = req.query(k, v);
    }
    Ok(req.call()?.into_json()?)
}

fn download(url: &str) -> Result<Vec<u8>> {
    let resp = ureq::get(url).set("User-Agent", USER_AGENT).timeout(TIMEOUT).call()?;
    let ct = resp.content_type().to_string();
    if !ct.starts_with("image/") {
        bail!("kein Bild ({ct})");
    }
    let mut data = Vec::new();
    resp.into_reader().take(MAX_IMAGE).read_to_end(&mut data)?;
    Ok(data)
}

/// Deezer: Titel suchen und das Cover seines Albums nehmen.
fn deezer_track(artist: &str, title: &str) -> Result<Option<String>> {
    let q = format!("{artist} {}", norm(title));
    let v = get_json("https://api.deezer.com/search", &[("q", &q), ("limit", "10")])?;
    let best = v["data"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|d| artist_matches(artist, d["artist"]["name"].as_str().unwrap_or("")))
        .filter_map(|d| title_score(title, d["title"].as_str().unwrap_or("")).map(|s| (s, d)))
        .max_by_key(|(s, _)| *s);
    Ok(best.and_then(|(_, d)| d["album"]["cover_big"].as_str().map(String::from)))
}

/// Deezer: Album suchen.
fn deezer_album(artist: &str, album: &str) -> Result<Option<String>> {
    let q = format!("{artist} {}", norm(album));
    let v = get_json("https://api.deezer.com/search/album", &[("q", &q), ("limit", "10")])?;
    let best = v["data"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|d| artist_matches(artist, d["artist"]["name"].as_str().unwrap_or("")))
        .filter_map(|d| title_score(album, d["title"].as_str().unwrap_or("")).map(|s| (s, d)))
        .max_by_key(|(s, _)| *s);
    Ok(best.and_then(|(_, d)| d["cover_big"].as_str().map(String::from)))
}

/// MusicBrainz + Cover Art Archive als Ersatz für Alben.
fn musicbrainz_album(artist: &str, album: &str) -> Result<Option<String>> {
    let esc = |s: &str| s.replace('"', "");
    let q = format!("release:\"{}\" AND artist:\"{}\"", esc(album), esc(artist));
    let v = get_json("https://musicbrainz.org/ws/2/release/", &[("query", &q), ("fmt", "json"), ("limit", "5")])?;
    for r in v["releases"].as_array().into_iter().flatten() {
        let credit = r["artist-credit"][0]["name"].as_str().unwrap_or("");
        if artist_matches(artist, credit) && title_score(album, r["title"].as_str().unwrap_or("")).is_some() {
            if let Some(id) = r["id"].as_str() {
                return Ok(Some(format!("https://coverartarchive.org/release/{id}/front-500")));
            }
        }
    }
    Ok(None)
}

fn find_cover(job: &CoverJob) -> Result<Option<Vec<u8>>> {
    let artist = if job.album_artist.is_empty() { &job.artist } else { &job.album_artist };
    let mut urls = Vec::new();
    if !job.album.is_empty() {
        if let Some(u) = deezer_album(artist, &job.album)? {
            urls.push(u);
        }
        std::thread::sleep(PAUSE);
    }
    if urls.is_empty() {
        if let Some(u) = deezer_track(&job.artist, &job.title)? {
            urls.push(u);
        }
    }
    if urls.is_empty() && !job.album.is_empty() {
        std::thread::sleep(Duration::from_secs(1)); // MusicBrainz: höchstens 1 Anfrage pro Sekunde
        if let Some(u) = musicbrainz_album(artist, &job.album)? {
            urls.push(u);
        }
    }
    for u in urls {
        match download(&u) {
            Ok(data) if data.len() > 1000 => return Ok(Some(data)),
            Ok(_) => {}
            Err(e) => log::debug!("Cover {u}: {e:#}"),
        }
    }
    Ok(None)
}

/// Sucht Cover für alle Titel ohne Bild, die noch nicht geprüft wurden.
/// Liefert die Zahl der gefundenen Cover. `progress(erledigt, gesamt)`.
pub fn fetch_missing(lib: &Library, progress: &mut dyn FnMut(usize, usize)) -> Result<usize> {
    let jobs = lib.cover_jobs()?;
    let dir = cover_dir();
    let total = jobs.len();
    let mut found = 0;
    // Gleicher Album-Tag (mit Interpret) → dasselbe Cover, nur einmal suchen.
    let mut albums: HashMap<(String, String), Option<PathBuf>> = HashMap::new();
    let mut failures = 0;
    for (i, job) in jobs.iter().enumerate() {
        progress(i, total);
        // Ohne Interpret ist die Verwechslungsgefahr zu groß.
        if job.artist.is_empty() && job.album_artist.is_empty() {
            lib.set_cover_file(job.id, None)?;
            continue;
        }
        let album_key = (!job.album.is_empty()).then(|| {
            let a = if job.album_artist.is_empty() { &job.artist } else { &job.album_artist };
            (norm(a), norm(&job.album))
        });
        if let Some(cached) = album_key.as_ref().and_then(|k| albums.get(k)) {
            lib.set_cover_file(job.id, cached.as_deref())?;
            found += cached.is_some() as usize;
            continue;
        }
        let result = find_cover(job);
        std::thread::sleep(PAUSE);
        let file = match result {
            Ok(Some(data)) => {
                let name = match &album_key {
                    Some((a, b)) => format!("album-{:016x}.jpg", fxhash(&format!("{a}\u{1}{b}"))),
                    None => format!("track-{}.jpg", job.id),
                };
                let path = dir.join(name);
                std::fs::write(&path, data)?;
                Some(path)
            }
            Ok(None) => None,
            Err(e) => {
                // Netzfehler: nicht als „geprüft“ markieren, beim nächsten Mal erneut versuchen.
                log::warn!("Cover-Suche für „{}“: {e:#}", job.title);
                failures += 1;
                if failures >= 5 {
                    return Err(anyhow!("Cover-Dienste nicht erreichbar: {e:#}"));
                }
                continue;
            }
        };
        if let Some(k) = album_key {
            albums.insert(k, file.clone());
        }
        found += file.is_some() as usize;
        lib.set_cover_file(job.id, file.as_deref())?;
    }
    progress(total, total);
    Ok(found)
}

/// Kleiner, stabiler Hash für Dateinamen (FNV-1a).
fn fxhash(s: &str) -> u64 {
    s.bytes().fold(0xcbf29ce484222325, |h, b| (h ^ b as u64).wrapping_mul(0x100000001b3))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matching_rules() {
        assert!(artist_matches("Santiano", "Santiano"));
        assert!(artist_matches("Eiffel 65", "Eiffel 65 feat. Someone"));
        assert!(!artist_matches("Santiano", "Helene Fischer"));
        assert!(!artist_matches("", "Santiano"));
        assert_eq!(title_score("Weh Mir", "Weh mir"), Some(3));
        // Live-Fassung wird abgewertet, wenn die Datei keine ist
        assert!(title_score("Weh Mir", "Weh mir (Live / Waldbühne Berlin / 2016)").unwrap() < 3);
        assert_eq!(title_score("Blue (Da Ba Dee)", "Blue (Da Ba Dee) (Samuele Sartini Remix)"), Some(1));
        assert_eq!(title_score("Blue", "Red"), None);
        assert_eq!(norm("Weh Mir (Official Video)!"), "weh mir");
    }
}
