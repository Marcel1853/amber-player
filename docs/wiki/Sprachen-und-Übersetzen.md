# Sprachen und Übersetzen

## Sprache einstellen

Amber Player gibt es auf **Deutsch, Englisch, Spanisch, Französisch, Italienisch,
Niederländisch, Polnisch und Türkisch**.

- Standardmäßig folgt die App der **Systemsprache**. Gibt es die nicht, nimmt sie Englisch.
- Fest einstellen: **Einstellungen → Darstellung → Sprache**. Die App lädt danach kurz neu.
- Am Handy folgt die Benachrichtigung immer der Systemsprache.

Übersetzt wird alles, was Amber Player selbst anzeigt – auch die Namen der Effekte und
Visualisierungen. **Nicht** übersetzt werden deine Inhalte: Titel, Interpreten, Alben,
Playlisten- und Sendernamen.

## Eine Übersetzung verbessern

Die Übersetzungen stehen als Tabellen im Quellcode:

```
crates/app/i18n/
├── source-1.py     Oberfläche (Bibliothek, Wiedergabe, Playlisten …)
├── source-2.py     Einstellungen, Radio, PC-Verbindung, Effekte, Länder
├── source-3.py     Fernsteuerung und Handy-Streams
└── build.py        erzeugt daraus crates/app/ui/lang/*.js
```

Jede Zeile ist ein Text: zuerst Deutsch (das ist zugleich der Schlüssel), dann Englisch,
Spanisch, Französisch, Italienisch, Niederländisch, Polnisch, Türkisch:

```python
("Neue Playlist", "New playlist", "Nueva lista", "Nouvelle playlist", "Nuova playlist", "Nieuwe afspeellijst", "Nowa playlista", "Yeni çalma listesi"),
```

1. Den Text in der richtigen Spalte ändern.
2. `python3 crates/app/i18n/build.py` ausführen. Das Skript meldet, wenn Platzhalter wie
   `{name}` oder Tags wie `<b>` in einer Übersetzung fehlen.
3. Die geänderten Dateien unter `crates/app/ui/lang/` gehören mit in den Commit.

**Platzhalter** wie `{n}`, `{name}`, `{path}` müssen in jeder Übersetzung vorkommen – an der
Stelle, an der sie in der Sprache hingehören. Zählwörter stehen als `1|Titel` (eins) und
`n|Titel` (mehrere) in der Tabelle.

## Eine neue Sprache hinzufügen

1. **Spalte ergänzen:** In allen `source-*.py` am Ende jeder Zeile die neue Übersetzung
   anhängen und in `build.py` den Sprachcode zu `CODES` hinzufügen (z. B. `"pt"`).
2. **Bekannt machen:** In `crates/app/ui/i18n.js` die Sprache in `LANGUAGES` eintragen (Code und
   Name in der eigenen Sprache) und in `LOCALE` die Zahlenschreibweise (z. B. `pt: 'pt-PT'`).
3. **Einbinden:** In `crates/app/ui/index.html` die Zeile
   `<script src="lang/pt.js"></script>` zu den anderen Sprachdateien setzen.
4. **Benachrichtigung (Android):** `crates/mobile-plugin/android/src/main/res/values-pt/strings.xml`
   nach dem Vorbild der anderen Sprachen anlegen.
5. `python3 crates/app/i18n/build.py` ausführen und die App bauen.

## Wie es technisch funktioniert

- Der deutsche Text ist der Schlüssel. Fehlt eine Übersetzung, gilt Englisch, zuletzt Deutsch.
- **Feste Texte** (Knöpfe, Überschriften, Hinweise, `title` und `placeholder`) übersetzt
  `i18n.js` automatisch, sobald sie im Dokument erscheinen (MutationObserver).
- **Texte mit Werten** laufen im Code über `tx('… {name} …', { name })`.
- Bereiche mit Inhalten aus der Bibliothek sind von der automatischen Übersetzung
  ausgenommen, damit ein Lied namens „Pause“ nicht plötzlich anders heißt.
- **Neue Texte im Code** brauchen eine Zeile in einer der Tabellen – sonst erscheinen sie in
  allen Sprachen auf Deutsch.
