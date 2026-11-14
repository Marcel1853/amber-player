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
| `translations` | | Übersetzungen der Texte im Paket, siehe [unten](#übersetzungen) |

\* **Kurzform für ein einzelnes Teil:** statt `contents` einfach `"type": "effect"` (oder
`"visualizer"`) und optional `"main": "effect.json"` angeben.

Jedes Teil ist genau so aufgebaut wie beschrieben in [Klang-Plugin erstellen](Klang-Plugin-erstellen.md)
und [Visualisierung erstellen](Visualisierung-erstellen.md). Die `id` der einzelnen Klangeffekte
und Visualisierungen muss über alle installierten Plugins eindeutig sein.

## Übersetzungen

Amber Player gibt es in acht Sprachen – dein Paket kann mitziehen. Unter `translations` steht pro
Sprache, wie die Texte deines Pakets dort heißen: **Originaltext → Übersetzung**.

```json
"translations": {
  "en": { "Nachtfahrt": "Night drive", "Kassette": "Cassette", "Alter": "Age", "Neu gekauft": "Brand new" },
  "fr": { "Nachtfahrt": "Route de nuit", "Kassette": "Cassette", "Alter": "Âge", "Neu gekauft": "Tout neuf" }
}
```

- Übersetzt werden **Paketname und -beschreibung** sowie **Namen, Beschreibungen, Regler und
  Presets** der Klangeffekte und Visualisierungen.
- Sprachcodes: `de`, `en`, `es`, `fr`, `it`, `nl`, `pl`, `tr`. Fehlt eine Sprache, nimmt Amber
  Player Englisch, sonst den Originaltext.
- Du schreibst dein Paket auf Englisch? Dann übersetze unter `"de"` ins Deutsche.
- Höchstens 64 KB. Ein vollständiges Beispiel ist `examples/plugins/nachtfahrt/plugin.json`.

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

## PC und Handy

Alle Teile eines Pakets – Klangeffekte, Visualisierungen und Online-Quellen in JavaScript –
laufen auf dem **PC und dem Handy**. In den Einstellungen steht das bei jedem Paket dabei. Nur
die älteren Programm-Plugins für Online-Quellen (siehe [Quellen-Plugin erstellen](Quellen-Plugin-erstellen.md))
laufen ausschließlich am PC.

Ist das Handy mit dem PC verbunden, holst du Pakete unter **PC → Plugins** direkt aufs Handy,
siehe [Mit dem PC verbinden](Mit-dem-PC-verbinden.md).

## Teilen

Installierte Pakete gibst du direkt aus Amber Player weiter – ohne Server, auch unterwegs ohne
gemeinsames WLAN:

- **Am Handy:** Einstellungen → Plugins → **Installierte Plugins** → **Teilen**. Es öffnet sich das
  Android-Teilen-Menü: **Quick Share** oder **Bluetooth** (direkt von Handy zu Handy, ohne
  Datenvolumen), Messenger oder Mail.
- **Am PC:** **Exportieren** speichert die `.amberplugin`-Datei, z. B. für einen USB-Stick.
- **Empfangen:** die Datei antippen (oder „Teilen → Amber Player“) – Amber Player zeigt Name,
  Teile, Autor und Lizenz und fragt, ob es installiert werden soll.

Jedes Paket hat **seine eigene Lizenz** – unabhängig von Amber Player. Sie regelt, was andere
damit tun dürfen, z. B. ob sie es verändern oder unter eigenem Namen veröffentlichen dürfen. Vor
dem Installieren zeigt Amber Player sie an.

## Installieren, aktualisieren, entfernen

- **Installieren:** Effekte → **Installieren** oder Einstellungen → Plugins → **Installieren**,
  Datei auswählen. Visualisierungen sind sofort da, Klangeffekte nach dem nächsten Start der
  App.
- **Aktualisieren:** Einstellungen → Plugins → **Installierte Plugins** → **Aktualisieren** und die
  neue Datei wählen – das alte Paket wird samt allen Teilen ersetzt, löschen musst du nichts.
  (Jede Installation eines Pakets mit derselben `id` ersetzt es ebenso.) Am Handy mit PC-Verbindung
  geht es noch einfacher: **PC → Plugins → Aktualisieren**.
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
