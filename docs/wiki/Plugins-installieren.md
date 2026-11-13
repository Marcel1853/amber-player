# Plugins installieren

Klangeffekte und Visualisierungen kommen als **Plugin-Paket** – eine ZIP-Datei mit eigener
Lizenz, siehe [Plugin-Pakete](Plugin-Pakete.md). Am schnellsten installierst du sie mit
**Installieren** (Effekte bzw. Einstellungen → Plugins) – am PC und am Handy gleich.

Amber Player kennt drei Arten von Plugins:

| Art | Datei | Wo | Wirkt |
|---|---|---|---|
| **Klangeffekt** | Paket (`.zip`) oder eine `.json`-Datei | PC und Handy | nach Neustart der App |
| **Visualisierung** | Paket (`.zip`) oder eine `.js`-Datei | PC und Handy | sofort (PC: nach „Neu laden“) |
| **Quelle** (Musik aus dem Netz) | ein Ordner mit `plugin.json` und Programm | nur PC | nach Neustart |

> **Sicherheit:** Klangeffekte sind reine Beschreibungen ohne Programmcode. Visualisierungen
> laufen abgeschottet ohne Zugriff auf Netz und Dateien. **Quellen-Plugins dagegen sind
> Programme** mit denselben Rechten wie du – installiere sie nur aus Quellen, denen du vertraust.

## Am PC (Linux)

Die Plugin-Ordner liegen in `~/.local/share/amber-player/`:

```
~/.local/share/amber-player/
├── effects/          ← Klangeffekte (*.json)
├── visualizers/      ← Visualisierungen (*.js)
└── sources/          ← Quellen-Plugins (je ein Unterordner)
```

Am einfachsten öffnest du sie aus dem Programm:

- **Effekte → Plugin-Ordner öffnen**
- **Einstellungen → Plugins → Ordner öffnen** (für jede Art einzeln)

Datei hineinkopieren, dann

- bei **Klangeffekten** Amber Player neu starten,
- bei **Visualisierungen** unter Einstellungen → Plugins **Neu laden** klicken,
- bei **Quellen-Plugins** den ganzen Ordner (z. B. `sources/mein-plugin/`) hineinkopieren und
  neu starten. Es erscheint ein neuer Reiter **Online**.

## Am Handy (Android)

Am Handy gibt es keinen Zugriff auf den App-Ordner, darum:

1. Das Paket (`.zip`) bzw. die `.json`- oder `.js`-Datei aufs Handy laden (Download, Messenger, Kabel …).
2. **Effekte → Installieren** (Klangeffekte) oder **Einstellungen → Plugins → Installieren**.
3. Datei auswählen.

Amber Player prüft Klangeffekte schon beim Import; eine fehlerhafte Datei wird abgelehnt und die
Meldung angezeigt. Klangeffekte wirken nach dem nächsten Start der App, Visualisierungen sofort.

Quellen-Plugins gibt es am Handy nicht.

## Plugins entfernen

Pakete: **Einstellungen → Plugins → Installierte Plugins → Entfernen**. Einzelne Dateien:
wieder löschen und Amber Player neu starten. Am Handy: App-Daten
löschen entfernt alle importierten Plugins (aber auch Bibliothek und Einstellungen).

## Wenn ein Plugin nicht erscheint

- **Klangeffekt:** Oben auf der Seite **Effekte** steht unter „Nicht geladen“, was nicht stimmt –
  z. B. ein Tippfehler im JSON oder eine doppelte `id`.
- **Visualisierung:** Fehler erscheinen unten rechts als Meldung. Die Datei darf höchstens
  512 KB groß sein und muss auf `.js` enden.
- **Quelle:** Fehler stehen auf der Seite **Online**; das Programm muss vom Plugin-Ordner aus
  startbar sein (z. B. `python3` installiert).

## Vorlagen

Im Quellcode liegen Beispiele zum Ausprobieren: [`examples/plugins/`](https://github.com/Marcel1853/amber-player/tree/main/examples/plugins)

- `telefon/` – Klangeffekt „Telefon“ (Paket, MIT-Lizenz)
- `sternenfeld/` – Visualisierung „Sternenfeld“ (Paket, MIT-Lizenz)

Selbst bauen: [Plugins erstellen](Plugins-erstellen.md)
