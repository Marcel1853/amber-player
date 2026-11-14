# Remote control and API

Amber Player on the PC has a built-in **HTTP API**. Use it to control the player from your own
programs, scripts, Home Assistant, a Stream Deck or the browser.

## Basics

- Address: `http://127.0.0.1:7878` – the API starts automatically with Amber Player.
- **Every action works with GET and POST.** A simple “open URL” button is enough.
- Actions answer with the current **status** as JSON.
- Errors: HTTP 4xx with `{"error": "…"}`.
- All addresses to copy are under **Settings → Remote control & MCP**.

First tries in a terminal:

```sh
curl http://127.0.0.1:7878/api/status
curl http://127.0.0.1:7878/api/toggle
curl "http://127.0.0.1:7878/api/volume?value=40"
curl "http://127.0.0.1:7878/api/play/music?favorites=true&shuffle=true"
```

## From your home network

By default the API is only reachable from the PC itself. For other devices:

1. Turn on **Settings → Remote control → Home network**.
2. Amber Player restarts once and creates a **token**.
3. Every request from another device needs the token:
   - appended as `?token=…`, or
   - as the header `Authorization: Bearer …`.

```sh
curl "http://192.168.178.35:7878/api/next?token=YOUR_TOKEN"
```

Programs on the PC itself don't need a token. Requests from foreign websites are rejected. If
the **ufw** firewall is active, Amber Player asks for permission to open the port via a
password window.

## The most important commands

| Address | Effect |
|---|---|
| `/api/status` | what's playing |
| `/api/toggle` · `/api/play` · `/api/pause` · `/api/stop` | playback |
| `/api/next` · `/api/prev` | next / previous track |
| `/api/volume?value=0..100` · `/api/volume/up` · `/api/volume/down` | volume |
| `/api/mute` | mute on/off |
| `/api/seek?seconds=90` · `/api/seek/forward` · `/api/seek/back` | seek |
| `/api/shuffle` · `/api/repeat` | toggle shuffle / repeat |
| `/api/sleep?minutes=30` | sleep timer |
| `/api/favorite` | favourite the current track |
| `/api/search?q=…` | search tracks |
| `/api/play/music?artist=&album=&genre=&favorites=true&shuffle=true` | play a selection |
| `/api/playlists` · `/api/playlists/{id}/play` | playlists |
| `/api/radio/play/{id}` | play a saved station |
| `/api/effects/equalizer?preset=Rock` | load an effect preset |
| `/api/system/volume?value=40` · `?muted=true` | the PC's main volume (PipeWire/PulseAudio); without a value: query |
| `/api/plugins` · `/api/plugins/{id}/files` | installed plugin packages (with platforms) or all files of one package |
| `/api/cover/current` | cover of the current track as an image |

**All endpoints with parameters and response format:** [`docs/API.md`](https://github.com/Marcel1853/amber-player/blob/main/docs/API.md) (German)

## Examples

**Bash script “Good night”:**

```sh
#!/bin/sh
API=http://127.0.0.1:7878/api
curl -s "$API/playlists/3/play?shuffle=true" > /dev/null
curl -s "$API/volume?value=25" > /dev/null
curl -s "$API/sleep?minutes=30" > /dev/null
```

**Home Assistant** (`configuration.yaml`):

```yaml
rest_command:
  amber_toggle:
    url: "http://192.168.178.35:7878/api/toggle?token=YOUR_TOKEN"
    method: POST
  amber_favourites:
    url: "http://192.168.178.35:7878/api/play/music?favorites=true&shuffle=true&token=YOUR_TOKEN"
    method: POST
```

**Python:**

```python
import json, urllib.request
status = json.load(urllib.request.urlopen("http://127.0.0.1:7878/api/status"))
print(status["track"]["title"] if status["track"] else "Nothing playing")
```

## Without a window

On a server or Raspberry Pi, Amber Player also runs without an interface – API only:

```sh
amber-player --headless --music ~/Music --lan
```

See also: [Stream Deck and TouchDeck](Stream-Deck-and-TouchDeck.md) · [MCP and AI assistants](MCP-and-AI.md) · [Connecting to your PC](Connecting-to-your-PC.md)
