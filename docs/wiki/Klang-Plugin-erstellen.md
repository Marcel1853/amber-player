# Klang-Plugin erstellen

Ein Klang-Plugin ist eine JSON-Datei. Sie beschreibt **Regler** und eine **Kette aus
Bausteinen** (Filter, Echo, Sättigung …). Die Klangberechnung macht der Player selbst – schnell,
sicher und ohne dass du programmieren musst.

## Schritt für Schritt: „Telefon“

Wir bauen einen Effekt, der klingt wie durchs Telefon: Tiefen und Höhen weg, etwas verzerrt,
mono.

**1. Datei anlegen**, z. B. `telefon.json`:

```json
{
  "id": "telefon",
  "name": "Telefon",
  "author": "Dein Name",
  "description": "Klingt wie durchs Telefon",
  "chain": [
    { "type": "highpass", "freq": 400 },
    { "type": "lowpass", "freq": 3200 },
    { "type": "saturate", "drive": 2.5 },
    { "type": "width", "amount": 0 }
  ]
}
```

**2. Installieren:** in den Ordner `effects` legen (PC) bzw. importieren (Handy), App neu
starten. Unter **Effekte** erscheint „Telefon“ mit einem Schalter.

**3. Einen Regler hinzufügen.** Mit `params` bekommt der Effekt Regler; ihre `id` kannst du in
Formeln verwenden. `mix` legt fest, wie viel vom bearbeiteten Klang zu hören ist:

```json
{
  "id": "telefon",
  "name": "Telefon",
  "author": "Dein Name",
  "description": "Klingt wie durchs Telefon",
  "params": [
    { "id": "staerke", "name": "Stärke", "min": 0, "max": 100, "default": 100, "step": 5, "unit": "%" },
    { "id": "enge", "name": "Enge", "min": 0, "max": 100, "default": 50, "unit": "%" }
  ],
  "presets": [
    { "name": "Leise Leitung", "values": [60, 30] },
    { "name": "Funkgerät", "values": [100, 100] }
  ],
  "mix": "staerke / 100",
  "chain": [
    { "type": "highpass", "freq": "200 + enge * 6", "q": 0.9 },
    { "type": "lowpass", "freq": "5000 - enge * 25", "q": 0.9 },
    { "type": "saturate", "drive": "1 + enge / 40" },
    { "type": "width", "amount": 0 },
    { "type": "gain", "db": 2 }
  ]
}
```

Jetzt hat der Effekt zwei Regler und zwei Presets.

## Felder

| Feld | Pflicht | Bedeutung |
|---|---|---|
| `id` | ja | 1–32 Zeichen aus Buchstaben, Ziffern, `-`, `_`; eindeutig |
| `name` | ja | Anzeigename (bis 40 Zeichen) |
| `author`, `description` | | erscheinen in der Effekt-Liste |
| `params` | | bis 16 Regler: `id`, `name`, `min`, `max`, `default`, optional `step`, `unit` |
| `presets` | | `{ "name": "…", "values": [ … ] }` – Werte in der Reihenfolge der Regler |
| `mix` | | Anteil des bearbeiteten Klangs von 0 bis 1 (Standard 1); Zahl oder Formel |
| `chain` | ja | 1–32 Bausteine, nacheinander angewendet |

## Bausteine

| `type` | Felder (Standardwert) | Wirkung |
|---|---|---|
| `gain` | `db` (0) | lauter/leiser in dB |
| `peaking` | `freq` (1000), `gain` (0), `q` (1) | Glockenfilter: Bereich um `freq` anheben/absenken |
| `lowshelf` | `freq` (200), `gain` (0), `q` (0.707) | alles unter `freq` anheben/absenken (Bass) |
| `highshelf` | `freq` (200), `gain` (0), `q` (0.707) | alles über `freq` anheben/absenken (Höhen) |
| `lowpass` | `freq` (1000), `q` (0.707) | nur Tiefen durchlassen |
| `highpass` | `freq` (1000), `q` (0.707) | nur Höhen durchlassen |
| `echo` | `time` in ms (300), `feedback` 0–0.95 (0.3), `mix` 0–1 (0.3) | Echo |
| `width` | `amount` (1) | Stereobreite: 0 = Mono, 1 = normal, 2 = breit |
| `saturate` | `drive` (1) | weiche Sättigung bis Verzerrung |
| `compressor` | `threshold` dB (−18), `ratio` (3), `attack` ms (10), `release` ms (150), `makeup` dB (0) | gleicht Lautstärkeunterschiede an |
| `autopan` | `rate` Hz (0.2), `depth` 0–1 (0.8) | Klang wandert zwischen links und rechts („8D“) |

`q` bestimmt die Breite eines Filters: kleiner = breiter und sanfter, größer = enger und schärfer.

## Formeln

Jeder Wert darf eine Zahl **oder eine Formel** mit den Regler-IDs sein:

```json
{ "type": "lowpass", "freq": "6000 - alter * 35" }
```

| Erlaubt | Beispiele |
|---|---|
| Rechenzeichen | `+ - * / ^` und Klammern |
| Funktionen | `min(a, b)`, `max(a, b)`, `clamp(x, a, b)`, `abs(x)`, `sqrt(x)`, `pow(a, b)`, `exp(x)`, `log10(x)` |
| Konstante | `pi` |

Formeln werden nur neu berechnet, wenn sich ein Regler ändert – sie kosten beim Abspielen
keine Rechenzeit.

## Tipps für guten Klang

- **Leiser machen, wenn du anhebst:** Hebst du kräftig an (`lowshelf` mit +9 dB), gleiche mit
  `gain` aus (z. B. `-staerke / 3`). Der Schutz-Limiter fängt Spitzen ab, aber ausgeglichene
  Lautstärke klingt besser.
- **Erst filtern, dann verzerren:** `saturate` nach den Filtern klingt sauberer.
- **`mix` für „wie viel“:** Ein Stärke-Regler über `mix` ist die einfachste Art, einen Effekt
  dosierbar zu machen.
- **Vorbilder:** Die mitgelieferten Effekte (Bass-Booster, Echo, Altes Radio, 8D-Rundflug)
  sind im selben Format geschrieben: [`crates/core/src/effects/builtin/`](https://github.com/Marcel1853/amber-player/tree/main/crates/core/src/effects/builtin).

## Fehlersuche

Fehlerhafte Plugins werden übersprungen, die Meldung steht oben auf der Seite **Effekte** unter
„Nicht geladen“. Häufige Ursachen:

| Meldung | Ursache |
|---|---|
| JSON-Fehler mit Zeilenangabe | Komma vergessen oder zu viel, Anführungszeichen fehlt |
| unbekannter Baustein | Tippfehler bei `type` |
| unbekanntes Feld | Feld gehört nicht zu diesem Baustein (z. B. `gain` bei `lowpass`) |
| unbekannter Name in Formel | Regler-`id` falsch geschrieben |
| Die ID gibt es schon | zwei Plugins mit gleicher `id` |
