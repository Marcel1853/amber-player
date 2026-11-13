# MCP und KI-Assistenten

Amber Player am PC hat einen eingebauten **MCP-Server** ([Model Context Protocol](https://modelcontextprotocol.io)).
Damit können KI-Assistenten wie Claude den Player steuern: Musik suchen und abspielen,
Playlisten anlegen, Effekte einstellen.

## Einrichten

Die Adresse steht unter **Einstellungen → Fernsteuerung & MCP** – dort gibt es auch den
fertigen Befehl zum Kopieren.

**Claude Code:**

```sh
claude mcp add --transport http amber-player http://127.0.0.1:7878/mcp
```

**Von einem anderen Gerät** (Heimnetz-Modus, mit Token):

```sh
claude mcp add --transport http amber-player http://192.168.178.35:7878/mcp \
  --header "Authorization: Bearer DEIN_TOKEN"
```

Andere Programme mit MCP-Unterstützung: Transport **Streamable HTTP**, Adresse
`http://127.0.0.1:7878/mcp`.

## Was du sagen kannst

- „Spiel was von Santiano.“
- „Mach den Equalizer auf Rock.“
- „Leg alle Favoriten in eine Playlist ‚Abend‘ und spiel sie zufällig.“
- „Was läuft gerade?“
- „Mach in 30 Minuten aus.“
- „Welche Alben habe ich von Unheilig?“

## Werkzeuge

| Bereich | Werkzeuge |
|---|---|
| Wiedergabe | `get_status`, `play`, `pause`, `toggle`, `stop`, `next`, `previous`, `seek`, `set_volume`, `set_shuffle`, `set_repeat`, `set_sleep_timer` |
| Bibliothek | `search_library`, `list_artists`, `list_albums`, `play_music`, `play_tracks`, `add_to_queue`, `get_queue`, `set_favorite` |
| Playlisten | `list_playlists`, `get_playlist`, `create_playlist`, `add_to_playlist`, `play_playlist` |
| Effekte | `list_effects`, `set_effect` |

Der MCP-Server ist zustandslos und nutzt dieselbe Zugriffskontrolle wie die
[API](Fernsteuerung-und-API.md): ohne Heimnetz-Freigabe nur vom eigenen PC, sonst nur mit Token.
In der Handy-App gibt es keinen MCP-Server.
