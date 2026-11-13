[English](README.md) · **Deutsch**

# Amber Player

**Musik-Player für Linux und Android** – schnell, schön, werbefrei und kostenlos.
Geschrieben in Rust (Tauri 2, rodio), mit eigener Oberfläche im warmen Dunkel-Design.

---

## Was Amber Player kann

**Musik hören**
- **Eigene Bibliothek:** Musikordner wählen – Amber Player liest sie ein und hält sie automatisch
  aktuell. MP3, FLAC, OGG/Opus, WAV, M4A/AAC, AIFF und mehr.
- **Ansichten** für Titel, Alben, Interpreten, Genres, Favoriten, Zuletzt gespielt und
  Meistgespielt, dazu eine Live-Suche.
- **Playlisten** anlegen, sortieren, als M3U importieren und exportieren.
- **Warteschlange** mit „Als Nächstes spielen“, Zufall, Wiederholen, Sleep-Timer. Beim nächsten
  Start geht es an derselben Stelle weiter.
- **Cover** aus der Datei, aus `cover.jpg`/`folder.jpg` oder automatisch aus dem Internet
  (Deezer, MusicBrainz – nur bei sicherem Treffer, abschaltbar).
- **Internetradio:** über 50.000 Sender suchen (radio-browser.info), eigene Sender speichern,
  Songtitel und Senderlogo werden angezeigt.

**Klang und Bild**
- **Klangeffekte als Plugins:** 10-Band-Equalizer mit Presets, Bass-Booster, Stimmen klarer,
  Kopfhörer-Crossfeed, Stereobreite, Echo, Altes Radio, 8D-Rundflug, Vorverstärker & Limiter.
  Ein Schutz-Limiter verhindert Übersteuern und Knacken.
- **Visualisierungen** im Vollbild: Balken, Welle, Ring (mit drehendem Cover) und Nebel.
- **Eigene Plugins** ohne den Player neu zu bauen: Klangeffekte als JSON-Datei (ohne
  Programmieren), Visualisierungen als JavaScript-Datei.

**Handy und PC zusammen**
- **Android-App** mit allen Funktionen zum Hören: Bibliothek, Playlisten, Effekte,
  Visualisierungen, Radio. Hintergrund-Wiedergabe mit Benachrichtigung, Sperrbildschirm und
  Kopfhörer-/Bluetooth-Tasten.
- **Mit dem PC koppeln** (QR-Code scannen): den PC vom Handy aus fernsteuern oder Musik und
  Playlisten vom PC aufs Handy streamen – ohne Dateien zu kopieren.
- **Fernsteuerung** über eine HTTP-API (Stream Deck, TouchDeck, Home Assistant, curl) und einen
  eingebauten **MCP-Server** für KI-Assistenten wie Claude.

**Überall zu Hause**
- **Acht Sprachen:** Deutsch, Englisch, Spanisch, Französisch, Italienisch, Niederländisch,
  Polnisch, Türkisch – automatisch nach Systemsprache oder fest einstellbar.
- **Passt sich an:** PC-Fenster, Tablet hoch und quer, Handy hoch und quer.
- **Design:** dunkel oder hell, Akzentfarbe frei wählbar.
- **Kostenlos und ohne Werbung** – jetzt und in Zukunft.

## Installieren

### Linux (PC)

| Paket | Für | Installieren |
|---|---|---|
| `.deb` | Ubuntu, Debian, Linux Mint, Pop!_OS | Doppelklick oder `sudo apt install "./Amber Player_<Version>_amd64.deb"` |
| `.rpm` | Fedora, openSUSE | `sudo dnf install "./Amber Player-<Version>-1.x86_64.rpm"` |
| `.AppImage` | alle Distributionen | `chmod +x` und starten |

Danach steht Amber Player im Startmenü. Beim ersten Start **Musikordner wählen** – fertig.

### Android

Die APK-Datei aufs Handy laden und öffnen (einmalig „Installation aus dieser Quelle erlauben“
bestätigen). Beim ersten Start fragt die App nach dem Zugriff auf deine Musik und nach
Benachrichtigungen und liest dann den Ordner **Music** ein. Mindestens Android 7, empfohlen ab 10.

Ausführlich: [Installation](docs/wiki/Installation.md) · [Handy-App](docs/wiki/Handy-App.md)

## Schnellstart

| Ich möchte … | So geht’s |
|---|---|
| Musik abspielen | Titel antippen bzw. doppelklicken, oder **Abspielen** / **Zufällig** |
| eine Playlist anlegen | **Playlisten → Neue Playlist**, Titel per Rechtsklick (Handy: lange drücken) hinzufügen |
| den Klang ändern | Regler-Symbol oben → **Effekte** |
| Radio hören | **Radio** → Sender suchen → antippen, mit ★ speichern |
| das Handy mit dem PC koppeln | am PC **Einstellungen → Fernsteuerung → Handy koppeln**, QR-Code scannen |
| die Sprache ändern | **Einstellungen → Darstellung → Sprache** |
| ein eigenes Plugin bauen | [Plugins erstellen](docs/wiki/Plugins-erstellen.md) |
| meine Version wissen | ganz unten in den **Einstellungen** |

## Dokumentation

Das ausführliche Handbuch steht im **Wiki** ([docs/wiki](docs/wiki/Home.md)):

- [Erste Schritte](docs/wiki/Erste-Schritte.md) · [Bedienung und Tastenkürzel](docs/wiki/Bedienung.md)
- [Handy-App](docs/wiki/Handy-App.md) · [Mit dem PC verbinden](docs/wiki/Mit-dem-PC-verbinden.md)
- [Internetradio](docs/wiki/Internetradio.md) · [Klangeffekte](docs/wiki/Klangeffekte.md) · [Visualisierungen](docs/wiki/Visualisierungen.md)
- [Plugins installieren](docs/wiki/Plugins-installieren.md) · [Plugins erstellen](docs/wiki/Plugins-erstellen.md)
- [Fernsteuerung und API](docs/wiki/Fernsteuerung-und-API.md) · [Stream Deck und TouchDeck](docs/wiki/Stream-Deck-und-TouchDeck.md) · [MCP und KI-Assistenten](docs/wiki/MCP-und-KI.md)
- [Sprachen und Übersetzen](docs/wiki/Sprachen-und-Übersetzen.md) · [Selbst bauen](docs/wiki/Selbst-bauen.md) · [Häufige Fragen](docs/wiki/Häufige-Fragen.md)

Nachschlagewerke für Entwickler:

- [docs/PLUGINS.md](docs/PLUGINS.md) – alle Plugin-Schnittstellen im Detail
- [docs/API.md](docs/API.md) – alle Endpunkte der HTTP-API und des MCP-Servers
- [integrations/touchdeck](integrations/touchdeck/README.de.md) – TouchDeck-Plugin mit fertiger Seite

## Selbst bauen (Kurzfassung)

```sh
# Voraussetzungen (Ubuntu/Debian): Rust über https://rustup.rs und
sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev \
                 librsvg2-dev libasound2-dev libdbus-1-dev

cargo test --workspace                     # Tests
cargo run -p amber-player                  # starten (Entwicklung)
cargo install tauri-cli --version "^2" --locked
cd crates/app && cargo tauri build         # .deb, .rpm, AppImage → target/release/bundle/
bash scripts/android-apk.sh                # Android-App → target/android/
```

Alles Weitere, auch für Android: [Selbst bauen](docs/wiki/Selbst-bauen.md).

## Aufbau

```
crates/core/            Herz des Players: Bibliothek (SQLite), Scanner, Warteschlange,
                        Audio-Engine, Effekte, Radio, PC-Verbindung, HTTP-API, MCP-Server
crates/app/             App für PC und Android (Tauri)
  ui/                   Oberfläche (HTML/CSS/JavaScript), Visualisierungen, Sprachdateien
  i18n/                 Übersetzungstabellen → ui/lang/*.js
  gen/android/          Android-Projekt
crates/mobile-plugin/   Android-Teil: Hintergrund-Wiedergabe, Benachrichtigung, Medientasten
examples/plugins/       Vorlagen für eigene Plugins
integrations/touchdeck/ Plugin für TouchDeck
docs/                   Handbuch (wiki/) und Nachschlagewerke
scripts/                Hilfsskripte (Android-App bauen, Wiki hochladen)
```

## Lizenz

**Alle Rechte vorbehalten.** Amber Player ist kostenlos zu benutzen, aber kein Open Source: Der
Quellcode ist einsehbar, darf aber nicht kopiert, verändert oder als eigene Version
weitergegeben werden. Eigene Plugins sind ausdrücklich erwünscht – jedes Plugin-Paket hat
seine eigene Lizenz; die Vorlagen in `examples/plugins/` stehen unter MIT. Details: [LICENSE](LICENSE).

## Datenschutz

Amber Player sammelt keine Daten und zeigt keine Werbung. Ins Internet geht nur, was du
einschaltest: Cover-Suche (Interpret, Titel, Album), Radio-Senderliste, die Radiostreams selbst
und – einmal am Tag, abschaltbar – die Update-Prüfung bei GitHub. Die HTTP-API ist standardmäßig nur auf dem eigenen PC erreichbar.
