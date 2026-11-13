# Plugin-Pakete

Ein Klangeffekt oder eine Visualisierung wird als **Plugin-Paket** weitergegeben: eine
ZIP-Datei mit Beschreibung, dem Plugin selbst und **seiner eigenen Lizenz**. Amber Player
installiert das Paket mit einem Klick – am PC und am Handy.

## Aufbau

```
telefon.zip
├── plugin.json     ← Beschreibung: Name, Art, Version, Autor, Lizenz
├── effect.json     ← das Plugin (bei Visualisierungen: visualizer.js)
├── LICENSE         ← die Lizenz dieses Plugins (Pflicht)
└── README.md       ← optional: Beschreibung, Anleitung
```

Die Dateien dürfen direkt in der ZIP-Datei liegen oder in einem gemeinsamen Ordner
(`telefon/plugin.json` …). Andere Dateien sind nicht erlaubt.

## plugin.json

```json
{
  "id": "telefon",
  "name": "Telefon",
  "type": "effect",
  "version": "1.0.0",
  "author": "Dein Name",
  "description": "Klingt wie durchs Telefon",
  "license": "MIT",
  "main": "effect.json",
  "homepage": "https://…"
}
```

| Feld | Pflicht | Bedeutung |
|---|---|---|
| `id` | ja | 1–32 Zeichen aus Buchstaben, Ziffern, `-`, `_`; bestimmt den Ordnernamen |
| `name` | ja | Anzeigename |
| `type` | ja | `effect` (Klangeffekt) oder `visualizer` (Visualisierung) |
| `version` | | z. B. `1.0.0` – erscheint in den Einstellungen |
| `author` | | dein Name |
| `description` | | kurze Beschreibung |
| `license` | | Kurzname der Lizenz, z. B. `MIT`, `GPL-3.0`, `CC-BY-4.0` oder „Alle Rechte vorbehalten“ |
| `main` | | Plugin-Datei; Standard `effect.json` bzw. `visualizer.js` |
| `homepage` | | Adresse, unter der es das Plugin gibt |

Die eigentliche Plugin-Datei (`effect.json` bzw. `visualizer.js`) ist genau so aufgebaut wie
beschrieben in [Klang-Plugin erstellen](Klang-Plugin-erstellen.md) und
[Visualisierung erstellen](Visualisierung-erstellen.md).

## Lizenz

Jedes Paket braucht eine eigene **LICENSE**-Datei. Welche Lizenz du nimmst, entscheidest du:

- **MIT** – jeder darf das Plugin nutzen, verändern und weitergeben, solange dein Name genannt
  bleibt. Gut für Vorlagen und alles, was andere weiterentwickeln sollen.
- **GPL-3.0** – wie MIT, aber veränderte Versionen müssen ebenfalls offen bleiben.
- **Alle Rechte vorbehalten** – andere dürfen das Plugin benutzen, aber nicht verändern oder
  weitergeben.

In den Einstellungen zeigt Amber Player bei jedem Paket die Lizenz an – und weist darauf hin,
wenn keine Lizenzdatei dabei ist.

## Paket bauen

Am einfachsten mit dem Skript aus dem Quellcode – es prüft vorher alles, was Amber Player beim
Installieren auch prüft:

```sh
python3 scripts/plugin-pack.py mein-plugin/        # → mein-plugin.zip
```

Von Hand geht es mit jedem ZIP-Programm: die Dateien des Ordners auswählen und als ZIP
speichern.

## Installieren, aktualisieren, entfernen

- **Installieren:** Effekte → **Installieren** oder Einstellungen → Plugins → **Installieren**,
  ZIP-Datei auswählen. Klangeffekte wirken nach dem nächsten Start der App, Visualisierungen
  sofort.
- **Aktualisieren:** die neue Version genauso installieren – ein Paket mit derselben `id` wird
  ersetzt.
- **Entfernen:** Einstellungen → Plugins → **Installierte Plugins** → **Entfernen**.

Installiert wird in einen eigenen Ordner: `~/.local/share/amber-player/effects/<id>/` bzw.
`…/visualizers/<id>/` (am Handy im privaten Ordner der App).

## Vorlagen

Im Quellcode unter [`examples/plugins/`](https://github.com/Marcel1853/amber-player/tree/main/examples/plugins)
liegen zwei fertige Pakete (MIT-Lizenz), fertig gepackt hängen sie auch an jedem
[Release](https://github.com/Marcel1853/amber-player/releases):

- `telefon/` – Klangeffekt
- `sternenfeld/` – Visualisierung

> Einzelne `.json`- und `.js`-Dateien ohne Paket funktionieren weiterhin, haben aber keine
> Version, keinen Autor und keine Lizenz.
