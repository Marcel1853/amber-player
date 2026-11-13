# Languages and translation

## Choosing the language

Amber Player is available in **English, German, Spanish, French, Italian, Dutch, Polish and
Turkish**.

- By default the app follows the **system language**. If that isn't available, it uses English.
- To set it: **Settings → Appearance → Language**. The app then reloads briefly.
- On the phone the notification always follows the system language.

Everything Amber Player shows itself is translated – including effect and visualizer names.
Your content is **not** translated: titles, artists, albums, playlist and station names.

This documentation is available in English and German. For other languages, use your browser's
translate function.

## Improving a translation

The translations are tables in the source code:

```
crates/app/i18n/
├── source-1.py     interface (library, playback, playlists …)
├── source-2.py     settings, radio, PC connection, effects, countries
├── source-3.py     remote control and phone streams
└── build.py        generates crates/app/ui/lang/*.js
```

Each line is one text: German first (it is also the key), then English, Spanish, French,
Italian, Dutch, Polish, Turkish:

```python
("Neue Playlist", "New playlist", "Nueva lista", "Nouvelle playlist", "Nuova playlist", "Nieuwe afspeellijst", "Nowa playlista", "Yeni çalma listesi"),
```

1. Change the text in the right column.
2. Run `python3 crates/app/i18n/build.py`. The script reports if placeholders like `{name}` or
   tags like `<b>` are missing in a translation.
3. Commit the changed files under `crates/app/ui/lang/` as well.

**Placeholders** like `{n}`, `{name}`, `{path}` must appear in every translation – wherever they
belong in that language. Count words are listed as `1|Titel` (one) and `n|Titel` (several).

## Adding a new language

1. **Add a column:** append the new translation at the end of every line in all `source-*.py`
   files and add the language code to `CODES` in `build.py` (e.g. `"pt"`).
2. **Register it:** in `crates/app/ui/i18n.js` add the language to `LANGUAGES` (code and name in
   its own language) and the number format to `LOCALE` (e.g. `pt: 'pt-PT'`).
3. **Load it:** in `crates/app/ui/index.html` add `<script src="lang/pt.js"></script>` next to
   the other language files.
4. **Notification (Android):** create `crates/mobile-plugin/android/src/main/res/values-pt/strings.xml`
   following the other languages.
5. Run `python3 crates/app/i18n/build.py` and build the app.

## How it works

- The German text is the key. If a translation is missing, English is used, finally German.
- **Fixed texts** (buttons, headings, hints, `title` and `placeholder`) are translated
  automatically by `i18n.js` as soon as they appear in the document (MutationObserver).
- **Texts with values** go through `tx('… {name} …', { name })` in the code.
- Areas showing library content are excluded from automatic translation, so a song called
  “Pause” doesn't suddenly get a different name.
- **New texts in the code** need a line in one of the tables – otherwise they appear in German in
  every language.
