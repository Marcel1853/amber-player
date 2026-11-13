# Plugins für Amber Player – Nachschlagewerk

Dieses Dokument beschreibt alle Plugin-Schnittstellen vollständig. Schritt-für-Schritt-Anleitungen
für Einsteiger stehen im Wiki: [Plugins erstellen](wiki/Plugins-erstellen.md) ·
[Plugins installieren](wiki/Plugins-installieren.md).

Amber Player lässt sich an drei Stellen erweitern – ohne den Player neu zu bauen:

| Art | Datei | Ordner (Linux) | Wirkt |
|---|---|---|---|
| **Klang-Plugin** | `.json` | `~/.local/share/amber-player/effects/` | nach Neustart |
| **Visualisierung** | `.js` | `~/.local/share/amber-player/visualizers/` | sofort nach „Neu laden“ |
| **Quelle (Online)** | Ordner mit `plugin.json` + Programm | `~/.local/share/amber-player/sources/<name>/` | nach Neustart |

**Am PC** öffnen sich die Ordner direkt aus dem Programm: **Effekte › Plugin-Ordner öffnen** bzw.
**Einstellungen › Plugins**.

**Am Handy (Android)** gibt es keinen Zugriff auf den App-Ordner. Dort heißt der Knopf
**Importieren**: Datei auswählen, Amber Player prüft sie (Klang-Plugins werden vorab geladen) und
legt sie in den Plugin-Ordner der App. Quellen-Plugins gibt es am Handy nicht.

Fertige Vorlagen liegen in [`examples/plugins/`](../examples/plugins).

---

## Klang-Plugins (JSON)

Ein Klang-Plugin beschreibt Regler und eine **Kette aus Bausteinen**. Programmieren muss man
nicht; die Klangberechnung übernimmt der Player selbst (schnell und sicher).

```json
{
  "id": "telefon",
  "name": "Telefon",
  "author": "Dein Name",
  "description": "Klingt wie durchs Telefon",
  "params": [
    { "id": "staerke", "name": "Stärke", "min": 0, "max": 100, "default": 100, "step": 5, "unit": "%" }
  ],
  "presets": [
    { "name": "Halb", "values": [50] }
  ],
  "mix": "staerke / 100",
  "chain": [
    { "type": "highpass", "freq": 400, "q": 0.9 },
    { "type": "lowpass", "freq": 3200, "q": 0.9 },
    { "type": "saturate", "drive": 2.5 },
    { "type": "width", "amount": 0 }
  ]
}
```

| Feld | Pflicht | Bedeutung |
|---|---|---|
| `id` | ja | 1–32 Zeichen: Buchstaben, Ziffern, `-`, `_`; muss eindeutig sein |
| `name` | ja | Anzeigename |
| `author`, `description` | | erscheinen in der Effekt-Liste |
| `params` | | bis 16 Regler: `id`, `name`, `min`, `max`, `default`, optional `step`, `unit` |
| `presets` | | `{ "name", "values": [ … ] }` – Werte in der Reihenfolge der Regler |
| `mix` | | Anteil des bearbeiteten Klangs 0–1 (Standard 1) |
| `chain` | ja | 1–32 Bausteine, werden nacheinander angewendet |

### Bausteine

| `type` | Felder (Standardwert) | Wirkung |
|---|---|---|
| `gain` | `db` (0) | lauter/leiser in dB |
| `peaking` | `freq` (1000), `gain` (0), `q` (1) | Glockenfilter: Frequenzbereich anheben/absenken |
| `lowshelf` | `freq` (200), `gain` (0), `q` (0.707) | Bassregler |
| `highshelf` | `freq` (200), `gain` (0), `q` (0.707) | Höhenregler |
| `lowpass` | `freq` (1000), `q` (0.707) | lässt nur Tiefen durch |
| `highpass` | `freq` (1000), `q` (0.707) | lässt nur Höhen durch |
| `echo` | `time` ms (300), `feedback` 0–0.95 (0.3), `mix` 0–1 (0.3) | Echo |
| `width` | `amount` (1) | Stereobreite: 0 = Mono, 1 = normal, 2 = breit |
| `saturate` | `drive` (1) | weiche Sättigung / Verzerrung |
| `compressor` | `threshold` dB (−18), `ratio` (3), `attack` ms (10), `release` ms (150), `makeup` dB (0) | gleicht Lautstärke an |
| `autopan` | `rate` Hz (0.2), `depth` 0–1 (0.8) | Klang wandert zwischen links und rechts („8D“) |

### Formeln

Jeder Wert darf eine Zahl **oder eine Formel** mit den Regler-IDs sein:

```json
{ "type": "lowpass", "freq": "6000 - alter * 35" }
```

Erlaubt sind `+ - * / ^`, Klammern und die Funktionen `min`, `max`, `clamp(x, a, b)`, `abs`,
`sqrt`, `pow`, `exp`, `log10` sowie `pi`. Formeln werden nur ausgerechnet, wenn sich ein Regler
ändert.

Fehlerhafte Plugins werden übersprungen; die Meldung steht oben auf der Seite **Effekte**.
Die mitgelieferten Plugins (Bass-Booster, Stimmen klarer, Altes Radio, Echo, 8D-Rundflug)
sind im selben Format geschrieben: [`crates/core/src/effects/builtin/`](../crates/core/src/effects/builtin).

---

## Visualisierungen (JavaScript)

Eine Visualisierung ist eine `.js`-Datei, die sich bei `amber` anmeldet und in jedem Bild auf
eine Leinwand zeichnet (Canvas 2D).

```js
amber.visualizer({
  id: 'mein-kreis',
  name: 'Mein Kreis',
  author: 'Dein Name',
  description: 'Ein Kreis, der mit dem Bass wächst',

  setup({ state }) {
    state.r = 0;
  },

  draw({ ctx, width, height, frame, info, state, amber }) {
    ctx.fillStyle = '#000';
    ctx.fillRect(0, 0, width, height);
    state.r = amber.lerp(state.r, frame.bass, 0.2);
    ctx.fillStyle = amber.rgba(info.accent);
    ctx.beginPath();
    ctx.arc(width / 2, height / 2, 40 + state.r * 200, 0, Math.PI * 2);
    ctx.fill();
  },
});
```

### Aufrufe

| Funktion | Wann |
|---|---|
| `setup(scene)` | beim Umschalten auf diese Visualisierung (optional) |
| `resize(scene)` | wenn sich die Fenstergröße ändert (optional) |
| `draw(scene)` | in jedem Bild (Pflicht) |

`scene` enthält:

| Feld | Inhalt |
|---|---|
| `ctx` | 2D-Zeichenkontext; Einheiten in Bildschirmpunkten, Pixeldichte ist schon eingerechnet |
| `width`, `height` | Größe der Fläche |
| `state` | leeres Objekt für eigene Daten, bleibt zwischen den Bildern erhalten |
| `frame.bands` | 64 Frequenzbänder 0–1, tief → hoch |
| `frame.wave` | Wellenform, Werte −1 … 1 |
| `frame.level` | Gesamtpegel 0–1 |
| `frame.bass`, `frame.mid`, `frame.treble` | Mittelwerte der Bereiche 0–1 |
| `frame.time`, `frame.dt` | Zeit und Abstand zum letzten Bild in Sekunden |
| `frame.playing` | läuft gerade Musik? |
| `info.title`, `info.artist`, `info.album` | laufender Titel |
| `info.cover` | Cover als Bild (für `ctx.drawImage`) oder `null` |
| `info.accent`, `info.accent2` | Akzentfarben als `[r, g, b]` |
| `amber` | Hilfen: `rgba(c, a)`, `mix(c1, c2, t, a)`, `lerp`, `clamp`, `range(bands, von, bis)` |

### Sicherheit

Visualisierungen laufen in einem **eigenen, abgeschotteten Hintergrund-Thread**. Sie haben
keinen Zugriff auf Netzwerk, Dateien, die Oberfläche oder die Steuerung des Players – nur auf
ihre Leinwand. Fehler und Endlosschleifen bremsen den Player nicht aus; die Meldung erscheint
unten rechts.

Die mitgelieferten Visualisierungen sind gute Vorlagen:
[`crates/app/ui/visualizers/`](../crates/app/ui/visualizers).

---

## Quellen-Plugins (Musik aus dem Netz)

Ein Quellen-Plugin findet Musik online und liefert abspielbare Adressen – Amber Player streamt sie,
gespeichert wird nichts. Nur am PC. Ein vollständiges Beispiel in Python steht im Wiki:
[Quellen-Plugin erstellen](wiki/Quellen-Plugin-erstellen.md).

> Quellen-Plugins sind Programme und laufen mit den Rechten des Nutzers. Beachte die
> Nutzungsbedingungen des Dienstes, den du einbindest.

Jedes Plugin ist ein Ordner mit `plugin.json` und einem beliebigen Programm:

```json
{
  "id": "meine-quelle",
  "name": "Meine Quelle",
  "description": "Kurzbeschreibung",
  "icon": "♪",
  "command": ["python3", "plugin.py"],
  "features": ["search", "playlist"]
}
```

Für jede Anfrage startet Amber Player das Programm im Plugin-Ordner, schreibt **eine Zeile JSON** auf
stdin und liest die Antwort als **eine Zeile JSON** von stdout (andere Ausgaben auf stderr):

| Anfrage | Antwort |
|---|---|
| `{"action":"search","query":"…","limit":25}` | `{"items":[Eintrag, …]}` |
| `{"action":"playlist","url":"…"}` | `{"name":"…","items":[…]}` |
| `{"action":"resolve","id":"…"}` | `{"url":"https://…","headers":{…}}` |
| Fehler | `{"error":"verständliche Meldung"}` |

Eintrag: `{"id","title","artist","duration_ms","thumbnail"}`. `resolve` wird bei jedem Abspielen
neu gefragt (Adressen laufen oft ab); die Adresse muss Range-Anfragen erlauben, damit Spulen geht.
Am besten AAC (MP4/M4A), MP3, FLAC oder Ogg Vorbis liefern.

Online-Titel verhalten sich wie eigene: Warteschlange, Favoriten, Playlisten, Effekte und
Visualisierung funktionieren. In Titel/Alben/Interpreten tauchen sie erst auf, wenn sie benutzt wurden.

Zum Ausprobieren ohne Player: `echo '{"action":"search","query":"test","limit":5}' | python3 plugin.py`
im Plugin-Ordner ausführen.
