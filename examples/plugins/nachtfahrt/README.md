# Nachtfahrt – Beispiel-Paket für Amber Player

Ein Paket mit **mehreren Teilen** in einer Datei:
*A package with several parts in one file:*

| Teil | Art | Beschreibung |
|---|---|---|
| Kassette | Klangeffekt | warmer Klang wie von einer alten Musikkassette |
| Stadion | Klangeffekt | breit und mit Nachhall wie aus Stadion-Lautsprechern |
| Polarlicht | Visualisierung | wehende Lichtbänder in der Akzentfarbe |

Lizenz: MIT – frei als Vorlage für eigene Pakete.

## Installieren

`nachtfahrt.amberplugin` in Amber Player unter **Einstellungen → Plugins → Installieren**
auswählen. „Polarlicht“ erscheint sofort in der Visualisierung, die Klangeffekte nach dem
nächsten Start der App unter **Effekte**. Die Datei baust du aus diesem Ordner mit:

```sh
python3 scripts/plugin-pack.py examples/plugins/nachtfahrt
```

## Aufbau

```
nachtfahrt/
├── plugin.json              ← Liste der Teile in „contents“
├── effects/kassette.json
├── effects/stadion.json
├── visualizers/polarlicht.js
├── LICENSE
└── README.md
```

Anleitung: [Plugin-Pakete](../../../docs/wiki/Plugin-Pakete.md) · [Plugin packages](../../../docs/wiki/en/Plugin-Packages.md)
