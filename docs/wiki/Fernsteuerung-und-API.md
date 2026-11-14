# Fernsteuerung und API

Amber Player am PC hat eine eingebaute **HTTP-API**. Damit steuerst du ihn aus eigenen
Programmen, Skripten, Home Assistant, einem Stream Deck oder dem Browser.

## Grundlagen

- Adresse: `http://127.0.0.1:7878` – die API startet automatisch mit Amber Player.
- **Jede Aktion geht per GET und POST.** Eine einfache „URL aufrufen“-Taste reicht.
- Aktionen antworten mit dem aktuellen **Status** als JSON.
- Fehler: HTTP 4xx mit `{"error": "…"}`.
- Alle Adressen zum Kopieren stehen unter **Einstellungen → Fernsteuerung & MCP**.

Erste Versuche im Terminal:

```sh
curl http://127.0.0.1:7878/api/status
curl http://127.0.0.1:7878/api/toggle
curl "http://127.0.0.1:7878/api/volume?value=40"
curl "http://127.0.0.1:7878/api/play/music?favorites=true&shuffle=true"
```

## Aus dem Heimnetz

Standardmäßig ist die API nur auf dem eigenen PC erreichbar. Für andere Geräte:

1. **Einstellungen → Fernsteuerung → Im Heimnetz** einschalten.
2. Amber Player startet einmal neu und erzeugt ein **Token**.
3. Jede Anfrage von einem anderen Gerät braucht das Token:
   - als Zusatz `?token=…` an der Adresse, oder
   - als Header `Authorization: Bearer …`.

```sh
curl "http://192.168.178.35:7878/api/next?token=DEIN_TOKEN"
```

Programme auf dem PC selbst brauchen kein Token. Anfragen fremder Webseiten werden abgewiesen.
Ist die Firewall **ufw** aktiv, fragt Amber Player per Passwortfenster nach der Freigabe des
Ports fürs Heimnetz.

## Die wichtigsten Befehle

| Adresse | Wirkung |
|---|---|
| `/api/status` | was gerade läuft |
| `/api/toggle` · `/api/play` · `/api/pause` · `/api/stop` | Wiedergabe |
| `/api/next` · `/api/prev` | nächster / voriger Titel |
| `/api/volume?value=0..100` · `/api/volume/up` · `/api/volume/down` | Lautstärke |
| `/api/mute` | Stumm an/aus |
| `/api/seek?seconds=90` · `/api/seek/forward` · `/api/seek/back` | springen |
| `/api/shuffle` · `/api/repeat` | Zufall / Wiederholen umschalten |
| `/api/sleep?minutes=30` | Sleep-Timer |
| `/api/favorite` | laufenden Titel als Favorit |
| `/api/search?q=…` | Titel suchen |
| `/api/play/music?artist=&album=&genre=&favorites=true&shuffle=true` | Auswahl spielen |
| `/api/playlists` · `/api/playlists/{id}/play` | Playlisten |
| `/api/radio/play/{id}` | gespeicherten Sender spielen |
| `/api/effects/equalizer?preset=Rock` | Effekt-Preset laden |
| `/api/system/volume?value=40` · `?muted=true` | Hauptlautstärke des PCs (PipeWire/PulseAudio), ohne Wert: abfragen |
| `/api/plugins` · `/api/plugins/{id}/files` | installierte Plugin-Pakete (mit Plattformen) bzw. alle Dateien eines Pakets |
| `/api/cover/current` | Cover des laufenden Titels als Bild |

**Alle Endpunkte mit Parametern und Antwortformat:** [`docs/API.md`](https://github.com/Marcel1853/amber-player/blob/main/docs/API.md)

## Beispiele

**Bash-Skript „Gute Nacht“:**

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
    url: "http://192.168.178.35:7878/api/toggle?token=DEIN_TOKEN"
    method: POST
  amber_favoriten:
    url: "http://192.168.178.35:7878/api/play/music?favorites=true&shuffle=true&token=DEIN_TOKEN"
    method: POST
```

**Python:**

```python
import json, urllib.request
status = json.load(urllib.request.urlopen("http://127.0.0.1:7878/api/status"))
print(status["track"]["title"] if status["track"] else "Nichts läuft")
```

## Ohne Fenster

Auf einem Server oder Raspberry Pi läuft Amber Player auch ganz ohne Oberfläche – nur mit API:

```sh
amber-player --headless --music ~/Musik --lan
```

Siehe auch: [Stream Deck und TouchDeck](Stream-Deck-und-TouchDeck.md) · [MCP und KI-Assistenten](MCP-und-KI.md) · [Mit dem PC verbinden](Mit-dem-PC-verbinden.md)
