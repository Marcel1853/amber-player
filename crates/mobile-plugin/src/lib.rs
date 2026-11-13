//! Android-Teil von Amber Player (Kotlin unter `android/`): Hintergrund-Wiedergabe mit
//! Benachrichtigung und Sperrbildschirm, Kopfhörer-/Bluetooth-Tasten, Audio-Fokus und
//! die Abfrage der Musik-Berechtigung. Die Oberfläche spricht ihn direkt an
//! (`plugin:amber-mobile|…`); am PC ist das Plugin leer.

use tauri::plugin::{Builder, TauriPlugin};
use tauri::Runtime;

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("amber-mobile")
        .setup(|_app, _api| {
            #[cfg(target_os = "android")]
            _api.register_android_plugin("de.amberplayer.mobile", "AmberMobilePlugin")?;
            Ok(())
        })
        .build()
}
