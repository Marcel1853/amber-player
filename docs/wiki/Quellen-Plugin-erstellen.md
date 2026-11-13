# Quellen-Plugin erstellen

Ein Quellen-Plugin bindet Musik aus einem Online-Dienst ein: Es **sucht**, liest **Playlisten**
und liefert **abspielbare Adressen**. Amber Player streamt die Musik – gespeichert wird nichts.
Quellen-Plugins gibt es nur am PC.

> **Achtung:** Ein Quellen-Plugin ist ein Programm und läuft mit deinen Rechten. Beachte außerdem
> die Nutzungsbedingungen des Dienstes, den du einbindest.

## Aufbau

Jedes Plugin ist ein Ordner in `~/.local/share/amber-player/sources/`:

```
sources/
└── meine-quelle/
    ├── plugin.json     ← Beschreibung
    └── plugin.py       ← das Programm (beliebige Sprache)
```

### plugin.json

```json
{
  "id": "meine-quelle",
  "name": "Meine Quelle",
  "description": "Freie Musik aus meinem Archiv",
  "icon": "♪",
  "command": ["python3", "plugin.py"],
  "features": ["search", "playlist"]
}
```

| Feld | Bedeutung |
|---|---|
| `id` | eindeutige Kennung (Buchstaben, Ziffern, `-`, `_`) |
| `name`, `description`, `icon` | Anzeige im Reiter **Online** |
| `command` | Programmaufruf, gestartet im Plugin-Ordner |
| `features` | `search` (Suche) und/oder `playlist` (Playlist-Link übernehmen) |

## Das Protokoll

Für **jede Anfrage** startet Amber Player das Programm, schreibt **eine Zeile JSON** auf die
Standardeingabe und liest **eine Zeile JSON** als Antwort von der Standardausgabe. Andere
Ausgaben (Protokoll, Fehler) gehören auf die Standardfehlerausgabe.

| Anfrage | Antwort |
|---|---|
| `{"action":"search","query":"…","limit":25}` | `{"items":[Eintrag, …]}` |
| `{"action":"playlist","url":"…"}` | `{"name":"…","items":[Eintrag, …]}` |
| `{"action":"resolve","id":"…"}` | `{"url":"https://…","headers":{"…":"…"}}` |
| bei einem Fehler | `{"error":"verständliche Meldung"}` |

Ein **Eintrag** sieht so aus:

```json
{ "id": "abc123", "title": "Titel", "artist": "Interpret", "duration_ms": 215000, "thumbnail": "https://…/bild.jpg" }
```

- `id` ist deine eigene Kennung des Titels; Amber Player gibt sie bei `resolve` zurück.
- `resolve` wird **bei jedem Abspielen** neu gefragt – Adressen bei Online-Diensten laufen oft ab.
- Die Adresse sollte **Range-Anfragen** erlauben, sonst geht Spulen nicht.
- Am besten liefern: AAC (MP4/M4A), MP3, FLAC oder Ogg Vorbis.
- `headers` ist optional (z. B. für einen `User-Agent`, den der Dienst verlangt).

## Beispiel in Python

Ein vollständiges, kleines Plugin, das Titel aus einer eigenen Liste anbietet:

```python
#!/usr/bin/env python3
"""Beispiel-Quelle für Amber Player: Titel aus einer festen Liste."""
import json
import sys

TITEL = [
    {"id": "1", "title": "Morgenlicht", "artist": "Beispiel", "duration_ms": 180000,
     "thumbnail": "", "url": "https://example.org/musik/morgenlicht.mp3"},
    {"id": "2", "title": "Abendrot", "artist": "Beispiel", "duration_ms": 240000,
     "thumbnail": "", "url": "https://example.org/musik/abendrot.mp3"},
]

def eintrag(t):
    return {k: t[k] for k in ("id", "title", "artist", "duration_ms", "thumbnail")}

def main():
    anfrage = json.loads(sys.stdin.readline())
    aktion = anfrage.get("action")
    if aktion == "search":
        q = anfrage.get("query", "").lower()
        treffer = [eintrag(t) for t in TITEL if q in (t["title"] + " " + t["artist"]).lower()]
        antwort = {"items": treffer[: anfrage.get("limit", 25)]}
    elif aktion == "playlist":
        antwort = {"name": "Alle Titel", "items": [eintrag(t) for t in TITEL]}
    elif aktion == "resolve":
        t = next((t for t in TITEL if t["id"] == anfrage.get("id")), None)
        antwort = {"url": t["url"], "headers": {}} if t else {"error": "Titel nicht gefunden"}
    else:
        antwort = {"error": f"Unbekannte Anfrage: {aktion}"}
    print(json.dumps(antwort))

if __name__ == "__main__":
    main()
```

## Testen ohne Amber Player

Du kannst das Plugin direkt im Terminal ausprobieren:

```sh
cd ~/.local/share/amber-player/sources/meine-quelle
echo '{"action":"search","query":"abend","limit":5}' | python3 plugin.py
echo '{"action":"resolve","id":"2"}' | python3 plugin.py
```

## Wie es sich im Player verhält

- Nach dem Neustart erscheint der Reiter **Online** mit Suche und – bei `playlist` – dem Knopf
  „Playlist übernehmen“.
- Online-Titel verhalten sich wie eigene: Warteschlange, Favoriten, Playlisten, Effekte und
  Visualisierung funktionieren.
- In Titel/Alben/Interpreten tauchen sie erst auf, wenn sie benutzt wurden (gespielt, Favorit
  oder in einer Playlist). Nur gesuchte Titel räumt Amber Player nach einem Tag wieder weg.
- Über die API: `/api/online/search`, `/api/online/play`, `/api/online/import`.
