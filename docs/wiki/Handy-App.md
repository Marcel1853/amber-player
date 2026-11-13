# Handy-App (Android)

Die Android-App ist ein vollwertiger Amber Player: eigene Bibliothek, Playlisten, Effekte,
Visualisierungen und Radio – auch ganz ohne PC. Mit einem gekoppelten PC kommen Fernsteuerung
und Streaming dazu, siehe [Mit dem PC verbinden](Mit-dem-PC-verbinden.md).

Installation: [Installation → Android](Installation.md#android-handy-und-tablet)

## Aufbau

- **Unten:** die Leiste **Titel · Alben · Playlisten · Radio · Mehr**.
- **Darüber:** der Mini-Player mit Cover, Titel, Play/Pause und Weiter.
- **Ein Tipp auf den Mini-Player** öffnet **Läuft gerade**: großes Cover, Fortschritt,
  Steuerung, Lautstärke, Favorit, Sleep-Timer, Visualisierung und Als Nächstes. Mit dem Pfeil
  oben links geht es zurück.
- **Mehr** öffnet: PC, Interpreten, Genres, Favoriten, Verlauf, Visualisierung, Effekte,
  Einstellungen.

**Tippen** spielt einen Titel ab, **lange drücken** öffnet das Menü (Als Nächstes,
Warteschlange, Zu Playlist, Favorit …).

## Im Hintergrund hören

Amber Player spielt weiter, wenn du die App verlässt oder den Bildschirm ausschaltest:

- In der **Benachrichtigung** und auf dem **Sperrbildschirm** siehst du Cover und Titel und hast
  Zurück, Play/Pause und Weiter.
- **Kopfhörer- und Bluetooth-Tasten** funktionieren.
- **Anruf oder Navigationsansage:** die Musik pausiert und läuft danach weiter.
- **Kopfhörer abgezogen:** die Musik pausiert.
- **App aus der Übersicht gewischt:** die Musik stoppt.

> **Musik stoppt nach einer Weile im Hintergrund?** Dann beendet die Akkuoptimierung die App.
> Beim ersten Start fragt Amber Player danach; nachträglich: Android-Einstellungen → Apps →
> Amber Player → Akku → **Nicht eingeschränkt**.

## Vollbild

Standardmäßig blendet die App Status- und Navigationsleiste aus. Wischen vom Rand zeigt sie
kurz an. Abschalten: **Einstellungen → Darstellung → Vollbild**. Ist das Vollbild aus, rückt die
App so ein, dass nichts unter den Leisten verschwindet.

## Musik auf dem Handy

Eingelesen wird der Ordner **Music** im internen Speicher. Lege deine Musik dort ab (gern in
Unterordnern) oder kopiere sie per Kabel hinein. **Einstellungen → Neu einlesen** liest neue
Dateien sofort ein.

## Plugins am Handy

Eigene Klangeffekte (`.json`) und Visualisierungen (`.js`) installierst du über
**Effekte → Installieren** bzw. **Einstellungen → Plugins → Installieren**: Paket (`.zip`) oder
Datei auswählen, fertig. Klang-Plugins wirken nach dem nächsten Start der App, Visualisierungen sofort.
Mehr dazu: [Plugins installieren](Plugins-installieren.md).

## Was es nur am PC gibt

| Funktion | Warum nicht am Handy |
|---|---|
| HTTP-API, MCP-Server, Stream-Deck-Steuerung | Das Handy ist die Fernbedienung, nicht das Ziel. |
| Quellen-Plugins (Online-Musik) | Sie starten Programme, das erlaubt Android nicht. |
| Handy-Streams (`.m3u` für Poweramp) | Dafür gibt es jetzt die direkte PC-Verbindung. |

## Tablets

Hochkant sieht die App aus wie am Handy, „Läuft gerade“ steht mittig. Quer bekommt das Tablet
die Ansicht wie am PC mit Player-Spalte links.

## Sprache

Die App folgt der Systemsprache. Fest einstellen: **Einstellungen → Darstellung → Sprache**.
Die Benachrichtigung folgt immer der Systemsprache.
