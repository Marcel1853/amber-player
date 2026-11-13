# Creating a source plugin

A source plugin adds music from an online service: it **searches**, reads **playlists** and
returns **playable addresses**. Amber Player streams the music – nothing is stored. Source
plugins are available on the PC only.

> **Note:** A source plugin is a program and runs with your rights. Also respect the terms of use
> of the service you integrate.

## Structure

Each plugin is a folder in `~/.local/share/amber-player/sources/`:

```
sources/
└── my-source/
    ├── plugin.json     ← description
    └── plugin.py       ← the program (any language)
```

### plugin.json

```json
{
  "id": "my-source",
  "name": "My source",
  "description": "Free music from my archive",
  "icon": "♪",
  "command": ["python3", "plugin.py"],
  "features": ["search", "playlist"]
}
```

| Field | Meaning |
|---|---|
| `id` | unique identifier (letters, digits, `-`, `_`) |
| `name`, `description`, `icon` | shown in the **Online** tab |
| `command` | program call, started inside the plugin folder |
| `features` | `search` and/or `playlist` (import a playlist link) |

## The protocol

For **every request** Amber Player starts the program, writes **one line of JSON** to standard
input and reads **one line of JSON** from standard output as the answer. Anything else (logs,
errors) goes to standard error.

| Request | Answer |
|---|---|
| `{"action":"search","query":"…","limit":25}` | `{"items":[entry, …]}` |
| `{"action":"playlist","url":"…"}` | `{"name":"…","items":[entry, …]}` |
| `{"action":"resolve","id":"…"}` | `{"url":"https://…","headers":{"…":"…"}}` |
| on error | `{"error":"understandable message"}` |

An **entry** looks like this:

```json
{ "id": "abc123", "title": "Title", "artist": "Artist", "duration_ms": 215000, "thumbnail": "https://…/image.jpg" }
```

- `id` is your own identifier for the track; Amber Player passes it back in `resolve`.
- `resolve` is called **every time a track is played** – addresses of online services often
  expire.
- The address should allow **range requests**, otherwise seeking won't work.
- Best formats: AAC (MP4/M4A), MP3, FLAC or Ogg Vorbis.
- `headers` is optional (e.g. a `User-Agent` the service requires).

## Example in Python

A complete small plugin that offers tracks from its own list:

```python
#!/usr/bin/env python3
"""Example source for Amber Player: tracks from a fixed list."""
import json
import sys

TRACKS = [
    {"id": "1", "title": "Morning light", "artist": "Example", "duration_ms": 180000,
     "thumbnail": "", "url": "https://example.org/music/morning-light.mp3"},
    {"id": "2", "title": "Sunset", "artist": "Example", "duration_ms": 240000,
     "thumbnail": "", "url": "https://example.org/music/sunset.mp3"},
]

def entry(t):
    return {k: t[k] for k in ("id", "title", "artist", "duration_ms", "thumbnail")}

def main():
    request = json.loads(sys.stdin.readline())
    action = request.get("action")
    if action == "search":
        q = request.get("query", "").lower()
        hits = [entry(t) for t in TRACKS if q in (t["title"] + " " + t["artist"]).lower()]
        answer = {"items": hits[: request.get("limit", 25)]}
    elif action == "playlist":
        answer = {"name": "All tracks", "items": [entry(t) for t in TRACKS]}
    elif action == "resolve":
        t = next((t for t in TRACKS if t["id"] == request.get("id")), None)
        answer = {"url": t["url"], "headers": {}} if t else {"error": "Track not found"}
    else:
        answer = {"error": f"Unknown request: {action}"}
    print(json.dumps(answer))

if __name__ == "__main__":
    main()
```

## Testing without Amber Player

You can try the plugin directly in a terminal:

```sh
cd ~/.local/share/amber-player/sources/my-source
echo '{"action":"search","query":"sun","limit":5}' | python3 plugin.py
echo '{"action":"resolve","id":"2"}' | python3 plugin.py
```

## How it behaves in the player

- After a restart the **Online** tab appears with a search box and – with `playlist` – the
  “Import playlist” button.
- Online tracks behave like your own: queue, favourites, playlists, effects and visualizer work.
- They only appear in Tracks/Albums/Artists once used (played, favourited or added to a
  playlist). Tracks that were only searched are cleaned up after a day.
- Via the API: `/api/online/search`, `/api/online/play`, `/api/online/import`.
