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
2. **Am Handy:** in Amber Player unter **PC** auf **QR-Code scannen** tippen und den Code in den
   Rahmen halten (beim ersten Mal fragt Android nach der Kamera). Oder den QR-Code mit der
   normalen Kamera-App scannen und **Öffnen** wählen.
3. Amber Player meldet „Mit dem PC verbunden“.

**Mehrere Geräte:** Jedes Handy und Tablet koppelst du mit demselben QR-Code – beliebig viele.

**Ohne Kamera:** Den Link unter dem QR-Code kopieren und in der App unter **Mehr → PC** einfügen.
Es geht auch die API-Adresse des PCs, z. B. `http://192.168.178.35:7878/api?token=…`.

> Der Link enthält das **Token**, das den Zugriff schützt. Gib ihn nicht weiter. Ein neues Token
> erzeugst du am PC unter Fernsteuerung → **Neu erzeugen**; danach muss das Handy neu gekoppelt
> werden.

## Die Ansicht „PC“

Unter **Mehr → PC** – oder direkt unten in der Leiste: In der Ansicht „PC“ auf **In die Leiste**
tippen (ändern unter Einstellungen → Darstellung → Leiste unten).

Oben steht immer, **was am PC läuft**: Cover, Titel, Favorit, Zeitleiste zum Spulen, Zufall,
Zurück, Play/Pause, Weiter, Wiederholen und zwei Regler – **Amber** (Lautstärke von Amber Player)
und **PC gesamt** (Hauptlautstärke des PCs). Darunter vier Bereiche:

| Bereich | Bedienung |
|---|---|
| **Läuft** | Als Nächstes am PC. Tippen spielt den Titel **auf dem Handy**, das Bildschirm-Symbol **am PC**. |
| **Musik** | Suche in der Bibliothek des PCs und die Playlisten (**Hier** = auf dem Handy, **Am PC** = am PC). |
| **Effekte** | Klangeffekte am PC ein- und ausschalten, Presets wählen. |
| **Plugins** | Plugin-Pakete vom PC **aufs Handy** holen – mit allen Teilen und der Lizenz. Ist eine andere Version auf dem Handy, steht dort **Aktualisieren**. |

**Verbindung trennen** löscht die Kopplung.

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
