# Stream Deck und TouchDeck

## Elgato Stream Deck

Jede Taste ruft einfach eine Adresse der [API](Fernsteuerung-und-API.md) auf.

1. Ein Plugin installieren, das Webadressen aufruft – z. B. **API Ninja** oder die eingebaute
   Aktion **Website** mit „Im Hintergrund öffnen“.
2. Als Adresse eintragen, z. B. `http://127.0.0.1:7878/api/toggle`.

Fertige Adressen stehen unter **Einstellungen → Fernsteuerung & MCP → Beispiele für
Stream-Deck-Tasten** zum Kopieren:

| Taste | Adresse |
|---|---|
| Play / Pause | `/api/toggle` |
| Nächster / Voriger Titel | `/api/next` · `/api/prev` |
| Lauter / Leiser | `/api/volume/up` · `/api/volume/down` |
| Lautstärke 30 % | `/api/volume?value=30` |
| Stumm | `/api/mute` |
| Zufall | `/api/shuffle` |
| Favorit (laufender Titel) | `/api/favorite` |
| Favoriten zufällig | `/api/play/music?favorites=true&shuffle=true` |
| Equalizer an/aus | `/api/effects/equalizer/toggle` |
| EQ-Preset „Bass“ | `/api/effects/equalizer?preset=Bass` |

**Cover als Tastenbild:** `http://127.0.0.1:7878/api/cover/current` liefert das Cover des
laufenden Titels als Bild.

Steht das Stream Deck an einem anderen PC: „Im Heimnetz“ einschalten und `?token=…` an jede
Adresse hängen (steht in den Beispielen dann schon drin).

## TouchDeck

Für TouchDeck gibt es ein fertiges Plugin mit einer ganzen
Player-Seite: Titel, Fortschritt und Lautstärke erscheinen **live** auf den Buttons.

- Plugin und Anleitung: [`integrations/touchdeck`](https://github.com/Marcel1853/amber-player/tree/main/integrations/touchdeck)
- Fertige Seiten auf Deutsch und Englisch zum Importieren

**Einrichtung:**

1. Amber Player starten.
2. Plugin in TouchDeck installieren, unter **Plugins → Installiert** die Adresse prüfen
   (Standard `http://127.0.0.1:7878`).
3. Fertige Seite „Amber Player“ importieren.

Läuft Amber Player auf einem anderen PC, im Plugin dessen Adresse und das Token eintragen.

**Was das Plugin kann:** Wiedergabe, Titel springen, Lautstärke, Stumm, Spulen, Zufall,
Wiederholen, Favorit, Playlisten abspielen, aus der Bibliothek abspielen (nach Interpret, Album,
Genre, Suchbegriff, Favoriten), Klangeffekte schalten und Presets laden, Sleep-Timer. Dazu
Auslöser wie „Neuer Titel gestartet“ für eigene Abläufe.
