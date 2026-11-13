//! Durchsucht Ordner nach Audiodateien und liest deren Tags.

use std::path::Path;
use std::sync::LazyLock;
use std::time::UNIX_EPOCH;

use lofty::file::{AudioFile, TaggedFileExt};
use lofty::tag::{Accessor, ItemKey};
use regex::Regex;
use walkdir::WalkDir;

use crate::model::TrackInfo;

/// Wird erhöht, wenn sich die Auswertung der Tags ändert – dann liest der Scanner alles neu ein.
pub const SCANNER_VERSION: u32 = 3;

pub const AUDIO_EXTENSIONS: &[&str] = &[
    "mp3", "flac", "ogg", "oga", "opus", "wav", "m4a", "aac", "mp4", "aif", "aiff", "caf", "mka",
    "webm",
];

pub fn is_audio_file(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| AUDIO_EXTENSIONS.contains(&e.to_ascii_lowercase().as_str()))
        .unwrap_or(false)
}

/// Alle Audiodateien unterhalb von `root` mit ihrer Änderungszeit.
pub fn find_audio_files(root: &Path) -> Vec<(String, i64)> {
    WalkDir::new(root)
        .follow_links(true)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file() && is_audio_file(e.path()))
        .filter_map(|e| {
            let path = e.path().to_str()?.to_owned();
            Some((path, mtime(e.path())))
        })
        .collect()
}

pub fn mtime(path: &Path) -> i64 {
    std::fs::metadata(path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Liest Tags und Dauer. Fehlen Tags, wird der Dateiname als Titel genommen.
pub fn read_track_info(path: &Path, mtime: i64) -> TrackInfo {
    let mut info = TrackInfo {
        path: path.to_string_lossy().into_owned(),
        title: path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default(),
        mtime,
        ..Default::default()
    };

    let Ok(tagged) = lofty::read_from_path(path) else {
        (info.title, info.artist) = clean_title(&info.title, &info.artist);
        return info;
    };
    info.duration_ms = tagged.properties().duration().as_millis() as u64;

    if let Some(tag) = tagged.primary_tag().or_else(|| tagged.first_tag()) {
        let text = |v: Option<std::borrow::Cow<'_, str>>| {
            v.map(|s| s.trim().to_owned()).filter(|s| !s.is_empty())
        };
        if let Some(t) = text(tag.title()) {
            info.title = t;
        }
        info.artist = text(tag.artist()).unwrap_or_default();
        info.album = text(tag.album()).unwrap_or_default();
        info.genre = text(tag.genre()).unwrap_or_default();
        info.album_artist = tag
            .get_string(&ItemKey::AlbumArtist)
            .map(|s| s.trim().to_owned())
            .unwrap_or_default();
        info.year = tag.year();
        info.track_no = tag.track();
        info.disc_no = tag.disk();
        info.has_cover = !tag.pictures().is_empty();
    }
    if !info.has_cover {
        info.has_cover = folder_cover(path).is_some();
    }
    let (title, artist) = clean_title(&info.title, &info.artist);
    info.title = title;
    info.artist = artist;
    info
}

static SITE_PREFIX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^\s*(yt5s\.\w+|y2mate\.\w+|y2meta\.\w+|ssstik\.io|savefrom\.net|ytmp3\.\w+|mp3juices\.\w+|flvto\.\w+|notube\.\w+|onlymp3\.\w+)\s*[-_]\s*").unwrap()
});
static NOISE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(concat!(
        r"(?i)\s*[\(\[]\s*(\d+\s*kbps|(official\s+)?(music\s+|lyric\s+|hd\s+)?(video|audio|clip)|lyric(s)?\s*video|lyrics|audio|hq|hd|4k|visuali[sz]er|musikvideo|offizielles\s+(musik)?video)\s*[\)\]]",
        r"|_\d{2,3}k$"
    ))
    .unwrap()
});

/// Bereinigt Titel aus Online-Konvertern: Website-Präfix, Bitrate und „(Official Video)“ fallen weg;
/// fehlt der Interpret, wird „Interpret - Titel“ aufgeteilt.
pub fn clean_title(title: &str, artist: &str) -> (String, String) {
    let mut t = SITE_PREFIX.replace(title, "").into_owned();
    // mehrfach, weil Zusätze hintereinander stehen können
    for _ in 0..3 {
        t = NOISE.replace_all(&t, "").into_owned();
    }
    let mut t = t.replace('_', " ").split_whitespace().collect::<Vec<_>>().join(" ");
    let mut a = artist.trim().to_string();
    if a.is_empty() {
        if let Some((left, right)) = t.split_once(" - ") {
            if !left.trim().is_empty() && !right.trim().is_empty() {
                a = left.trim().to_string();
                t = right.trim().to_string();
            }
        }
    } else if let Some(rest) = t
        .get(..a.len())
        .filter(|head| head.eq_ignore_ascii_case(&a))
        .and_then(|_| t[a.len()..].strip_prefix(" - "))
    {
        // „Santiano - Weh Mir“ bei Interpret „Santiano“
        t = rest.trim().to_string();
    }
    if t.is_empty() {
        t = title.trim().to_string();
    }
    (t, a)
}

#[cfg(test)]
mod tests {
    use super::clean_title;

    #[test]
    fn cleans_converter_names() {
        let c = |t: &str, a: &str| clean_title(t, a);
        assert_eq!(c("yt5s.com - Eiffel 65 - Blue (Da Ba Dee) (128 kbps)", ""), ("Blue (Da Ba Dee)".into(), "Eiffel 65".into()));
        assert_eq!(c("y2meta.com - Eiffel 65 - Blue (Da Ba Dee)", ""), ("Blue (Da Ba Dee)".into(), "Eiffel 65".into()));
        assert_eq!(c("Weh Mir (Official Video)", "Santiano"), ("Weh Mir".into(), "Santiano".into()));
        assert_eq!(c("Peyton Parrish - Battle Cries (Viking Chant)_256k", ""), ("Battle Cries (Viking Chant)".into(), "Peyton Parrish".into()));
        assert_eq!(c("Santiano - Weh Mir (Official Video)", "Santiano"), ("Weh Mir".into(), "Santiano".into()));
        assert_eq!(c("Tanz mit mir", ""), ("Tanz mit mir".into(), "".into()));
        assert_eq!(c("Lotus (Music Video)", "Ummet Ozcan"), ("Lotus".into(), "Ummet Ozcan".into()));
        // „(1)“ bleibt: gleichnamige Dateien sind oft verschiedene Lieder.
        assert_eq!(c("Lied (1)", ""), ("Lied (1)".into(), "".into()));
    }
}

/// Eingebettetes Cover oder ein Bild wie `cover.jpg` im selben Ordner.
pub fn read_cover(path: &Path) -> Option<(Vec<u8>, String)> {
    if let Ok(tagged) = lofty::read_from_path(path) {
        for tag in tagged.tags() {
            let pics = tag.pictures();
            let pic = pics
                .iter()
                .find(|p| p.pic_type() == lofty::picture::PictureType::CoverFront)
                .or_else(|| pics.first());
            if let Some(pic) = pic {
                let mime = pic
                    .mime_type()
                    .map(|m| m.as_str().to_owned())
                    .unwrap_or_else(|| "image/jpeg".into());
                return Some((pic.data().to_vec(), mime));
            }
        }
    }
    let file = folder_cover(path)?;
    let mime = match file.extension().and_then(|e| e.to_str()) {
        Some("png") => "image/png",
        Some("webp") => "image/webp",
        _ => "image/jpeg",
    };
    Some((std::fs::read(&file).ok()?, mime.into()))
}

fn folder_cover(track: &Path) -> Option<std::path::PathBuf> {
    let dir = track.parent()?;
    const NAMES: &[&str] = &["cover", "folder", "front", "album", "albumart"];
    std::fs::read_dir(dir).ok()?.filter_map(Result::ok).find_map(|e| {
        let p = e.path();
        let stem = p.file_stem()?.to_str()?.to_ascii_lowercase();
        let ext = p.extension()?.to_str()?.to_ascii_lowercase();
        (NAMES.contains(&stem.as_str()) && ["jpg", "jpeg", "png", "webp"].contains(&ext.as_str()))
            .then_some(p)
    })
}
