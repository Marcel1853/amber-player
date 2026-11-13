# Sternenfeld – Visualisierung für Amber Player

Sterne fliegen auf dich zu, der Bass gibt das Tempo vor, die Höhen lassen sie funkeln.
*Stars fly towards you; the bass sets the pace, the treble makes them sparkle.*

| | |
|---|---|
| Art | Visualisierung (`visualizer.js`) |
| Farben | folgen der Akzentfarbe |
| Lizenz | MIT – frei als Vorlage für eigene Plugins |

## Installieren

`sternenfeld.zip` in Amber Player unter **Einstellungen → Plugins → Installieren** auswählen.
Die Visualisierung erscheint sofort als „Sternenfeld ★“. Die ZIP-Datei baust du aus diesem
Ordner mit:

```sh
python3 scripts/plugin-pack.py examples/plugins/sternenfeld
```

## Als Vorlage nutzen

Ordner kopieren, in `plugin.json` und `visualizer.js` die `id` und den `name` ändern, eigene
Zeichnung in `draw` schreiben. Anleitung: [Visualisierung erstellen](../../../docs/wiki/Visualisierung-erstellen.md)
· [Creating a visualizer](../../../docs/wiki/en/Creating-a-Visualizer.md)
