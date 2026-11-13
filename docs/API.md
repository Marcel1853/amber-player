# HTTP-API von Amber Player – Nachschlagewerk

Basis-Adresse: `http://127.0.0.1:7878` (Port in den Einstellungen änderbar). Die API gibt es nur
in Amber Player am PC (auch ohne Fenster mit `--headless`), nicht in der Handy-App – die ist
selbst eine Gegenstelle. Einführung mit Beispielen: [Fernsteuerung und API](wiki/Fernsteuerung-und-API.md).

- **Jede Aktion geht per GET und POST** – so funktionieren auch einfache „URL aufrufen“-Tasten.
- Aktionen antworten mit dem **Status** als JSON (siehe unten).
- Fehler: HTTP 4xx mit `{ "error": "…" }`.
- **Im Heimnetz-Modus** braucht jede Anfrage das Token: `?token=…` an die Adresse hängen oder
  Header `Authorization: Bearer …`.
- Ohne Token werden Anfragen fremder Webseiten (Header `Origin` nicht localhost) abgewiesen.

## Wiedergabe

| Endpunkt | Wirkung |
|---|---|
| `GET /api/status` | aktueller Status |
| `/api/play` · `/api/pause` · `/api/toggle` · `/api/stop` | Wiedergabe |
| `/api/next` · `/api/prev` | nächster / voriger Titel (`prev` nach 3 s: an den Anfang) |
| `/api/seek?seconds=90` | an Stelle springen |
| `/api/seek/forward` · `/api/seek/back` | 10 s vor / zurück |
| `/api/volume?value=0..100` | Lautstärke setzen (ohne `value`: nur Status) |
| `/api/volume/up` · `/api/volume/down` | ±5 |
| `/api/mute` | Stumm an/aus |
| `/api/shuffle` · `/api/shuffle?on=true` | Zufall umschalten / setzen |
| `/api/repeat` · `/api/repeat?mode=off\|all\|one` | Wiederholen weiterschalten / setzen |
| `/api/sleep?minutes=30` · `/api/sleep` | Sleep-Timer setzen / ausschalten |
| `/api/favorite` | laufenden Titel als Favorit an/aus |

## Bibliothek und Warteschlange

| Endpunkt | Wirkung |
|---|---|
| `GET /api/search?q=wald&limit=50` | Titel suchen (Liste von Titeln) |
| `/api/play/track/{id}` | einen Titel spielen |
| `/api/play/music?artist=&album=&genre=&search=&favorites=true&shuffle=true` | Auswahl spielen; ohne Filter: alles |
| `GET /api/queue` | die nächsten Titel |
| `/api/queue/add/{id}` · `/api/queue/next/{id}` | einreihen / als Nächstes |
| `/api/queue/clear` | alles nach dem laufenden Titel entfernen |
| `GET /api/library` | Anzahl Titel/Alben/Interpreten, Ordner, läuft ein Scan |
| `/api/library/scan` | Bibliothek neu einlesen |
| `GET /api/cover/current` · `GET /api/cover/{id}` | Cover als Bild (z. B. als Tastenbild) |

## Playlisten

| Endpunkt | Wirkung |
|---|---|
| `GET /api/playlists` | alle Playlisten |
| `GET /api/playlists/{id}` | Playlist mit Titeln |
| `/api/playlists/{id}/play?shuffle=true` | abspielen |

## Internetradio

| Endpunkt | Wirkung |
|---|---|
| `GET /api/radio` | gespeicherte Sender („Meine Sender“) |
| `GET /api/radio/search?q=1live&country=DE` | Sender suchen (radio-browser.info); ohne `q`: beliebteste |
| `/api/radio/play/{id}` | gespeicherten Sender abspielen |
| `/api/radio/play?url=…&name=…` | beliebige Stream-Adresse abspielen |

Läuft Radio, steht im Status `radio` (Sender und Songtitel) statt `track`; `next`/`prev` wechseln
zwischen den gespeicherten Sendern.

## Streams fürs Handy

Nur im Heimnetz-Modus sinnvoll; Geräte im Netz brauchen das Token (`?token=…`). Die Handy-App
von Amber Player nutzt `/api/search`, `/api/playlists` und `/stream/track/…`, wenn sie mit dem PC
gekoppelt ist ([Mit dem PC verbinden](wiki/Mit-dem-PC-verbinden.md)).

| Endpunkt | Inhalt |
|---|---|
| `GET /stream/all.m3u` | alle Titel als Playlist-Datei |
| `GET /stream/favorites.m3u` | Favoriten |
| `GET /stream/playlist/{id}.m3u` | eine Playlist |
| `GET /stream/track/{id}.mp3` | ein Titel (mit Range-Anfragen zum Spulen) |

Poweramp liest Playlisten nicht direkt aus dem Netz: die `.m3u` herunterladen und mit Poweramp öffnen.
Einzelne Titel- oder Radio-Adressen gehen auch über „Streams › URL hinzufügen“.

## Effekt-Plugins

IDs der mitgelieferten Effekte: `equalizer`, `crossfeed`, `stereo_width`, `limiter` sowie die
JSON-Effekte `bass-booster`, `stimmen-klarer`, `altes-radio`, `echo`, `rundflug`. Eigene
Klang-Plugins sind unter ihrer `id` erreichbar. `GET /api/effects` listet alle.

| Endpunkt | Wirkung |
|---|---|
| `GET /api/effects` | alle Plugins mit Parametern, Werten und Presets |
| `/api/effects/{id}?enabled=true` | an-/ausschalten |
| `/api/effects/{id}?preset=Rock` | Preset laden (schaltet ein) |
| `/api/effects/{id}?values=3,2,0,0,0,0,0,1,2,3` | alle Werte setzen |
| `/api/effects/{id}/toggle` | umschalten |
| `/api/effects/{id}/param/{param}?value=4` | einen Parameter setzen, z. B. `equalizer/param/62` |

Equalizer-Presets: Neutral, Bass, Höhen, Pop, Rock, Elektronisch, Klassik, Stimme.

## Status

```json
{
  "state": "playing",
  "track": {
    "id": 7, "title": "Neonregen", "artist": "Lumen Drift", "album": "Nachtfalter",
    "album_artist": "", "genre": "Electronic", "year": 2023, "track_no": 1, "disc_no": null,
    "duration_ms": 32000, "has_cover": true, "play_count": 3, "favorite": false,
    "path": "/home/…/01 Neonregen.flac"
  },
  "position_ms": 12034,
  "duration_ms": 32000,
  "volume": 70,
  "muted": false,
  "shuffle": false,
  "repeat": "off",
  "sleep_remaining_s": null,
  "queue_length": 12,
  "error": null
}
```

## MCP

`POST /mcp` – Model Context Protocol („Streamable HTTP“, zustandslos).

```sh
claude mcp add --transport http amber-player http://127.0.0.1:7878/mcp
# im Heimnetz-Modus:
claude mcp add --transport http amber-player http://PC:7878/mcp --header "Authorization: Bearer TOKEN"
```

Werkzeuge: `get_status`, `play`, `pause`, `toggle`, `stop`, `next`, `previous`, `set_volume`,
`seek`, `set_shuffle`, `set_repeat`, `set_sleep_timer`, `search_library`, `list_artists`,
`list_albums`, `play_music`, `play_tracks`, `add_to_queue`, `get_queue`, `set_favorite`,
`list_playlists`, `get_playlist`, `create_playlist`, `add_to_playlist`, `play_playlist`,
`list_effects`, `set_effect`.

## Stream Deck

Mit dem Plugin „API Ninja“ oder „Website“ (im Hintergrund öffnen) einfach die Adresse eintragen,
z. B. `http://127.0.0.1:7878/api/toggle`. Für ein Tastenbild mit dem aktuellen Cover:
`http://127.0.0.1:7878/api/cover/current`.
