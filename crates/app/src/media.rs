//! Medientasten und Systemsteuerung: MPRIS (Linux), SMTC (Windows), Now Playing (macOS).

use std::sync::Arc;
use std::time::{Duration, Instant};

use amber_core::{Command, PlayState, Player, Status};
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use souvlaki::{
    MediaControlEvent, MediaControls, MediaMetadata, MediaPlayback, MediaPosition, PlatformConfig,
    SeekDirection,
};
use tauri::{Runtime, WebviewWindow};

const SEEK_STEP: Duration = Duration::from_secs(10);

pub fn start<R: Runtime>(player: Arc<Player>, window: &WebviewWindow<R>) {
    // Unter Windows braucht die Systemsteuerung das Fensterhandle.
    let hwnd = window.window_handle().ok().and_then(|h| match h.as_raw() {
        RawWindowHandle::Win32(w) => Some(w.hwnd.get() as usize),
        _ => None,
    });

    std::thread::Builder::new()
        .name("amber-player-media".into())
        .spawn(move || {
            if let Err(e) = run(player, hwnd) {
                log::warn!("Medientasten nicht verfügbar: {e}");
            }
        })
        .ok();
}

fn run(player: Arc<Player>, hwnd: Option<usize>) -> Result<(), String> {
    let config = PlatformConfig {
        dbus_name: "amberplayer",
        display_name: "Amber Player",
        hwnd: hwnd.map(|h| h as *mut std::ffi::c_void),
    };
    let mut controls = MediaControls::new(config).map_err(|e| format!("{e:?}"))?;

    let p = player.clone();
    controls
        .attach(move |event| {
            let cmd = match event {
                MediaControlEvent::Play => Command::Play,
                MediaControlEvent::Pause => Command::Pause,
                MediaControlEvent::Toggle => Command::Toggle,
                MediaControlEvent::Next => Command::Next,
                MediaControlEvent::Previous => Command::Prev,
                MediaControlEvent::Stop => Command::Stop,
                MediaControlEvent::Seek(dir) | MediaControlEvent::SeekBy(dir, _) => {
                    let ms = SEEK_STEP.as_millis() as i64;
                    Command::SeekBy {
                        ms: if dir == SeekDirection::Forward { ms } else { -ms },
                    }
                }
                MediaControlEvent::SetPosition(MediaPosition(pos)) => Command::Seek {
                    ms: pos.as_millis() as u64,
                },
                MediaControlEvent::SetVolume(v) => Command::SetVolume {
                    volume: (v.clamp(0.0, 1.0) * 100.0).round() as u8,
                },
                _ => return,
            };
            let p = p.clone();
            // Nicht im Callback-Thread blockieren.
            std::thread::spawn(move || {
                p.send(cmd);
            });
        })
        .map_err(|e| format!("{e:?}"))?;

    let cover_file = dirs::cache_dir().map(|d| d.join("amber-player").join("cover"));
    let mut rx = player.subscribe();
    let mut last_key = String::new();
    let mut last_state: Option<PlayState> = None;
    let mut last_pos = (0u64, Instant::now());

    update(&mut controls, &player, &player.status(), &cover_file, &mut last_key);
    loop {
        let status: Status = match rx.blocking_recv() {
            Ok(s) => s,
            Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
            Err(_) => break,
        };
        if now_key(&status) != last_key {
            update(&mut controls, &player, &status, &cover_file, &mut last_key);
        }
        // Position nur melden, wenn sie springt; Clients rechnen selbst weiter.
        let expected = if last_state == Some(PlayState::Playing) {
            last_pos.0 + last_pos.1.elapsed().as_millis() as u64
        } else {
            last_pos.0
        };
        let jumped = status.position_ms.abs_diff(expected) > 1500;
        if last_state != Some(status.state) || jumped {
            let progress = Some(MediaPosition(Duration::from_millis(status.position_ms)));
            let playback = match status.state {
                PlayState::Playing => MediaPlayback::Playing { progress },
                PlayState::Paused => MediaPlayback::Paused { progress },
                PlayState::Stopped => MediaPlayback::Stopped,
            };
            let _ = controls.set_playback(playback);
            last_state = Some(status.state);
        }
        last_pos = (status.position_ms, Instant::now());
    }
    Ok(())
}

/// Ändert sich dieser Wert, müssen Titel und Bild neu gemeldet werden.
fn now_key(status: &Status) -> String {
    match (&status.track, &status.radio) {
        (Some(t), _) => format!("t{}:{}", t.id, t.has_cover),
        (None, Some(r)) => format!("r{}:{}", r.station.url, r.title.as_deref().unwrap_or("")),
        _ => String::new(),
    }
}

fn update(
    controls: &mut MediaControls,
    player: &Player,
    status: &Status,
    cover_file: &Option<std::path::PathBuf>,
    last_key: &mut String,
) {
    *last_key = now_key(status);
    if let (None, Some(r)) = (&status.track, &status.radio) {
        let title = r.title.as_deref().unwrap_or(&r.station.name);
        let _ = controls.set_metadata(MediaMetadata {
            title: Some(title),
            artist: Some(&r.station.name),
            album: Some("Internetradio"),
            cover_url: (!r.station.favicon.is_empty()).then_some(r.station.favicon.as_str()),
            duration: None,
        });
        return;
    }
    let Some(t) = &status.track else {
        let _ = controls.set_metadata(MediaMetadata::default());
        return;
    };
    // Cover als Datei ablegen, damit das System es anzeigen kann.
    let cover_url = cover_file.as_ref().and_then(|path| {
        let (data, _) = player.library().cover(t.id).ok().flatten()?;
        let dir = path.parent()?;
        std::fs::create_dir_all(dir).ok()?;
        for old in std::fs::read_dir(dir).ok()?.filter_map(Result::ok) {
            if old.file_name().to_string_lossy().starts_with("cover-") {
                let _ = std::fs::remove_file(old.path());
            }
        }
        // Eigener Dateiname je Titel, damit das System das neue Bild auch wirklich lädt.
        let file = path.with_file_name(format!("cover-{}", t.id));
        std::fs::write(&file, data).ok()?;
        Some(format!("file://{}", file.to_string_lossy()))
    });
    let _ = controls.set_metadata(MediaMetadata {
        title: Some(&t.title),
        artist: (!t.artist.is_empty()).then_some(t.artist.as_str()),
        album: (!t.album.is_empty()).then_some(t.album.as_str()),
        cover_url: cover_url.as_deref(),
        duration: (t.duration_ms > 0).then(|| Duration::from_millis(t.duration_ms)),
    });
}
