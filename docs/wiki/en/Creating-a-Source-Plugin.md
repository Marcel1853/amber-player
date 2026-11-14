# Creating a source plugin

A source plugin adds music from an online service: it **searches**, reads **playlists** and
returns **playable addresses**. Amber Player streams the music – nothing is stored.

There are two kinds:

| Kind | Runs on | Language | Network |
|---|---|---|---|
| **Online source** in a plugin package (recommended) | PC and phone | JavaScript | only to approved addresses |
| Program plugin | PC only | any (e.g. Python) | unrestricted |

> **Respect** the terms of use of the service you integrate, and don't bypass access restrictions
> or protection measures.

## Online source (JavaScript)

An online source is an **extended plugin**: it runs sandboxed in its own small JavaScript
environment inside the player – without access to files, programs or the rest of the device. It
may only reach the internet through `amber.http`, only via **https** and only to the addresses
listed in `plugin.json`. It only becomes active once the user agrees under **Settings → Plugins →
Extended plugins**.

### Structure

```
my-source/
├── plugin.json
├── source.js
├── LICENSE
└── README.md
```

```json
{
  "id": "my-source",
  "name": "My source",
  "version": "1.0.0",
  "author": "Your name",
  "license": "MIT",
  "contents": [{ "type": "source", "file": "source.js" }],
  "permissions": { "network": ["api.example.org", "*.cdn.example.org"] }
}
```

`permissions.network` is required: `example.org` allows exactly that address, `*.example.org`
also all subdomains. Redirects are not followed.

### source.js

```js
amber.source({
  id: 'my-source',
  name: 'My source',
  description: 'Free music from my archive',
  icon: '♪',

  search(query, limit) {
    const r = amber.http.get('https://api.example.org/search?q=' + encodeURIComponent(query));
    if (!r.ok) throw new Error('Search failed: ' + r.status);
    return JSON.parse(r.body).tracks.slice(0, limit).map((t) => ({
      id: String(t.id),
      title: t.title,
      artist: t.artist,
      duration_ms: t.seconds * 1000,
      thumbnail: t.image,
    }));
  },

  // optional – only if the service has playlists
  playlist(url) {
    const id = url.split('/').pop();
    const p = JSON.parse(amber.http.get('https://api.example.org/playlists/' + id).body);
    return { name: p.name, items: p.tracks.map((t) => ({ id: String(t.id), title: t.title, artist: t.artist })) };
  },

  resolve(id) {
    const t = JSON.parse(amber.http.get('https://api.example.org/tracks/' + id).body);
    return { url: t.stream_url, headers: {} };
  },
});
```

| Function | Required | Returns |
|---|---|---|
| `search(query, limit)` | no* | list of entries |
| `playlist(url)` | no | `{ name, items: [entries] }` |
| `resolve(id)` | yes | `{ url, headers }` – the playable address |

\* Without `search` the **Online** tab has no search box.

An **entry**: `{ id, title, artist, duration_ms, thumbnail }`. `resolve` is called **every time**
a track is played. The address should allow range requests (otherwise no seeking). Best formats:
AAC (MP4/M4A), MP3, FLAC or Ogg Vorbis.

### Helpers in `amber`

| Call | Effect |
|---|---|
| `amber.http.get(url, headers)` | GET request, response `{ status, ok, headers, body }` |
| `amber.http.post(url, body, headers)` | POST; an object as `body` is sent as JSON |
| `amber.http.request({ method, url, headers, body })` | any request |
| `amber.log(...)` | message to the log |

Calls are **synchronous** – no `await` needed. Limits: 64 MB memory, 10 seconds of computing time
per call (waiting for the network doesn't count), 40 network requests per call, 8 MB per response.

### Testing and sharing

Without starting the player – in Amber Player's source folder:

```sh
cargo run -p amber-core --example source-probe -- my-source search "test"
cargo run -p amber-core --example source-probe -- my-source resolve 123 --decode
```

`--decode` opens the returned address and decodes a piece of it (without sound) – so you see
whether the player can really play it. Pack it with
`python3 scripts/plugin-pack.py my-source/` → `my-source.amberplugin`, see
[Plugin packages](Plugin-Packages.md).

## Program plugin (PC only)

The older kind: a folder in `~/.local/share/amber-player/sources/` with `plugin.json` and any
program. It runs with the user's rights – only install from trusted sources.

```json
{
  "id": "my-source",
  "name": "My source",
  "icon": "♪",
  "command": ["python3", "plugin.py"],
  "features": ["search", "playlist"]
}
```

For every request Amber Player starts the program, writes **one line of JSON** to standard input
and reads **one line of JSON** as the answer:

| Request | Answer |
|---|---|
| `{"action":"search","query":"…","limit":25}` | `{"items":[entry, …]}` |
| `{"action":"playlist","url":"…"}` | `{"name":"…","items":[…]}` |
| `{"action":"resolve","id":"…"}` | `{"url":"https://…","headers":{…}}` |
| on error | `{"error":"understandable message"}` |

Testing: `echo '{"action":"search","query":"test","limit":5}' | python3 plugin.py`

## In the player

- After a restart the **Online** tab appears (on the phone under **More**).
- Online tracks behave like your own: queue, favourites, playlists, effects and visualizer work.
- They only appear in Tracks/Albums/Artists once used; tracks that were only searched are cleaned
  up after a day.
