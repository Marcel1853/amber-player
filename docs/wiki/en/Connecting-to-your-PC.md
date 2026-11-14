# Connecting to your PC

Once the phone app is paired with Amber Player on your PC, you can

- **control the PC:** see what's playing, play/pause, skip, volume, start tracks and playlists
  on the PC,
- **stream from the PC:** search the PC's music and listen to tracks or whole playlists on your
  phone – without copying files, with seeking.

## Requirements

- PC and phone are on the **same Wi-Fi**.
- Amber Player is running on the PC.
- On the PC, **Settings → Remote control → Home network** is on. (The “Pair phone” button asks
  for it otherwise and restarts Amber Player once.)
- If the **ufw** firewall is active on the PC, Amber Player opens the port for your home network
  – a password window appears once.

## Pairing with a QR code

1. **On the PC:** Settings → Remote control & MCP → **Pair phone**. A QR code appears.
2. **On the phone:** in Amber Player under **PC**, tap **Scan QR code** and hold the code inside
   the frame (the first time, Android asks for the camera). Or scan the QR code with the normal
   camera app and choose **Open**.
3. Amber Player reports “Connected to the PC”.

**Several devices:** pair every phone and tablet with the same QR code – as many as you like.

**Without a camera:** copy the link below the QR code and paste it in the app under
**More → PC**. The PC's API address works too, e.g. `http://192.168.178.35:7878/api?token=…`.

> The link contains the **token** that protects access. Don't share it. To create a new token,
> use **Regenerate** on the PC; the phone then has to be paired again.

## The “PC” view

Under **More → PC** – or right in the bottom bar: in the “PC” view, tap **Add to bar** (change it
under Settings → Appearance → Bottom bar).

At the top you always see **what's playing on the PC**: cover, title, favourite, a timeline for
seeking, shuffle, previous, play/pause, next, repeat and two sliders – **Amber** (Amber Player's
volume) and **PC overall** (the PC's main volume). Below are four sections:

| Section | Usage |
|---|---|
| **Playing** | Up next on the PC. Tapping plays the track **on the phone**, the screen icon **on the PC**. |
| **Music** | search the PC's library and its playlists (**Here** = on the phone, **On PC** = on the PC). |
| **Effects** | turn sound effects on the PC on and off, choose presets. |
| **Plugins** | get plugin packages from the PC **onto the phone** – with all parts and the license. If the phone has a different version, the button says **Update**. |

**Disconnect** removes the pairing.

Tracks from the PC behave like your own on the phone: queue, favourites, effects and visualizer
work. They are streamed, not stored – without a connection to the PC they won't play.

## Other player apps (Poweramp & co.)

If you prefer another player app: on the PC under **Settings → Stream to your phone** there's a
link with QR code for every playlist. Download the `.m3u` file and open it with your player app
(in Poweramp, don't use “Add URL” – Poweramp can't read lists there).

## Problems

| Message | Solution |
|---|---|
| “PC not reachable” | Same Wi-Fi? Amber Player running on the PC? “Home network” on? Firewall opened? |
| “The PC refused the connection” | The token changed – pair again. |
| QR code doesn't open anything | The camera app doesn't offer the link → copy the link and paste it under **More → PC**. |

See also [FAQ](FAQ.md).
