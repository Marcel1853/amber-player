# Sound effects

Open them with the **sliders icon** at the top of the library. Each effect has an on/off switch,
sliders and often ready-made **presets**. Changes apply immediately to the music that's playing
and are saved.

## Included effects

| Effect | Purpose |
|---|---|
| **Equalizer** | 10 bands from 31 Hz to 16 kHz. Presets: Neutral, Bass, Treble, Pop, Rock, Electronic, Classical, Voice |
| **Tone** | boost or cut bass and treble separately, like on a stereo system |
| **Bass booster** | more punch in the deep bass without booming. Presets: Subtle, Club, Headphones |
| **Clearer voices** | brings voices forward – audiobooks, podcasts, quiet vocal passages |
| **Headphone crossfeed** | blends low frequencies slightly to the other side – more relaxed on headphones |
| **Stereo & balance** | Width: 0 % = mono, 100 % = normal, up to 200 % = wide. Balance: louder left or right |
| **Reverb** | room sound from a small room to a stadium – see below |
| **Echo** | repeats like in a hall or in the mountains |
| **Old radio** | narrow, warm, slightly distorted like a valve radio |
| **8D flight** | the music slowly moves around your head – best with headphones |
| **Preamp & limiter** | louder/quieter and catch peaks |

## Reverb

| Slider | Effect |
|---|---|
| **Size** | how big the room is |
| **Damping** | how quickly the high tones fade in the reverb – more damping sounds softer and warmer, like a room with carpets and curtains |
| **Filter** | removes the lows below this frequency from the reverb so the bass stays clear instead of booming |
| **Decay time** | how long the reverb rings out (seconds) |
| **Pre-delay** | gap between the original and the start of the reverb – larger values make big rooms sound more natural |
| **Mix** | how strongly the reverb is heard overall, so a big, long reverb can still stay subtle |

Presets: Light reverb (default), Room, Studio, Hall, Church, Stadium.

## Your own presets

Below every effect: **Save** stores the current values under a name; they then appear in the
list under **Your own**. **Reset** puts all sliders back to their defaults. Select one of your
presets and press **Delete preset** to remove it.

## Tips

- **Double-click** a slider to reset it to its default (0 dB on the equalizer).
- The equalizer automatically lowers the volume by as much as you boost. All bands at +12 dB
  therefore sounds almost like all at 0 – boost specific ranges instead.
- A **safety limiter** always sits at the end of the chain: it catches peaks smoothly instead of
  clipping them hard, so nothing crackles even with strong settings.
- On the phone the equalizer has one horizontal row per band – easy to grab with a finger.

## Your own effects

An effect is a small JSON file built from ready-made blocks (filters, echo, saturation,
compressor …) – no programming needed.

- Install: [Installing plugins](Installing-Plugins.md)
- Build: [Creating a sound plugin](Creating-a-Sound-Plugin.md)

## Via the API

```sh
curl "http://127.0.0.1:7878/api/effects/equalizer?preset=Rock"
curl "http://127.0.0.1:7878/api/effects/bass-booster/toggle"
```

Note: preset names in the API are the German originals (e.g. `Rock`, `Bass`, `Klassik`).
All commands: [Remote control and API](Remote-Control-and-API.md).
