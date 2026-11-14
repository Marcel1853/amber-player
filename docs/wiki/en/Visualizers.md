# Visualizers

Open them with **V** or the icon in the player. Choose the visualizer at the top; at the bottom
you see the title (long names scroll) and the controls.

| Visualizer | Description |
|---|---|
| **Bars** | frequency bars with reflection and falling peaks |
| **Wave** | glowing waveform line with a trail |
| **Ring** | spinning cover (for radio: the station logo) with a circular spectrum, pulsing with the bass |
| **Fog** | flowing clouds of colour reacting to bass, mids and treble |

## Choosing

The button at the top left opens the list of all visualizers – grouped into **Favourites**,
**Built-in** and one group per **plugin package** (with name and author), plus your own single
files.

- **Search:** type at the top of the list; **Enter** picks the first match.
- **Star** next to an entry: it then appears at the very top under **Favourites**.
- **Change automatically** (at the bottom of the list): **In order**, **Random** or **Random from
  favourites** – every 15 seconds up to 5 minutes, or **with every new track**.

## Controls

| Key | Effect |
|---|---|
| 1–9 | choose a visualizer |
| F | full screen |
| Esc or V | close |

After a few seconds without the mouse, the bars and the pointer fade out – set how long under
**Settings → Appearance → Visualizer** (3, 6, 10, 20 seconds or never). On the phone, **tap**
to show them and **tap again** to hide them right away.

The colours follow your **accent colour** (Settings → Appearance).

### On the phone

In **Now playing**, tap the visualizer icon:

1. **once** – the visualizer runs right behind the title and buttons. After a few seconds the
   controls fade out; tap once to bring them back. **Swipe left/right** to switch the style.
2. **twice** – full screen with its own controls
3. **three times** – off

## Your own visualizers

A visualizer is a JavaScript file that draws on a canvas. It runs sandboxed – without access to
the network, files or the player.

- Install: [Installing plugins](Installing-Plugins.md)
- Build: [Creating a visualizer](Creating-a-Visualizer.md)

Your own visualizers are marked with ★ in the selection.
