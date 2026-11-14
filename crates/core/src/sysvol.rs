//! Lautstärke des ganzen PCs (Linux): PipeWire über `wpctl`, sonst PulseAudio über `pactl`.
//! Amber Player hat daneben seine eigene Lautstärke – die hier ist der Hauptregler des Systems.

use std::process::Command;

use anyhow::{bail, Result};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct SystemVolume {
    /// 0–100 (über 100 wird nicht gesetzt)
    pub volume: u8,
    pub muted: bool,
}

fn run(cmd: &str, args: &[&str]) -> Option<String> {
    let out = Command::new(cmd).args(args).output().ok()?;
    out.status.success().then(|| String::from_utf8_lossy(&out.stdout).into_owned())
}

/// `Volume: 0.45` bzw. `Volume: 0.45 [MUTED]`
fn parse_wpctl(text: &str) -> Option<SystemVolume> {
    let v: f32 = text.split_whitespace().nth(1)?.parse().ok()?;
    Some(SystemVolume { volume: (v * 100.0).round().clamp(0.0, 100.0) as u8, muted: text.contains("MUTED") })
}

/// Erste Prozentangabe aus `pactl get-sink-volume`, z. B. `… 45% …`
fn parse_pactl_percent(text: &str) -> Option<u8> {
    text.split_whitespace()
        .find_map(|w| w.strip_suffix('%')?.parse::<u32>().ok())
        .map(|v| v.min(100) as u8)
}

pub fn get() -> Option<SystemVolume> {
    if let Some(t) = run("wpctl", &["get-volume", "@DEFAULT_AUDIO_SINK@"]) {
        return parse_wpctl(&t);
    }
    let vol = parse_pactl_percent(&run("pactl", &["get-sink-volume", "@DEFAULT_SINK@"])?)?;
    let muted = run("pactl", &["get-sink-mute", "@DEFAULT_SINK@"]).is_some_and(|t| t.contains("yes"));
    Some(SystemVolume { volume: vol, muted })
}

pub fn set(volume: u8) -> Result<SystemVolume> {
    let v = volume.min(100);
    let ok = run("wpctl", &["set-volume", "@DEFAULT_AUDIO_SINK@", &format!("{:.2}", v as f32 / 100.0)]).is_some()
        || run("pactl", &["set-sink-volume", "@DEFAULT_SINK@", &format!("{v}%")]).is_some();
    if !ok {
        bail!("Die Systemlautstärke lässt sich hier nicht einstellen (weder wpctl noch pactl gefunden)");
    }
    get().ok_or_else(|| anyhow::anyhow!("Systemlautstärke nicht lesbar"))
}

pub fn set_muted(muted: bool) -> Result<SystemVolume> {
    let m = if muted { "1" } else { "0" };
    let ok = run("wpctl", &["set-mute", "@DEFAULT_AUDIO_SINK@", m]).is_some()
        || run("pactl", &["set-sink-mute", "@DEFAULT_SINK@", m]).is_some();
    if !ok {
        bail!("Die Systemlautstärke lässt sich hier nicht einstellen (weder wpctl noch pactl gefunden)");
    }
    get().ok_or_else(|| anyhow::anyhow!("Systemlautstärke nicht lesbar"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_tool_output() {
        let v = parse_wpctl("Volume: 0.45\n").unwrap();
        assert_eq!((v.volume, v.muted), (45, false));
        let v = parse_wpctl("Volume: 1.20 [MUTED]\n").unwrap();
        assert_eq!((v.volume, v.muted), (100, true));
        let t = "Volume: front-left: 29491 /  45% / -20.81 dB,   front-right: 29491 /  45% / -20.81 dB";
        assert_eq!(parse_pactl_percent(t), Some(45));
    }
}
