# MCP and AI assistants

Amber Player on the PC has a built-in **MCP server** ([Model Context Protocol](https://modelcontextprotocol.io)).
AI assistants like Claude can use it to control the player: search and play music, create
playlists, adjust effects.

## Setup

The address is under **Settings → Remote control & MCP** – including a ready-made command to
copy.

**Claude Code:**

```sh
claude mcp add --transport http amber-player http://127.0.0.1:7878/mcp
```

**From another device** (home network mode, with token):

```sh
claude mcp add --transport http amber-player http://192.168.178.35:7878/mcp \
  --header "Authorization: Bearer YOUR_TOKEN"
```

Other programs with MCP support: transport **Streamable HTTP**, address
`http://127.0.0.1:7878/mcp`.

## What you can say

- “Play something by Santiano.”
- “Set the equalizer to Rock.”
- “Put all my favourites into a playlist called ‘Evening’ and shuffle it.”
- “What's playing right now?”
- “Stop in 30 minutes.”
- “Which albums by Unheilig do I have?”

## Tools

| Area | Tools |
|---|---|
| Playback | `get_status`, `play`, `pause`, `toggle`, `stop`, `next`, `previous`, `seek`, `set_volume`, `set_shuffle`, `set_repeat`, `set_sleep_timer` |
| Library | `search_library`, `list_artists`, `list_albums`, `play_music`, `play_tracks`, `add_to_queue`, `get_queue`, `set_favorite` |
| Playlists | `list_playlists`, `get_playlist`, `create_playlist`, `add_to_playlist`, `play_playlist` |
| Effects | `list_effects`, `set_effect` |

The MCP server is stateless and uses the same access control as the
[API](Remote-Control-and-API.md): without home network access only from the PC itself,
otherwise only with the token. The phone app has no MCP server.
