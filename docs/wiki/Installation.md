# Installation

## Linux (PC)

Amber Player gibt es als drei Pakete. Nimm das, das zu deiner Distribution passt:

| Paket | Für | So installierst du es |
|---|---|---|
| `Amber Player_<Version>_amd64.deb` | Ubuntu, Debian, Linux Mint, Pop!_OS, Zorin | Doppelklick (öffnet die Softwareverwaltung) oder im Terminal: `sudo apt install "./Amber Player_<Version>_amd64.deb"` |
| `Amber Player-<Version>-1.x86_64.rpm` | Fedora, openSUSE, Nobara | `sudo dnf install "./Amber Player-<Version>-1.x86_64.rpm"` |
| `Amber Player_<Version>_amd64.AppImage` | jede Distribution, ohne Installation | `chmod +x "Amber Player_<Version>_amd64.AppImage"` und per Doppelklick starten |

Danach findest du **Amber Player** im Startmenü.

### Aktualisieren

Einfach das neue Paket genauso installieren – Bibliothek, Playlisten und Einstellungen bleiben
erhalten. Sie liegen getrennt vom Programm in `~/.local/share/amber-player/`.

### Entfernen

```sh
sudo apt remove amber-player        # bzw. sudo dnf remove amber-player
rm -rf ~/.local/share/amber-player  # nur, wenn auch Bibliothek und Einstellungen weg sollen
```

### Schließen und Hintergrund

- **Läuft keine Musik**, beendet das Schließen des Fensters Amber Player komplett – es bleibt
  nichts im Hintergrund.
- **Läuft Musik**, spielt sie weiter; ein Symbol in der Systemleiste bietet Öffnen,
  Play/Pause, Weiter und Beenden. Ist die Musik zu Ende oder eine Minute pausiert, beendet sich
  Amber Player von selbst.

## Android (Handy und Tablet)

1. Die Datei `AmberPlayer-<Version>.apk` aufs Gerät laden (z. B. über den Browser oder per
   Kabel).
2. Antippen. Android fragt beim ersten Mal, ob Apps aus dieser Quelle installiert werden dürfen –
   für den Browser bzw. Dateimanager erlauben.
3. **Installieren** tippen.
4. Beim ersten Start fragt die App nach:
   - **Zugriff auf Musik und Audio** – nötig, um deine Musik einzulesen,
   - **Benachrichtigungen** – für die Wiedergabe-Steuerung in der Benachrichtigungsleiste,
   - **Akkuoptimierung ausnehmen** – sonst stoppen manche Hersteller (z. B. Samsung) die Musik
     im Hintergrund.

Voraussetzungen: Android 7 oder neuer, 64-Bit-Gerät (arm64 – das sind praktisch alle Handys und
Tablets der letzten Jahre).

### Aktualisieren

Die neue APK einfach drüber installieren. Bibliothek und Einstellungen bleiben erhalten.

> **Samsung „Dual Messenger“:** Taucht Amber Player zweimal auf, wurde die App auch im
> Dual-Messenger-Profil installiert. Eine der beiden Kopien kannst du einfach deinstallieren.

## Ohne Fenster (Server, Raspberry Pi am Fernseher)

Amber Player läuft auch ohne Oberfläche – dann nur Wiedergabe, HTTP-API und MCP-Server:

```sh
amber-player --headless --music ~/Musik
amber-player --headless --music /mnt/nas/musik --lan --port 7878
```

| Option | Bedeutung |
|---|---|
| `--music <Ordner>` | Musikordner hinzufügen (mehrfach möglich) |
| `--port <Port>` | Port der API (Standard 7878) |
| `--lan` | API im Heimnetz freigeben; das Token wird beim Start angezeigt |

Gesteuert wird dann über die [API](Fernsteuerung-und-API.md), ein [Stream Deck oder TouchDeck](Stream-Deck-und-TouchDeck.md)
oder die [Handy-App](Mit-dem-PC-verbinden.md).

Weiter mit: [Erste Schritte](Erste-Schritte.md)
