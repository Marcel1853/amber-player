//! Android-Teil von Amber Player (Kotlin unter `android/`): Hintergrund-Wiedergabe mit
//! Benachrichtigung und Sperrbildschirm, Kopfhörer-/Bluetooth-Tasten, Audio-Fokus und
//! die Abfrage der Musik-Berechtigung. Die Oberfläche spricht ihn direkt an
//! (`plugin:amber-mobile|…`); am PC ist das Plugin leer.
//!
//! [`MediaBridge`] verbindet die Medientasten und den Sperrbildschirm direkt mit dem Player –
//! ohne Umweg über die Oberfläche. So funktionieren sie auch, wenn Android die Oberfläche im
//! Hintergrund pausiert oder ihren Prozess beendet hat.

use tauri::plugin::{Builder, TauriPlugin};
use tauri::Runtime;
#[cfg(target_os = "android")]
use tauri::Manager;

/// Direkte Verbindung Player ↔ Android (Benachrichtigung, Sperrbildschirm, Medientasten).
#[cfg(target_os = "android")]
pub struct MediaBridge<R: Runtime>(tauri::plugin::PluginHandle<R>);

#[cfg(target_os = "android")]
impl<R: Runtime> MediaBridge<R> {
    /// Was gerade läuft an Benachrichtigung und Sperrbildschirm geben.
    pub fn update(&self, payload: serde_json::Value) {
        if let Err(e) = self.0.run_mobile_plugin::<serde_json::Value>("updateMedia", payload) {
            log::warn!("Sperrbildschirm aktualisieren: {e}");
        }
    }

    /// Gespeicherte Sender (mit Logos) für das Radio-Widget.
    pub fn stations(&self, payload: serde_json::Value) {
        if let Err(e) = self.0.run_mobile_plugin::<serde_json::Value>("updateStations", payload) {
            log::warn!("Radio-Widget aktualisieren: {e}");
        }
    }

    pub fn stop(&self) {
        let _ = self.0.run_mobile_plugin::<serde_json::Value>("stopMedia", serde_json::json!({}));
    }

    /// Medientasten (play, pause, next, prev, stop, seek) und Meldungen (log) direkt empfangen.
    pub fn connect(&self, on_action: impl Fn(&str, i64, &str) + Send + Sync + 'static) -> Result<(), String> {
        let channel = tauri::ipc::Channel::<serde_json::Value>::new(move |body| {
            if let tauri::ipc::InvokeResponseBody::Json(text) = body {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) {
                    on_action(
                        v["action"].as_str().unwrap_or_default(),
                        v["positionMs"].as_i64().unwrap_or(0),
                        v["message"].as_str().unwrap_or_default(),
                    );
                }
            }
            Ok(())
        });
        self.0
            .run_mobile_plugin::<serde_json::Value>("connectMedia", serde_json::json!({ "channel": channel }))
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("amber-mobile")
        .setup(|_app, _api| {
            #[cfg(target_os = "android")]
            {
                let handle = _api.register_android_plugin("de.amberplayer.mobile", "AmberMobilePlugin")?;
                _app.manage(MediaBridge(handle));
            }
            Ok(())
        })
        .build()
}
