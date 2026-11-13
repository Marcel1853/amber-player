# Internet radio

Amber Player searches the free station database [radio-browser.info](https://www.radio-browser.info)
with more than 50,000 stations from all over the world.

## Find and listen

1. Open **Radio**. Without a search term you see the most popular stations of the selected
   country.
2. Enter a name or genre in the search box, e.g. “bbc”, “rock”, “jazz”.
3. Narrow down with the country selection or choose **All countries**.
4. Tap or click a station – it plays immediately.

While a station plays, Amber Player shows the song title the station sends. In the
[visualizer](Visualizers.md) the station logo appears (e.g. in the centre of the Ring).

## My stations

- Save a station under **My stations** with **★**.
- **Next / Previous** switch between your saved stations while radio is playing.
- To remove: right-click (phone: long-press) → **Remove from “My stations”**.

## Custom address

If a station is missing from the database: **Custom address** and enter the stream address. It
starts with `http://` or `https://`; playlist addresses ending in `.m3u` or `.pls` work too.

## Notes

- Radio streams are only played, never recorded or rebroadcast.
- Some stations send no song titles or logo – then only the station name is shown.
- Sound effects work on radio too.
- Via the API: `/api/radio/search?q=…`, `/api/radio/play/{id}` – see [Remote control and API](Remote-Control-and-API.md).
