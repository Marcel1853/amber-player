# Telefon – Klangeffekt für Amber Player

Klingt wie durchs Telefon: Tiefen und Höhen weg, leicht verzerrt, mono.
*Sounds like a phone call: lows and highs removed, slightly distorted, mono.*

| | |
|---|---|
| Art | Klangeffekt (`effect.json`) |
| Regler | Stärke (0–100 %) |
| Lizenz | MIT – frei als Vorlage für eigene Plugins |

## Installieren

`telefon.amberplugin` in Amber Player unter **Effekte → Installieren** auswählen und die App neu
starten. Das Paket baust du aus diesem Ordner mit:

```sh
python3 scripts/plugin-pack.py examples/plugins/telefon
```

## Als Vorlage nutzen

Ordner kopieren, in `plugin.json` und `effect.json` die `id` und den `name` ändern, eigene
Bausteine eintragen. Anleitung: [Klang-Plugin erstellen](../../../docs/wiki/Klang-Plugin-erstellen.md)
· [Creating a sound plugin](../../../docs/wiki/en/Creating-a-Sound-Plugin.md)
