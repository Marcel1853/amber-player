# Mit dem PC verbinden

Ist die Handy-App mit dem Amber Player am PC gekoppelt, kannst du

- **den PC fernsteuern:** sehen, was läuft, Play/Pause, Titel wechseln, Lautstärke, Titel und
  Playlisten am PC starten,
- **vom PC streamen:** in der Musik des PCs suchen und Titel oder ganze Playlisten auf dem
  Handy hören – ohne Dateien zu kopieren, mit Spulen.

## Voraussetzungen

- PC und Handy sind im **selben WLAN**.
- Amber Player läuft am PC.
- Am PC ist **Einstellungen → Fernsteuerung → Im Heimnetz** eingeschaltet. (Der Knopf
  „Handy koppeln“ fragt sonst selbst danach und startet Amber Player einmal neu.)
- Ist am PC die Firewall **ufw** aktiv, gibt Amber Player den Port fürs Heimnetz frei – dafür
  erscheint einmal ein Passwortfenster.

## Koppeln mit QR-Code

1. **Am PC:** Einstellungen → Fernsteuerung & MCP → **Handy koppeln**. Ein QR-Code erscheint.
2. **Am Handy:** QR-Code mit der Kamera-App scannen und **Öffnen** wählen.
3. Amber Player öffnet sich und meldet „Mit dem PC verbunden“.

**Ohne Kamera:** Den Link unter dem QR-Code kopieren und in der App unter **Mehr → PC** einfügen.
Es geht auch die API-Adresse des PCs, z. B. `http://192.168.178.35:7878/api?token=…`.

> Der Link enthält das **Token**, das den Zugriff schützt. Gib ihn nicht weiter. Ein neues Token
> erzeugst du am PC unter Fernsteuerung → **Neu erzeugen**; danach muss das Handy neu gekoppelt
> werden.

## Die Ansicht „PC“

Unter **Mehr → PC**:

| Bereich | Bedienung |
|---|---|
| **Was am PC läuft** | Cover, Titel, Zurück, Play/Pause, Weiter, Lautstärke. Aktualisiert sich alle zwei Sekunden. |
| **Musik vom PC** | Suchfeld: Tippen auf einen Treffer spielt ihn **auf dem Handy**; das Bildschirm-Symbol spielt ihn **am PC**. |
| **Playlisten am PC** | **Hier** spielt die Playlist auf dem Handy, **Am PC** am PC. |
| **Verbindung trennen** | Kopplung löschen. |

Titel vom PC erscheinen auf dem Handy wie eigene: Warteschlange, Favoriten, Effekte und
Visualisierung funktionieren. Sie werden gestreamt, nicht gespeichert – ohne Verbindung zum PC
spielen sie nicht.

## Andere Player-Apps (Poweramp & Co.)

Wer lieber eine andere Player-App nutzt: Am PC unter **Einstellungen → Aufs Handy streamen**
gibt es für jede Playlist einen Link mit QR-Code. Die `.m3u`-Datei herunterladen und mit der
Player-App öffnen (bei Poweramp nicht unter „URL hinzufügen“ eintragen – dort liest Poweramp
keine Listen).

## Probleme

| Meldung | Lösung |
|---|---|
| „PC nicht erreichbar“ | Gleiches WLAN? Amber Player am PC gestartet? „Im Heimnetz“ an? Firewall freigegeben? |
| „Der PC hat die Verbindung abgelehnt“ | Das Token hat sich geändert – neu koppeln. |
| QR-Code öffnet nichts | Kamera-App bietet den Link nicht an → Link kopieren und unter **Mehr → PC** einfügen. |

Siehe auch [Häufige Fragen](Häufige-Fragen.md).
