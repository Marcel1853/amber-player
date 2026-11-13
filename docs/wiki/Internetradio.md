# Internetradio

Amber Player durchsucht die freie Senderdatenbank [radio-browser.info](https://www.radio-browser.info)
mit über 50.000 Sendern aus aller Welt.

## Sender finden und hören

1. **Radio** öffnen. Ohne Suchbegriff siehst du die beliebtesten Sender des gewählten Landes.
2. Im Suchfeld Name oder Genre eingeben, z. B. „1live“, „rock“, „jazz“.
3. Mit der Länderauswahl eingrenzen oder **Alle Länder** wählen.
4. Sender antippen bzw. anklicken – er spielt sofort.

Während ein Sender läuft, zeigt Amber Player den Songtitel, den der Sender mitschickt. In der
[Visualisierung](Visualisierungen.md) erscheint das Senderlogo (z. B. in der Mitte des Rings).

## Meine Sender

- Mit **★** speicherst du einen Sender unter **Meine Sender**.
- **Weiter / Zurück** wechseln beim Radio zwischen deinen gespeicherten Sendern.
- Entfernen: Rechtsklick (Handy: lange drücken) → **Aus „Meine Sender“ entfernen**.

## Eigene Adresse

Fehlt ein Sender in der Datenbank: **Eigene Adresse** und die Stream-Adresse eintragen. Sie
beginnt mit `http://` oder `https://`; auch Playlist-Adressen auf `.m3u` oder `.pls` gehen.

## Hinweise

- Radiostreams werden nur abgespielt, nicht aufgenommen oder weitergesendet.
- Manche Sender liefern keine Songtitel oder kein Logo – dann steht dort nur der Sendername.
- Klangeffekte wirken auch beim Radio.
- Über die API: `/api/radio/search?q=…`, `/api/radio/play/{id}` – siehe [Fernsteuerung und API](Fernsteuerung-und-API.md).
