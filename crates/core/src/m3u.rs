//! Lesen und Schreiben von M3U/M3U8-Playlisten.

use std::path::{Path, PathBuf};

use crate::model::Track;

/// Liefert die Dateipfade einer M3U-Datei; relative Pfade werden zum Ordner der Datei aufgelöst.
pub fn parse(content: &str, base_dir: &Path) -> Vec<PathBuf> {
    content
        .lines()
        .map(|l| l.trim().trim_start_matches('\u{feff}'))
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|l| {
            let l = l.strip_prefix("file://").unwrap_or(l);
            let p = PathBuf::from(l.replace('\\', "/"));
            if p.is_absolute() {
                p
            } else {
                base_dir.join(p)
            }
        })
        .collect()
}

pub fn write(tracks: &[Track]) -> String {
    let mut out = String::from("#EXTM3U\n");
    for t in tracks {
        let secs = t.duration_ms / 1000;
        let name = if t.artist.is_empty() {
            t.title.clone()
        } else {
            format!("{} - {}", t.artist, t.title)
        };
        out.push_str(&format!("#EXTINF:{secs},{name}\n{}\n", t.path));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_absolute_relative_and_comments() {
        let m3u = "\u{feff}#EXTM3U\n#EXTINF:120,A - B\n/music/a.mp3\n\nsub\\b.flac\nfile:///x/c.ogg\n";
        let paths = parse(m3u, Path::new("/lists"));
        assert_eq!(
            paths,
            vec![
                PathBuf::from("/music/a.mp3"),
                PathBuf::from("/lists/sub/b.flac"),
                PathBuf::from("/x/c.ogg"),
            ]
        );
    }
}
