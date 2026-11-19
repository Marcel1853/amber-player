//! Handy: Medientasten, Benachrichtigung und Sperrbildschirm direkt am Player – ohne Umweg
//! über die Oberfläche. Android pausiert die Oberfläche im Hintergrund oder beendet ihren
//! Prozess bei Speichermangel; die Musik und ihre Steuerung laufen trotzdem weiter.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use amber_core::model::{PlayState, Status};
use amber_core::player::{Command, Player};
use base64::Engine as _;
use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_amber_mobile::MediaBridge;

/// Senderlogos (Adresse → Base64), einmal geladen.
static LOGOS: Mutex<Option<HashMap<String, String>>> = Mutex::new(None);

/// Logo eines Senders als Base64 (leer, wenn keins da oder nicht ladbar).
fn logo(url: &str) -> String {
    if url.is_empty() || !(url.starts_with("http://") || url.starts_with("https://")) {
        return String::new();
    }
    if let Some(hit) = LOGOS.lock().unwrap().get_or_insert_with(HashMap::new).get(url) {
        return hit.clone();
    }
    let data = (|| -> Option<Vec<u8>> {
        let resp = ureq::get(url).timeout(Duration::from_secs(6)).call().ok()?;
        let mut data = Vec::new();
        std::io::Read::read_to_end(&mut std::io::Read::take(resp.into_reader(), 512 * 1024), &mut data).ok()?;
        (data.len() > 32).then_some(data)
    })();
    let b64 = data.map(|d| base64::engine::general_purpose::STANDARD.encode(d)).unwrap_or_default();
    LOGOS.lock().unwrap().get_or_insert_with(HashMap::new).insert(url.to_string(), b64.clone());
    b64
}

/// Gespeicherte Sender (bis zu 5) ans Radio-Widget geben.
fn push_stations<R: Runtime>(app: &AppHandle<R>, player: &Player) -> String {
    let stations = player.library().stations().unwrap_or_default();
    let sig: String = stations.iter().take(5).map(|s| format!("{}:{};", s.id, s.name)).collect();
    let list: Vec<serde_json::Value> = stations
        .iter()
        .take(5)
        .map(|s| serde_json::json!({ "id": s.id, "name": s.name, "logo": logo(&s.favicon) }))
        .collect();
    app.state::<MediaBridge<R>>().stations(serde_json::json!({ "stations": list }));
    sig
}

pub fn start<R: Runtime>(app: &AppHandle<R>, player: Arc<Player>) {
    let p = player.clone();
    let bridge = app.state::<MediaBridge<R>>();
    let connected = bridge.connect(move |action, pos, message| {
        if action == "log" {
            log::warn!("Android: {message}");
            return;
        }
        log::info!("Medientaste: {action}");
        let cmd = match action {
            "play" => Command::Play,
            "pause" => Command::Pause,
            "toggle" => Command::Toggle,
            "next" => Command::Next,
            "prev" => Command::Prev,
            "stop" => Command::Stop,
            "seek" => Command::Seek { ms: pos.max(0) as u64 },
            // Radio-Widget: gespeicherten Sender abspielen (ID in `pos`)
            "station" => match p.library().station(pos) {
                Ok(Some(station)) => Command::PlayStation { station },
                _ => return,
            },
            _ => return,
        };
        p.send(cmd);
    });
    if let Err(e) = connected {
        log::error!("Medientasten verbinden: {e}");
    }

    // Effekte für andere Apps: bei jeder Änderung an Effekten oder Anhebung die Kette neu
    // vermessen und als Klangkurve, Balance und Hall an Android geben
    {
        let app = app.clone();
        let player = player.clone();
        std::thread::Builder::new()
            .name("effekte-andere-apps".into())
            .spawn(move || {
                let mut seen = 0u64;
                let mut last: Option<amber_core::effects::outside::OutsideProfile> = None;
                loop {
                    let fx = player.effects();
                    let v = fx.settings_version();
                    if v != seen {
                        seen = v;
                        let profile = fx.outside_profile();
                        if last.as_ref() != Some(&profile) {
                            match serde_json::to_value(&profile) {
                                Ok(json) => app.state::<MediaBridge<R>>().global_eq_values(json),
                                Err(e) => log::warn!("Effekte für andere Apps: {e}"),
                            }
                            last = Some(profile);
                        }
                    }
                    std::thread::sleep(Duration::from_millis(500));
                }
            })
            .ok();
    }

    // Benachrichtigung und Sperrbildschirm bei jedem Titel-/Zustandswechsel aktualisieren
    let app = app.clone();
    let mut rx = player.subscribe();
    std::thread::Builder::new()
        .name("sperrbildschirm".into())
        .spawn(move || {
            let mut last_key = String::new();
            let mut last_pos: u64 = 0;
            let mut cover_for: Option<i64> = None;
            let mut cover = String::new();
            // Sender fürs Radio-Widget: gleich beim Start und dann bei Änderungen
            let mut station_sig = push_stations(&app, &player);
            let mut station_check = Instant::now();
            loop {
                let s: Status = match rx.blocking_recv() {
                    Ok(s) => s,
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(_) => break,
                };
                if station_check.elapsed() >= Duration::from_secs(15) {
                    station_check = Instant::now();
                    let sig: String = player.library().stations().unwrap_or_default().iter().take(5).map(|s| format!("{}:{};", s.id, s.name)).collect();
                    if sig != station_sig {
                        station_sig = push_stations(&app, &player);
                    }
                }
                let bridge = app.state::<MediaBridge<R>>();
                if s.track.is_none() && s.radio.is_none() {
                    if !last_key.is_empty() {
                        bridge.stop();
                        last_key.clear();
                    }
                    continue;
                }
                let playing = s.state == PlayState::Playing;
                let key = format!(
                    "{:?}|{:?}|{}|{:?}",
                    s.track.as_ref().map(|t| t.id),
                    s.radio.as_ref().map(|r| (&r.station.url, &r.title)),
                    playing,
                    s.track.as_ref().map(|t| t.favorite)
                );
                // Gesprungen (z. B. gespult)? Dann Position neu melden.
                let jumped = s.position_ms.abs_diff(last_pos) > 5000;
                last_pos = s.position_ms;
                if key == last_key && !jumped {
                    continue;
                }
                last_key = key;
                if let Some(t) = &s.track {
                    if cover_for != Some(t.id) {
                        cover_for = Some(t.id);
                        cover = if t.has_cover {
                            player
                                .library()
                                .cover(t.id)
                                .ok()
                                .flatten()
                                .map(|(data, _)| base64::engine::general_purpose::STANDARD.encode(data))
                                .unwrap_or_default()
                        } else {
                            String::new()
                        };
                    }
                }
                let payload = match (&s.track, &s.radio) {
                    (Some(t), _) => serde_json::json!({
                        "title": t.title,
                        "artist": if t.artist.is_empty() { "Unbekannter Interpret".to_string() } else { t.artist.clone() },
                        "album": t.album,
                        "playing": playing,
                        "positionMs": s.position_ms,
                        "durationMs": s.duration_ms,
                        "cover": cover,
                    }),
                    (None, Some(r)) => serde_json::json!({
                        "title": r.title.clone().unwrap_or_else(|| r.station.name.clone()),
                        "artist": r.station.name,
                        "album": "Internetradio",
                        "playing": playing,
                        "positionMs": s.position_ms,
                        "durationMs": 0,
                        "cover": logo(&r.station.favicon),
                        "radio": true,
                        "stationId": r.station.id,
                        "stationName": r.station.name,
                    }),
                    _ => continue,
                };
                bridge.update(payload);
            }
        })
        .ok();
}
