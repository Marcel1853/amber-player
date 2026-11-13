# FAQ

## General

**Does Amber Player cost anything? Are there ads?**
No and no. Amber Player is free and ad-free – and will stay that way.

**Does Amber Player collect data?**
No. There's no account and no statistics. Only what you use goes online: the cover search
(sends artist, title, album), the radio station list and the radio streams.

**Is Amber Player open source?**
No. The source code is public, but all rights are reserved – see
[LICENSE](https://github.com/Marcel1853/amber-player/blob/main/LICENSE). Writing your own plugins
is explicitly welcome.

**Is there Amber Player for Windows, macOS or iPhone?**
Not at the moment. There are packages for Linux and an app for Android.

**Which version do I have?**
At the very bottom of **Settings**.

**How do I get updates?**
Amber Player checks GitHub for a new version once a day. If there is one, a message appears and
the **Download** button shows up at the top of **Settings** – on the phone it downloads the APK,
on the PC it opens the release page. Library, playlists and settings are kept during the update.
To turn this off: at the very bottom of Settings.

## Library

**A track is missing.**
Is it inside a configured music folder? **Settings → Rescan**. On the phone only the **Music**
folder is scanned.

**Many tracks show “Unknown artist”.**
The files have no artist tag. Amber Player tries to read it from file names like
“Artist - Title”. The most reliable fix is to add the tags with a tag editor (e.g. Kid3, Picard)
and rescan.

**Two different songs have the same name – are they merged?**
No, never. Every file stays its own track.

**A cover is wrong or missing.**
Amber Player only uses online covers when artist and title really match. Add your own cover as
`cover.jpg` or `folder.jpg` in the album folder or embed it in the file.

## Playback

**It crackles.**
- On the PC: check effects with very strong boosts – the safety limiter catches peaks, but less
  is often more.
- On the phone: update to the latest version (larger audio buffer since 0.5.1).

**Music on the phone stops in the background.**
Turn off battery optimisation: Android settings → Apps → Amber Player → Battery →
**Unrestricted**.

**The equalizer barely changes anything.**
Boosting all bands equally just means “louder” – and Amber Player compensates so nothing clips.
Boost specific ranges or use a preset.

**Amber Player keeps running after closing.**
Only while music plays. Without music Amber Player quits completely; also when the music ends or
after a minute of pause.

## Phone and PC

**The phone can't find the PC.**
Same Wi-Fi? Amber Player running on the PC? **Home network** turned on? Firewall permission
confirmed? Details: [Connecting to your PC](Connecting-to-your-PC.md).

**“The PC refused the connection”.**
The token was regenerated on the PC. Pair again.

**The app is installed twice (Samsung).**
It was also installed in the “Dual Messenger” profile. Uninstall one copy.

**How do I get music from the PC onto the phone without copying it?**
With pairing: **More → PC → Music from the PC** or play a playlist with **Here**.

## Plugins

**My sound plugin doesn't show up.**
Open the **Effects** page – the top lists under “Not loaded” what's wrong. Sound plugins only
take effect after restarting the app.

**My visualizer shows nothing.**
Errors appear as a message at the bottom right. Check that `draw` exists and the file ends in
`.js`. See [Creating a visualizer](Creating-a-Visualizer.md).

**Can a plugin harm my computer?**
Sound plugins can't – they contain no program code. Visualizers run sandboxed. Source plugins
are programs: only install them from trusted sources.

## Remote control

**The API doesn't respond.**
Is Amber Player running? Default address `http://127.0.0.1:7878`. From another device only with
**Home network** and the token.

**Can I change the port?**
Yes, under **Settings → Remote control → Port**. Takes effect after a restart.
