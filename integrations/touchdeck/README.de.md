# Amber Player für TouchDeck

[English](README.md) · **Deutsch**

Steuert den Musik-Player **Amber Player** von deinen TouchDeck-Buttons aus: Play/Pause, Titel
springen, Lautstärke, Zufall, Wiederholen, Favoriten, Playlisten, Equalizer-Presets und
Sleep-Timer. Titel, Fortschritt und Lautstärke erscheinen live auf den Buttons.

Normales Plugin (JavaScript, abgeschottet). Es spricht nur mit Amber Player auf diesem PC
oder im Heimnetz (`lan`), nichts geht ins Internet.

## Einrichtung

1. Amber Player starten. Die API läuft automatisch auf `http://127.0.0.1:7878`.
2. Plugin installieren und unter **Plugins › Installiert** prüfen, dass die Adresse stimmt.
3. Fertige Seite „Amber Player“ importieren (Plugin-Karte › Fertige Seiten).

Läuft Amber Player auf einem anderen PC: dort unter **Einstellungen › Fernsteuerung**
„Im Heimnetz“ einschalten, Amber Player neu starten und im Plugin die Adresse
(z. B. `http://192.168.1.20:7878`) und das Token eintragen.

## Aktionen

| Aktion | Wirkung |
|---|---|
| Abspielen / Pause, Abspielen, Pause, Stopp | Wiedergabe |
| Nächster / Voriger Titel | springen |
| Lautstärke | setzen, lauter, leiser |
| Stumm an/aus | |
| Im Titel springen | 10 s vor/zurück oder an eine Stelle |
| Zufallswiedergabe, Wiederholen | umschalten oder fest setzen |
| Laufender Titel: Favorit an/aus | |
| Playlist abspielen | per Name oder ID, optional zufällig |
| Aus der Bibliothek abspielen | nach Interpret, Album, Genre, Suchbegriff oder nur Favoriten |
| Klangeffekt | Equalizer, Crossfeed, Stereobreite, Limiter: an, aus, umschalten, Preset laden |
| Sleep-Timer | nach X Minuten pausieren, 0 = aus |
| Aktueller Titel (als Daten) | Titel, Interpret, Album, Position, Länge, Lautstärke für folgende Aktionen |

## Live-Anzeigen

Läuft gerade, Titel, Interpret, Album, Fortschritt (Balken), Wiedergabe (an, solange Musik
läuft), Lautstärke (Balken), Zufall, Wiederholen, Favorit, Equalizer und Sleep-Timer.

## Auslöser

- **Neuer Titel gestartet** – mit `{Event.title}`, `{Event.artist}`, `{Event.album}`
- **Wiedergabe gestartet / pausiert / gestoppt** – mit `{Event.state}`

## Fertige Seite

| | | | | |
|---|---|---|---|---|
| 🎵 Läuft gerade (3 breit) | | | ⏱️ Fortschritt (2 breit) | |
| ⏮️ Zurück | ▶️ Play/Pause | ⏭️ Weiter | 🔀 Zufall | 🔁 Wiederholen |
| 🔉 Leiser | 🔇 Stumm | 🔊 Lauter | 🎚️ Lautstärke | ❤️ Favorit |
| ⏪ −10 s | ⏩ +10 s | 🎛️ Equalizer (halten: Preset „Bass“) | ⭐ Favoriten mixen | 🌙 Sleep 30 (halten: aus) |

Neu erzeugen: `python3 tools/make_profile.py`

## Mehr

Alle Befehle der API: [docs/API.md](../../docs/API.md) · Handbuch: [Stream Deck und TouchDeck](../../docs/wiki/Stream-Deck-und-TouchDeck.md)

## Lizenz

MIT
