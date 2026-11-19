//! Fehlerprotokoll zum Einschalten (Einstellungen → Fehlersuche). Ist es an, landen die
//! Meldungen von Amber Player zusätzlich in einer Datei im Datenordner, die man sich in der App
//! ansehen und teilen kann. Die Datei wird bei 1 MB gekürzt (die ältere Hälfte fällt weg).

use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

static ENABLED: AtomicBool = AtomicBool::new(false);
static WRITE: Mutex<()> = Mutex::new(());
const MAX_BYTES: u64 = 1024 * 1024;
pub const SETTING: &str = "debug_log";

pub fn path() -> PathBuf {
    crate::library::Library::data_dir().join("amber-debug.log")
}

pub fn enabled() -> bool {
    ENABLED.load(Ordering::Relaxed)
}

pub fn set_enabled(on: bool) {
    let was = ENABLED.swap(on, Ordering::Relaxed);
    if on && !was {
        record("INFO", "amber", &format!("Protokoll eingeschaltet – Amber Player {}", env!("CARGO_PKG_VERSION")));
    }
}

/// Eine Zeile anhängen (nur wenn eingeschaltet).
pub fn record(level: &str, target: &str, msg: &str) {
    if !enabled() {
        return;
    }
    let _guard = WRITE.lock().unwrap_or_else(|e| e.into_inner());
    let p = path();
    if std::fs::metadata(&p).map(|m| m.len()).unwrap_or(0) > MAX_BYTES {
        // ältere Hälfte verwerfen
        if let Ok(text) = std::fs::read_to_string(&p) {
            let cut = text.len() / 2;
            let start = text[cut..].find('\n').map(|i| cut + i + 1).unwrap_or(cut);
            let _ = std::fs::write(&p, &text[start..]);
        }
    }
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&p) {
        let now = chrono_like_now();
        let _ = writeln!(f, "{now} {level:5} {target}: {msg}");
    }
}

/// Inhalt der Protokolldatei (leer, wenn es keine gibt).
pub fn read() -> String {
    std::fs::read_to_string(path()).unwrap_or_default()
}

pub fn clear() {
    let _guard = WRITE.lock().unwrap_or_else(|e| e.into_inner());
    let _ = std::fs::remove_file(path());
}

/// Ortszeit „TT.MM. HH:MM:SS“ ohne zusätzliche Bibliothek (UTC-Versatz aus der Umgebung ist
/// auf dem Handy nicht zuverlässig – darum UTC mit Kennzeichnung).
fn chrono_like_now() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let (days, rem) = (secs / 86_400, secs % 86_400);
    // Tag/Monat aus Tagen seit 1970 (Gregorianisch)
    let z = days as i64 + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let _ = era;
    format!("{d:02}.{m:02}. {:02}:{:02}:{:02} UTC", rem / 3600, rem % 3600 / 60, rem % 60)
}

#[cfg(test)]
mod tests {
    #[test]
    fn date_format_looks_right() {
        let s = super::chrono_like_now();
        assert_eq!(s.len(), "17.11. 19:24:15 UTC".len(), "{s}");
    }
}
