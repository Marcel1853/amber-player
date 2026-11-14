# Plugin-Pakete

Klangeffekte, Visualisierungen und Online-Quellen werden als **Plugin-Paket** weitergegeben: eine Datei mit der
Endung **`.amberplugin`**. Ein Paket kann **einen oder mehrere Teile** enthalten – zum Beispiel
drei Klangeffekte und zwei Visualisierungen – und hat **seine eigene Lizenz**. Amber Player
installiert es mit einem Klick, am PC und am Handy.

Innen ist ein `.amberplugin` eine ganz normale ZIP-Datei; Amber Player nimmt auch Pakete mit der
Endung `.zip` an.

## Aufbau

```
nachtfahrt.amberplugin
├── plugin.json              ← Beschreibung und Liste der Teile
├── effects/
│   ├── kassette.json        ← Klangeffekt
│   └── stadion.json         ← Klangeffekt
├── visualizers/
│   └── polarlicht.js        ← Visualisierung
├── LICENSE                  ← die Lizenz des Pakets (Pflicht)
└── README.md                ← optional: Beschreibung, Anleitung
```

Die Teile dürfen direkt im Paket oder in **einem** Unterordner liegen (z. B. `effects/`). Andere
Dateien als `plugin.json`, die Teile, `LICENSE` und `README` sind nicht erlaubt.

## plugin.json

```json
{
  "id": "nachtfahrt",
  "name": "Nachtfahrt",
  "version": "1.0.0",
  "author": "Dein Name",
  "description": "Zwei Klangeffekte und eine Visualisierung für die Nacht",
  "license": "MIT",
  "homepage": "https://…",
  "contents": [
    { "type": "effect", "file": "effects/kassette.json" },
    { "type": "effect", "file": "effects/stadion.json" },
    { "type": "visualizer", "file": "visualizers/polarlicht.js" }
  ]
}
```

| Feld | Pflicht | Bedeutung |
|---|---|---|
| `id` | ja | 1–32 Zeichen aus Buchstaben, Ziffern, `-`, `_`; bestimmt den Ordnernamen |
| `name` | ja | Anzeigename des Pakets |
| `contents` | ja* | Liste der Teile: `type` (`effect`, `visualizer` oder `source`) und `file` (Pfad im Paket) – bis zu 32 Teile |
| `permissions` | bei `source` | `{ "network": ["api.example.org", "*.example.org"] }` – Adressen, die Online-Quellen erreichen dürfen, siehe [Quellen-Plugin erstellen](Quellen-Plugin-erstellen.md) |
| `version` | | z. B. `1.0.0` – erscheint in den Einstellungen |
| `author` | | dein Name |
| `description` | | kurze Beschreibung |
| `license` | | Kurzname der Lizenz, z. B. `MIT`, `GPL-3.0`, `CC-BY-4.0` oder „Alle Rechte vorbehalten“ |
| `homepage` | | Adresse, unter der es das Paket gibt |

\* **Kurzform für ein einzelnes Teil:** statt `contents` einfach `"type": "effect"` (oder
`"visualizer"`) und optional `"main": "effect.json"` angeben.

Jedes Teil ist genau so aufgebaut wie beschrieben in [Klang-Plugin erstellen](Klang-Plugin-erstellen.md)
und [Visualisierung erstellen](Visualisierung-erstellen.md). Die `id` der einzelnen Klangeffekte
und Visualisierungen muss über alle installierten Plugins eindeutig sein.

## Lizenz

Jedes Paket braucht eine eigene **LICENSE**-Datei. Sie gilt für alle Teile im Paket. Welche
Lizenz du nimmst, entscheidest du:

- **MIT** – jeder darf das Paket nutzen, verändern und weitergeben, solange dein Name genannt
  bleibt. Gut für Vorlagen und alles, was andere weiterentwickeln sollen.
- **GPL-3.0** – wie MIT, aber veränderte Versionen müssen ebenfalls offen bleiben.
- **Alle Rechte vorbehalten** – andere dürfen das Paket benutzen, aber nicht verändern oder
  weitergeben.

In den Einstellungen zeigt Amber Player bei jedem Paket die Lizenz an – und weist darauf hin,
wenn keine Lizenzdatei dabei ist.

## Paket bauen

Am einfachsten mit dem Skript aus dem Quellcode – es prüft vorher alles, was Amber Player beim
Installieren auch prüft, und warnt, wenn Dateien im Ordner liegen, die nicht in `contents`
stehen:

```sh
python3 scripts/plugin-pack.py mein-paket/        # → mein-paket.amberplugin
```

Von Hand geht es mit jedem ZIP-Programm: die Dateien des Ordners als ZIP speichern und die Endung
in `.amberplugin` ändern (oder `.zip` lassen).

## Installieren, aktualisieren, entfernen

- **Installieren:** Effekte → **Installieren** oder Einstellungen → Plugins → **Installieren**,
  Datei auswählen. Visualisierungen sind sofort da, Klangeffekte nach dem nächsten Start der
  App.
- **Aktualisieren:** die neue Version genauso installieren – ein Paket mit derselben `id` wird
  samt allen Teilen ersetzt.
- **Entfernen:** Einstellungen → Plugins → **Installierte Plugins** → **Entfernen**. Alle Teile
  des Pakets verschwinden zusammen.

Installiert wird nach `~/.local/share/amber-player/packages/<id>/` (am Handy in den privaten
Ordner der App).

## Vorlagen

Im Quellcode unter [`examples/plugins/`](https://github.com/Marcel1853/amber-player/tree/main/examples/plugins),
alle mit MIT-Lizenz; fertig gepackt hängen sie an jedem
[Release](https://github.com/Marcel1853/amber-player/releases):

- `nachtfahrt/` – **mehrere Teile:** Klangeffekte „Kassette“ und „Stadion“, Visualisierung „Polarlicht“
- `telefon/` – ein Klangeffekt (Kurzform mit `type`)
- `sternenfeld/` – eine Visualisierung (Kurzform mit `type`)

> Einzelne `.json`- und `.js`-Dateien ohne Paket funktionieren weiterhin, haben aber keine
> Version, keinen Autor und keine Lizenz.
