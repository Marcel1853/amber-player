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
2. **On the phone:** scan the QR code with the camera app and choose **Open**.
3. Amber Player opens and reports “Connected to the PC”.

**Without a camera:** copy the link below the QR code and paste it in the app under
**More → PC**. The PC's API address works too, e.g. `http://192.168.178.35:7878/api?token=…`.

> The link contains the **token** that protects access. Don't share it. To create a new token,
> use **Regenerate** on the PC; the phone then has to be paired again.

## The “PC” view

Under **More → PC**:

| Section | Usage |
|---|---|
| **What's playing on the PC** | cover, title, previous, play/pause, next, volume. Refreshes every two seconds. |
| **Music from the PC** | search box: tapping a result plays it **on the phone**; the screen icon plays it **on the PC**. |
| **Playlists on the PC** | **Here** plays the playlist on the phone, **On PC** on the PC. |
| **Disconnect** | remove the pairing. |

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
