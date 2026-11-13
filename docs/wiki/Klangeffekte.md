# Klangeffekte

Öffnen über das **Regler-Symbol** oben in der Bibliothek. Jeder Effekt hat einen Schalter
(an/aus), Regler und oft fertige **Presets**. Änderungen wirken sofort auf die laufende Musik
und werden gespeichert.

## Mitgelieferte Effekte

| Effekt | Wofür |
|---|---|
| **Equalizer** | 10 Bänder von 31 Hz bis 16 kHz. Presets: Neutral, Bass, Höhen, Pop, Rock, Elektronisch, Klassik, Stimme |
| **Bass-Booster** | mehr Druck im Tiefbass, ohne dass es dröhnt. Presets: Dezent, Club, Kopfhörer |
| **Stimmen klarer** | hebt Stimmen hervor – für Hörbücher, Podcasts, leise Gesangsstellen |
| **Kopfhörer-Crossfeed** | mischt tiefe Frequenzen leicht auf die andere Seite – entspannter mit Kopfhörern |
| **Stereobreite** | 0 % = Mono, 100 % = normal, bis 200 % = breit |
| **Echo** | Wiederholungen wie in einer Halle oder am Berg |
| **Altes Radio** | schmal, warm, leicht verzerrt wie ein Röhrenradio |
| **8D-Rundflug** | die Musik wandert langsam um den Kopf – am besten mit Kopfhörern |
| **Vorverstärker & Limiter** | lauter/leiser und Spitzen abfangen |

## Tipps

- **Doppelklick** auf einen Regler setzt ihn auf den Standardwert (beim Equalizer 0 dB).
- Der Equalizer senkt die Lautstärke automatisch so weit ab, wie du anhebst. Alle Bänder auf
  +12 dB klingt deshalb fast wie alle auf 0 – heb gezielt einzelne Bereiche an.
- Am Ende der Kette sitzt immer ein **Schutz-Limiter**: er fängt Spitzen weich ab, statt sie
  hart abzuschneiden. So knackt es auch bei kräftigen Einstellungen nicht.
- Am Handy hat der Equalizer eine waagerechte Zeile pro Band – gut mit dem Finger zu greifen.

## Eigene Effekte

Ein eigener Effekt ist eine kleine JSON-Datei aus fertigen Bausteinen (Filter, Echo, Sättigung,
Kompressor …) – programmieren musst du nicht.

- Installieren: [Plugins installieren](Plugins-installieren.md)
- Selbst bauen: [Klang-Plugin erstellen](Klang-Plugin-erstellen.md)

## Über die API

```sh
curl "http://127.0.0.1:7878/api/effects/equalizer?preset=Rock"
curl "http://127.0.0.1:7878/api/effects/bass-booster/toggle"
```

Alle Befehle: [Fernsteuerung und API](Fernsteuerung-und-API.md).
