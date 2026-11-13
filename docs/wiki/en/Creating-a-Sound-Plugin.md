# Creating a sound plugin

A sound plugin is a JSON file. It describes **sliders** and a **chain of building blocks**
(filters, echo, saturation …). The player does the audio processing itself – fast, safe and
without any programming on your side.

## Step by step: “Telephone”

We'll build an effect that sounds like a phone call: lows and highs removed, slightly distorted,
mono.

**1. Create a file**, e.g. `telephone.json`:

```json
{
  "id": "telephone",
  "name": "Telephone",
  "author": "Your name",
  "description": "Sounds like a phone call",
  "chain": [
    { "type": "highpass", "freq": 400 },
    { "type": "lowpass", "freq": 3200 },
    { "type": "saturate", "drive": 2.5 },
    { "type": "width", "amount": 0 }
  ]
}
```

**2. Install it:** put it in the `effects` folder (PC) or import it (phone) and restart the app.
“Telephone” appears under **Effects** with a switch.

**3. Add sliders.** `params` gives the effect sliders; their `id` can be used in formulas. `mix`
sets how much of the processed sound you hear:

```json
{
  "id": "telephone",
  "name": "Telephone",
  "author": "Your name",
  "description": "Sounds like a phone call",
  "params": [
    { "id": "strength", "name": "Strength", "min": 0, "max": 100, "default": 100, "step": 5, "unit": "%" },
    { "id": "narrow", "name": "Narrowness", "min": 0, "max": 100, "default": 50, "unit": "%" }
  ],
  "presets": [
    { "name": "Quiet line", "values": [60, 30] },
    { "name": "Walkie-talkie", "values": [100, 100] }
  ],
  "mix": "strength / 100",
  "chain": [
    { "type": "highpass", "freq": "200 + narrow * 6", "q": 0.9 },
    { "type": "lowpass", "freq": "5000 - narrow * 25", "q": 0.9 },
    { "type": "saturate", "drive": "1 + narrow / 40" },
    { "type": "width", "amount": 0 },
    { "type": "gain", "db": 2 }
  ]
}
```

Now the effect has two sliders and two presets.

## Fields

| Field | Required | Meaning |
|---|---|---|
| `id` | yes | 1–32 characters: letters, digits, `-`, `_`; unique |
| `name` | yes | display name (up to 40 characters) |
| `author`, `description` | | shown in the effects list |
| `params` | | up to 16 sliders: `id`, `name`, `min`, `max`, `default`, optional `step`, `unit` |
| `presets` | | `{ "name": "…", "values": [ … ] }` – values in slider order |
| `mix` | | share of the processed sound from 0 to 1 (default 1); number or formula |
| `chain` | yes | 1–32 blocks, applied one after another |

## Building blocks

| `type` | Fields (default) | Effect |
|---|---|---|
| `gain` | `db` (0) | louder/quieter in dB |
| `peaking` | `freq` (1000), `gain` (0), `q` (1) | bell filter: boost/cut the range around `freq` |
| `lowshelf` | `freq` (200), `gain` (0), `q` (0.707) | boost/cut everything below `freq` (bass) |
| `highshelf` | `freq` (200), `gain` (0), `q` (0.707) | boost/cut everything above `freq` (treble) |
| `lowpass` | `freq` (1000), `q` (0.707) | let only lows through |
| `highpass` | `freq` (1000), `q` (0.707) | let only highs through |
| `echo` | `time` in ms (300), `feedback` 0–0.95 (0.3), `mix` 0–1 (0.3) | echo |
| `width` | `amount` (1) | stereo width: 0 = mono, 1 = normal, 2 = wide |
| `saturate` | `drive` (1) | soft saturation up to distortion |
| `compressor` | `threshold` dB (−18), `ratio` (3), `attack` ms (10), `release` ms (150), `makeup` dB (0) | evens out loudness differences |
| `autopan` | `rate` Hz (0.2), `depth` 0–1 (0.8) | sound moves between left and right (“8D”) |

`q` sets a filter's width: smaller = wider and gentler, larger = narrower and sharper.

## Formulas

Every value may be a number **or a formula** using the slider IDs:

```json
{ "type": "lowpass", "freq": "6000 - age * 35" }
```

| Allowed | Examples |
|---|---|
| operators | `+ - * / ^` and parentheses |
| functions | `min(a, b)`, `max(a, b)`, `clamp(x, a, b)`, `abs(x)`, `sqrt(x)`, `pow(a, b)`, `exp(x)`, `log10(x)` |
| constant | `pi` |

Formulas are only recalculated when a slider changes – they cost no processing time during
playback.

## Tips for good sound

- **Compensate when boosting:** if you boost a lot (`lowshelf` at +9 dB), balance it with `gain`
  (e.g. `-strength / 3`). The safety limiter catches peaks, but balanced loudness sounds better.
- **Filter first, then distort:** `saturate` after the filters sounds cleaner.
- **Use `mix` for “how much”:** a strength slider via `mix` is the easiest way to make an effect
  adjustable.
- **Examples:** the included effects (Bass booster, Echo, Old radio, 8D flight) use the same
  format: [`crates/core/src/effects/builtin/`](https://github.com/Marcel1853/amber-player/tree/main/crates/core/src/effects/builtin).

## Troubleshooting

Broken plugins are skipped; the message appears at the top of the **Effects** page under
“Not loaded”. Common causes:

| Problem | Cause |
|---|---|
| JSON error with line number | missing or extra comma, missing quote |
| unknown block | typo in `type` |
| unknown field | field doesn't belong to this block (e.g. `gain` on `lowpass`) |
| unknown name in formula | slider `id` misspelled |
| the ID already exists | two plugins with the same `id` |
