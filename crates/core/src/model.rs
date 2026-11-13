use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Track {
    pub id: i64,
    pub path: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub album_artist: String,
    pub genre: String,
    pub year: Option<u32>,
    pub track_no: Option<u32>,
    pub disc_no: Option<u32>,
    pub duration_ms: u64,
    pub has_cover: bool,
    pub play_count: u32,
    pub favorite: bool,
}

/// Metadaten einer Datei, wie der Scanner sie liest (noch ohne Datenbank-ID).
#[derive(Debug, Clone, Default)]
pub struct TrackInfo {
    pub path: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub album_artist: String,
    pub genre: String,
    pub year: Option<u32>,
    pub track_no: Option<u32>,
    pub disc_no: Option<u32>,
    pub duration_ms: u64,
    pub has_cover: bool,
    pub mtime: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Album {
    pub name: String,
    pub artist: String,
    pub year: Option<u32>,
    pub track_count: u32,
    /// Ein Titel des Albums mit eingebettetem Cover (für die Cover-Anzeige).
    pub cover_track_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NamedCount {
    pub name: String,
    pub track_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Playlist {
    pub id: i64,
    pub name: String,
    pub track_count: u32,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum Repeat {
    #[default]
    Off,
    All,
    One,
}

impl Repeat {
    pub fn cycle(self) -> Self {
        match self {
            Repeat::Off => Repeat::All,
            Repeat::All => Repeat::One,
            Repeat::One => Repeat::Off,
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "off" => Some(Repeat::Off),
            "all" => Some(Repeat::All),
            "one" => Some(Repeat::One),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum PlayState {
    #[default]
    Stopped,
    Playing,
    Paused,
}

/// Läuft gerade Internetradio: Sender und – falls gemeldet – der Songtitel.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RadioNow {
    pub station: crate::radio::Station,
    pub title: Option<String>,
}

/// Aktueller Zustand des Players – wird an UI, HTTP-API und MCP ausgeliefert.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Status {
    pub state: PlayState,
    pub track: Option<Track>,
    /// Gesetzt, solange Internetradio läuft (dann ist `track` leer).
    pub radio: Option<RadioNow>,
    pub position_ms: u64,
    pub duration_ms: u64,
    /// 0–100
    pub volume: u8,
    pub muted: bool,
    pub shuffle: bool,
    pub repeat: Repeat,
    pub sleep_remaining_s: Option<u64>,
    pub queue_length: usize,
    pub error: Option<String>,
}
