# Creating a visualizer

A visualizer is a JavaScript file. It registers with `amber` and draws on a canvas
([Canvas 2D](https://developer.mozilla.org/en-US/docs/Web/API/CanvasRenderingContext2D)) in
every frame. Amber Player provides frequency bands, waveform, level, title, cover and accent
colours.

## The smallest example

```js
amber.visualizer({
  id: 'my-circle',
  name: 'My circle',
  author: 'Your name',
  description: 'A circle that grows with the bass',

  draw({ ctx, width, height, frame, info, amber }) {
    ctx.fillStyle = '#000';
    ctx.fillRect(0, 0, width, height);
    ctx.fillStyle = amber.rgba(info.accent);
    ctx.beginPath();
    ctx.arc(width / 2, height / 2, 40 + frame.bass * 200, 0, Math.PI * 2);
    ctx.fill();
  },
});
```

Save it as `my-circle.js` in the `visualizers` folder (PC) or import it (phone), then open the
visualizer (**V**) – “My circle ★” appears at the top.

## Smoother: `state`

Values jump from frame to frame. With `state` (kept between frames) and `amber.lerp` you can
smooth them:

```js
amber.visualizer({
  id: 'my-circle',
  name: 'My circle',

  setup({ state }) {
    state.r = 0;
  },

  draw({ ctx, width, height, frame, info, state, amber }) {
    ctx.fillStyle = 'rgba(0, 0, 0, 0.25)'; // semi-transparent → trail effect
    ctx.fillRect(0, 0, width, height);
    state.r = amber.lerp(state.r, frame.bass, 0.15);
    ctx.fillStyle = amber.mix(info.accent, info.accent2, frame.treble);
    ctx.beginPath();
    ctx.arc(width / 2, height / 2, 40 + state.r * 220, 0, Math.PI * 2);
    ctx.fill();
  },
});
```

## Callbacks

| Function | When | Required |
|---|---|---|
| `setup(scene)` | when switching to this visualizer | no |
| `resize(scene)` | when the window size changes | no |
| `draw(scene)` | every frame (up to 60 times per second) | yes |

## What's in `scene`

| Field | Content |
|---|---|
| `ctx` | 2D drawing context. Units are screen points, pixel density is already applied |
| `width`, `height` | size of the area |
| `state` | object for your own data, kept between frames |
| `frame.bands` | 64 frequency bands from low to high, each 0–1 |
| `frame.wave` | waveform, values −1 to 1 |
| `frame.level` | overall level 0–1 |
| `frame.bass`, `frame.mid`, `frame.treble` | averages of bass, mids, treble (0–1) |
| `frame.time`, `frame.dt` | running time and time since the last frame in seconds |
| `frame.playing` | is music playing? |
| `info.title`, `info.artist`, `info.album` | current track |
| `info.cover` | cover as an image for `ctx.drawImage` (for radio: the station logo) or `null` |
| `info.accent`, `info.accent2` | accent colours as `[r, g, b]` |

## Helpers in `amber`

| Function | Effect |
|---|---|
| `amber.rgba([r, g, b], a)` | colour as CSS text |
| `amber.mix(c1, c2, t, a)` | mix two colours (`t` 0–1) |
| `amber.lerp(a, b, t)` | smoothly from `a` to `b` |
| `amber.clamp(x, a, b)` | limit a value |
| `amber.range(bands, from, to)` | average of a range of bands (0–1 = low–high), e.g. `amber.range(frame.bands, 0.6, 1)` for the treble |

## Security and limits

Visualizers run in a **separate, sandboxed background thread** (Web Worker):

- no network (`fetch`, WebSocket … are disabled), no files, no storage,
- no access to the interface or the player's controls,
- errors and endless loops don't slow down the player; the message appears at the bottom right.

The file may be at most 512 KB.

## Tips

- **Paint over semi-transparently** (`rgba(…, 0.2)`) instead of clearing completely for soft
  trails.
- **Use `frame.dt` for movement** so it runs at the same speed on every device.
- **Check `frame.playing`** and calm down while paused – looks more alive.
- **Use the accent colours** so the visualizer matches the user's colour scheme.
- **Test on a phone:** it has less power – prefer 200 particles over 5,000.

## Templates

- [`examples/plugins/sternenfeld.js`](https://github.com/Marcel1853/amber-player/blob/main/examples/plugins/sternenfeld.js) – stars flying towards you
- [`crates/app/ui/visualizers/`](https://github.com/Marcel1853/amber-player/tree/main/crates/app/ui/visualizers) – Bars, Wave, Ring, Fog
