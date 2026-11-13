# Aufbau des Codes

Amber Player ist ein Rust-Workspace mit drei Crates. Dieselbe Oberfläche und derselbe Kern
laufen am PC und auf Android.

```
crates/
├── core/            amber-core – alles ohne Fenster
├── app/             amber-player – die App (Tauri), PC und Android
└── mobile-plugin/   tauri-plugin-amber-mobile – Android-Teil in Kotlin
```

## crates/core – der Kern

| Modul | Aufgabe |
|---|---|
| `library.rs` | Bibliothek in SQLite: Titel, Playlisten, Favoriten, Verlauf, Sender, Einstellungen |
| `scanner.rs` | Musikordner einlesen, Tags lesen (lofty), Titel aufräumen |
| `player.rs` | Audio-Engine im eigenen Thread: Warteschlange abspielen, Befehle, Status-Ereignisse |
| `decoder.rs` | Audio dekodieren (Symphonia, minimp3 als Rückfall für kaputte MP3s) |
| `httpsource.rs` | spulbare HTTP-Quellen, Vorpuffer in eigenem Thread, fragmentiertes MP4 |
| `queue.rs` | Warteschlange mit Zufall und Wiederholen |
| `effects/` | Effekt-Plugins: Rust-Effekte (Equalizer, Limiter …), JSON-Effekte, Schutz-Limiter |
| `analyzer.rs` | Frequenzbänder und Wellenform für die Visualisierungen |
| `radio.rs`, `netstream.rs` | Sendersuche (radio-browser.info) und Radiostreams mit Songtiteln |
| `covers.rs` | Cover-Suche im Internet |
| `sources.rs` | Quellen-Plugins (externe Programme) |
| `pc.rs` | Handy → PC: Kopplung, Anfragen an die PC-API, PC-Titel streamen |
| `api.rs`, `mcp.rs` | HTTP-API und MCP-Server (Feature `server`, nur am PC) |

## crates/app – die App

| Datei | Aufgabe |
|---|---|
| `src/lib.rs` | Start der App für PC und Android: Datenordner, Player, Plugins, API |
| `src/main.rs` | Einstieg am PC: Kommandozeile (`--headless`, `--help`) |
| `src/commands.rs` | Befehle, die die Oberfläche aufruft (`invoke`) |
| `src/background.rs` | PC: Systemleisten-Symbol, Beenden ohne Musik |
| `src/media.rs` | PC: Medientasten und MPRIS |
| `ui/index.html`, `ui/style.css`, `ui/app.js` | die Oberfläche |
| `ui/i18n.js`, `ui/lang/` | Übersetzungen, siehe [Sprachen und Übersetzen](Sprachen-und-Übersetzen.md) |
| `ui/viz-worker.js`, `ui/visualizers/` | Visualisierungen im Web Worker |
| `gen/android/` | Android-Projekt (Gradle), u. a. `MainActivity.kt` (Vollbild) |

PC-only-Code ist mit `#[cfg(desktop)]` markiert, Android-Code mit `#[cfg(mobile)]` bzw.
`#[cfg(target_os = "android")]`.

## crates/mobile-plugin – Android

Ein Tauri-Plugin mit Kotlin-Teil:

- `AmberMobilePlugin.kt` – Befehle für die Oberfläche: Benachrichtigung aktualisieren,
  Berechtigungen, Akkuoptimierung, Vollbild, Kopplungs-Link aus dem QR-Code.
- `PlaybackService.kt` – Vordergrunddienst mit MediaSession: Benachrichtigung, Sperrbildschirm,
  Medientasten, Audio-Fokus.

Die Oberfläche spricht es über `invoke('plugin:amber-mobile|…')` an; Tastendrücke kommen als
Ereignis `media` zurück.

## Wie ein Titel abgespielt wird

1. Die Oberfläche ruft `play_ids` auf → `commands.rs` schickt `Command::PlayList` an den Player.
2. Die Engine (`player.rs`) holt den Titel aus der Bibliothek und öffnet ihn:
   - lokale Datei → `decoder.rs` (am Handy mit Vorpuffer),
   - `plugin://pc/…` → Stream vom gekoppelten PC,
   - `plugin://<quelle>/…` → Adresse vom Quellen-Plugin.
3. Der Ton läuft durch die **Effektkette** (`effects/`), den **Analyzer** und den
   **Schutz-Limiter** zur Ausgabe (rodio/cpal; am Handy mit größerem Puffer über Oboe).
4. Die Engine meldet den Status; die App reicht ihn als Ereignis `status` an die Oberfläche.

## Daten

| | Ort |
|---|---|
| Bibliothek und Einstellungen | `~/.local/share/amber-player/library.db` (Android: privater App-Ordner) |
| Plugins | `~/.local/share/amber-player/{effects,visualizers,sources}/` |
| geladene Cover | `~/.local/share/amber-player/covers/` |

## Tests

```sh
cargo test --workspace
```

Getestet werden u. a. Warteschlange, Bibliothek, JSON-Effekte und Formeln, Decoder, HTTP-Quellen,
API-Bereiche und die PC-Kopplung.
