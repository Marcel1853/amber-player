//! Kern von Amber Player: Bibliothek, Playlisten, Warteschlange, Effekt-Plugins und Audio-Engine.

pub mod analyzer;
#[cfg(feature = "server")]
pub mod api;
pub mod covers;
pub mod decoder;
pub mod effects;
pub mod httpsource;
pub mod jssource;
pub mod library;
pub mod m3u;
#[cfg(feature = "server")]
pub mod mcp;
#[cfg(feature = "server")]
pub mod sysvol;
pub mod model;
pub mod netstream;
pub mod packages;
pub mod pc;
pub mod player;
pub mod queue;
pub mod radio;
pub mod scanner;
pub mod sources;

pub use library::{Library, ScanResult, Sort, TrackQuery};
pub use model::*;
pub use player::{Command, Player};
pub use effects::{EffectInfo, EffectPlugin, PluginHost};
