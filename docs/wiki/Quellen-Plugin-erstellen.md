# Quellen-Plugin erstellen

Ein Quellen-Plugin bindet Musik aus einem Online-Dienst ein: Es **sucht**, liest **Playlisten**
und liefert **abspielbare Adressen**. Amber Player streamt die Musik – gespeichert wird nichts.

Es gibt zwei Arten:

| Art | Läuft auf | Sprache | Ins Netz |
|---|---|---|---|
| **Online-Quelle** im Plugin-Paket (empfohlen) | PC und Handy | JavaScript | nur zu freigegebenen Adressen |
| Programm-Plugin | nur PC | beliebig (z. B. Python) | uneingeschränkt |

> **Beachte** die Nutzungsbedingungen des Dienstes, den du einbindest, und umgehe keine
> Zugangssperren oder Schutzmaßnahmen.

## Online-Quelle (JavaScript)

Eine Online-Quelle ist ein **erweitertes Plugin**: Sie läuft abgeschottet in einer eigenen kleinen
JavaScript-Umgebung im Player – ohne Zugriff auf Dateien, Programme oder den Rest des Geräts.
Ins Internet darf sie nur über `amber.http`, nur per **https** und nur zu den Adressen, die in der
`plugin.json` stehen. Aktiv wird sie erst, wenn der Nutzer unter **Einstellungen → Plugins →
Erweiterte Plugins** zustimmt.

### Aufbau

```
meine-quelle/
├── plugin.json
├── source.js
├── LICENSE
└── README.md
```

```json
{
  "id": "meine-quelle",
  "name": "Meine Quelle",
  "version": "1.0.0",
  "author": "Dein Name",
  "license": "MIT",
  "contents": [{ "type": "source", "file": "source.js" }],
  "permissions": { "network": ["api.example.org", "*.cdn.example.org"] }
}
```

`permissions.network` ist Pflicht: `example.org` erlaubt genau diese Adresse, `*.example.org`
zusätzlich alle Unteradressen. Weiterleitungen werden nicht verfolgt.

### source.js

```js
amber.source({
  id: 'meine-quelle',
  name: 'Meine Quelle',
  description: 'Freie Musik aus meinem Archiv',
  icon: '♪',

  search(query, limit) {
    const r = amber.http.get('https://api.example.org/search?q=' + encodeURIComponent(query));
    if (!r.ok) throw new Error('Suche fehlgeschlagen: ' + r.status);
    return JSON.parse(r.body).tracks.slice(0, limit).map((t) => ({
      id: String(t.id),
      title: t.title,
      artist: t.artist,
      duration_ms: t.seconds * 1000,
      thumbnail: t.image,
    }));
  },

  // optional – nur wenn der Dienst Playlisten hat
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

| Funktion | Pflicht | Rückgabe |
|---|---|---|
| `search(query, limit)` | nein* | Liste von Einträgen |
| `playlist(url)` | nein | `{ name, items: [Einträge] }` |
| `resolve(id)` | ja | `{ url, headers }` – die abspielbare Adresse |

\* Ohne `search` gibt es im Reiter **Online** kein Suchfeld.

Ein **Eintrag**: `{ id, title, artist, duration_ms, thumbnail }`. `resolve` wird bei **jedem
Abspielen** neu gefragt. Die Adresse sollte Range-Anfragen erlauben (sonst kein Spulen). Am besten
AAC (MP4/M4A), MP3, FLAC oder Ogg Vorbis.

### Hilfen in `amber`

| Aufruf | Wirkung |
|---|---|
| `amber.http.get(url, headers)` | GET-Anfrage, Antwort `{ status, ok, headers, body }` |
| `amber.http.post(url, body, headers)` | POST; ein Objekt als `body` wird als JSON gesendet |
| `amber.http.request({ method, url, headers, body })` | beliebige Anfrage |
| `amber.log(...)` | Meldung ins Protokoll |

Die Aufrufe sind **synchron** – kein `await` nötig. Grenzen: 64 MB Speicher, 10 Sekunden
Rechenzeit je Aufruf (Wartezeit im Netz zählt nicht mit), 40 Netzanfragen je Aufruf, 8 MB je
Antwort.

### Ausprobieren und weitergeben

Ohne den Player zu starten – im Quellcode-Ordner von Amber Player:

```sh
cargo run -p amber-core --example source-probe -- meine-quelle search "test"
cargo run -p amber-core --example source-probe -- meine-quelle resolve 123 --decode
```

`--decode` öffnet die gelieferte Adresse und dekodiert ein Stück (ohne Ton) – so siehst du, ob der
Player sie wirklich abspielen kann. Gepackt wird mit
`python3 scripts/plugin-pack.py meine-quelle/` → `meine-quelle.amberplugin`, siehe
[Plugin-Pakete](Plugin-Pakete.md).

## Programm-Plugin (nur PC)

Die ältere Art: ein Ordner in `~/.local/share/amber-player/sources/` mit `plugin.json` und einem
beliebigen Programm. Es läuft mit den Rechten des Nutzers – nur aus vertrauenswürdigen Quellen
installieren.

```json
{
  "id": "meine-quelle",
  "name": "Meine Quelle",
  "icon": "♪",
  "command": ["python3", "plugin.py"],
  "features": ["search", "playlist"]
}
```

Für jede Anfrage startet Amber Player das Programm, schreibt **eine Zeile JSON** auf die
Standardeingabe und liest **eine Zeile JSON** als Antwort:

| Anfrage | Antwort |
|---|---|
| `{"action":"search","query":"…","limit":25}` | `{"items":[Eintrag, …]}` |
| `{"action":"playlist","url":"…"}` | `{"name":"…","items":[…]}` |
| `{"action":"resolve","id":"…"}` | `{"url":"https://…","headers":{…}}` |
| bei einem Fehler | `{"error":"verständliche Meldung"}` |

Testen: `echo '{"action":"search","query":"test","limit":5}' | python3 plugin.py`

## Im Player

- Nach dem Neustart erscheint der Reiter **Online** (am Handy unter **Mehr**).
- Online-Titel verhalten sich wie eigene: Warteschlange, Favoriten, Playlisten, Effekte und
  Visualisierung funktionieren.
- In Titel/Alben/Interpreten tauchen sie erst auf, wenn sie benutzt wurden; nur gesuchte Titel
  räumt Amber Player nach einem Tag wieder weg.
