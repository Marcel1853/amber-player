# Visualisierung erstellen

Eine Visualisierung ist eine JavaScript-Datei. Sie meldet sich bei `amber` an und zeichnet in
jedem Bild auf eine Leinwand ([Canvas 2D](https://developer.mozilla.org/de/docs/Web/API/CanvasRenderingContext2D)).
Amber Player liefert dir dafür Frequenzbänder, Wellenform, Pegel, Titel, Cover und Akzentfarben.

## Das kleinste Beispiel

```js
amber.visualizer({
  id: 'mein-kreis',
  name: 'Mein Kreis',
  author: 'Dein Name',
  description: 'Ein Kreis, der mit dem Bass wächst',

  draw({ ctx, width, height, frame, info, amber }) {
    ctx.fillStyle = '#000';
    ctx.fillRect(0, 0, width, height);
    ctx.fillStyle = amber.rgba(info.accent);
    ctx.beginPath();
    ctx.arc(width / 2, height / 2, 40 + frame.bass * 200, 0, Math.PI * 2);
    ctx.fill();
  },
});
```

Als `mein-kreis.js` in den Ordner `visualizers` legen (PC) bzw. importieren (Handy), dann
Visualisierung öffnen (**V**) – oben erscheint „Mein Kreis ★“.

## Weicher werden: `state`

Werte springen von Bild zu Bild. Mit `state` (bleibt zwischen den Bildern erhalten) und
`amber.lerp` glättest du sie:

```js
amber.visualizer({
  id: 'mein-kreis',
  name: 'Mein Kreis',

  setup({ state }) {
    state.r = 0;
  },

  draw({ ctx, width, height, frame, info, state, amber }) {
    ctx.fillStyle = 'rgba(0, 0, 0, 0.25)'; // halbdurchsichtig → Nachzieheffekt
    ctx.fillRect(0, 0, width, height);
    state.r = amber.lerp(state.r, frame.bass, 0.15);
    ctx.fillStyle = amber.mix(info.accent, info.accent2, frame.treble);
    ctx.beginPath();
    ctx.arc(width / 2, height / 2, 40 + state.r * 220, 0, Math.PI * 2);
    ctx.fill();
  },
});
```

## Aufrufe

| Funktion | Wann | Pflicht |
|---|---|---|
| `setup(scene)` | beim Umschalten auf diese Visualisierung | nein |
| `resize(scene)` | wenn sich die Fenstergröße ändert | nein |
| `draw(scene)` | in jedem Bild (bis zu 60-mal pro Sekunde) | ja |

## Was in `scene` steckt

| Feld | Inhalt |
|---|---|
| `ctx` | 2D-Zeichenkontext. Einheiten in Bildschirmpunkten, die Pixeldichte ist schon eingerechnet |
| `width`, `height` | Größe der Fläche |
| `state` | Objekt für eigene Daten, bleibt erhalten |
| `frame.bands` | 64 Frequenzbänder von tief nach hoch, je 0–1 |
| `frame.wave` | Wellenform, Werte −1 bis 1 |
| `frame.level` | Gesamtpegel 0–1 |
| `frame.bass`, `frame.mid`, `frame.treble` | Mittelwerte von Bass, Mitten, Höhen (0–1) |
| `frame.time`, `frame.dt` | Laufzeit und Abstand zum letzten Bild in Sekunden |
| `frame.playing` | läuft gerade Musik? |
| `info.title`, `info.artist`, `info.album` | laufender Titel |
| `info.cover` | Cover als Bild für `ctx.drawImage` (beim Radio das Senderlogo) oder `null` |
| `info.accent`, `info.accent2` | Akzentfarben als `[r, g, b]` |

## Hilfen in `amber`

| Funktion | Wirkung |
|---|---|
| `amber.rgba([r, g, b], a)` | Farbe als CSS-Text |
| `amber.mix(c1, c2, t, a)` | zwischen zwei Farben mischen (`t` 0–1) |
| `amber.lerp(a, b, t)` | weich von `a` nach `b` |
| `amber.clamp(x, a, b)` | Wert begrenzen |
| `amber.range(bands, von, bis)` | Mittelwert eines Bereichs der Bänder (0–1 = tief–hoch), z. B. `amber.range(frame.bands, 0.6, 1)` für die Höhen |

## Sicherheit und Grenzen

Visualisierungen laufen in einem **eigenen, abgeschotteten Hintergrund-Thread** (Web Worker):

- kein Netzwerk (`fetch`, WebSocket … sind abgeschaltet), keine Dateien, kein Speicher,
- kein Zugriff auf die Oberfläche oder die Steuerung des Players,
- Fehler und Endlosschleifen bremsen den Player nicht aus; die Meldung erscheint unten rechts.

Die Datei darf höchstens 512 KB groß sein.

## Tipps

- **Halbdurchsichtig übermalen** (`rgba(…, 0.2)`) statt komplett löschen ergibt weiche Spuren.
- **`frame.dt` für Bewegung** nutzen, dann läuft es auf jedem Gerät gleich schnell.
- **`frame.playing`** prüfen und bei Pause ruhiger werden – sieht lebendiger aus.
- **Akzentfarben** verwenden, dann passt die Visualisierung zum Farbschema des Nutzers.
- **Am Handy testen:** dort ist weniger Rechenleistung – lieber 200 Teilchen als 5.000.

## Vorlagen

- [`examples/plugins/sternenfeld/`](https://github.com/Marcel1853/amber-player/tree/main/examples/plugins/sternenfeld) – Sterne fliegen auf dich zu
- [`crates/app/ui/visualizers/`](https://github.com/Marcel1853/amber-player/tree/main/crates/app/ui/visualizers) – Balken, Welle, Ring, Nebel
