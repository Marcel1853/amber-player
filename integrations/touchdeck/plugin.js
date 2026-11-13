// TouchDeck-Plugin für Amber Player. Spricht die HTTP-API von Amber Player an (Standard: http://127.0.0.1:7878).

const base = String(td.settings.url || 'http://127.0.0.1:7878').trim().replace(/\/+$/, '');
const token = String(td.settings.token || '').trim();
const interval = Math.max(1, Number(td.settings.interval) || 1) * 1000;

function api(path) {
  const headers = token ? { Authorization: `Bearer ${token}` } : {};
  let res;
  try {
    // Amber Player nimmt jede Aktion auch per GET an.
    res = td.http({ method: 'GET', url: base + path, headers });
  } catch (e) {
    throw new Error(`Amber Player nicht erreichbar (${base}) – läuft das Programm?`);
  }
  if (!res.status) throw new Error(`Amber Player nicht erreichbar (${base}) – läuft das Programm?`);
  if (res.status === 401) throw new Error('Token fehlt oder ist falsch – in den Plugin-Einstellungen eintragen');
  let data = null;
  try {
    data = res.json();
  } catch (e) {
    data = null;
  }
  if (!res.ok) throw new Error((data && data.error) || `Amber Player antwortete mit ${res.status}`);
  return data;
}

const enc = encodeURIComponent;

function query(params) {
  const parts = Object.keys(params)
    .filter((k) => params[k] !== undefined && params[k] !== null && params[k] !== '')
    .map((k) => `${k}=${enc(params[k])}`);
  return parts.length ? `?${parts.join('&')}` : '';
}

function fmt(ms) {
  const s = Math.max(0, Math.floor((ms || 0) / 1000));
  return `${Math.floor(s / 60)}:${String(s % 60).padStart(2, '0')}`;
}

function nowText(s) {
  if (!s.track) return 'Nichts läuft';
  return s.track.artist ? `${s.track.title} – ${s.track.artist}` : s.track.title;
}

// ---------- Live-Anzeigen und Auslöser ----------

let lastTrack;
let lastState;
let offline = false;
let eqOn = null;
let eqChecked = 0;

function showOffline() {
  td.live.set('now', { text: 'Amber Player aus', on: false });
  for (const id of ['title', 'artist', 'album', 'sleep']) td.live.set(id, '');
  td.live.set('progress', { text: '', value: 0, on: false });
  td.live.set('state', { text: '⏹', on: false });
  td.live.set('volume', { text: '', value: 0, on: false });
}

function update(s) {
  const t = s.track;
  const playing = s.state === 'playing';
  td.live.set('now', { text: nowText(s), on: playing });
  td.live.set('title', { text: t ? t.title : '', on: playing });
  td.live.set('artist', { text: t ? t.artist : '', on: playing });
  td.live.set('album', { text: t ? t.album : '', on: playing });
  const pct = s.duration_ms ? (s.position_ms / s.duration_ms) * 100 : 0;
  td.live.set('progress', { text: t ? `${fmt(s.position_ms)} / ${fmt(s.duration_ms)}` : '', value: pct, number: s.position_ms / 1000, on: playing });
  td.live.set('state', { text: playing ? '▶' : s.state === 'paused' ? '⏸' : '⏹', on: playing });
  td.live.set('volume', { text: s.muted ? 'Stumm' : `${s.volume} %`, value: s.muted ? 0 : s.volume, number: s.volume, on: !s.muted });
  td.live.set('shuffle', { text: s.shuffle ? 'Zufall an' : 'Zufall aus', on: s.shuffle });
  td.live.set('repeat', { text: { off: 'Aus', all: 'Alle', one: 'Einen' }[s.repeat] || s.repeat, on: s.repeat !== 'off' });
  td.live.set('favorite', { text: t && t.favorite ? '❤️' : '🤍', on: !!(t && t.favorite) });
  td.live.set('sleep', {
    text: s.sleep_remaining_s != null ? `${Math.ceil(s.sleep_remaining_s / 60)} min` : 'Aus',
    on: s.sleep_remaining_s != null,
  });

  const id = t ? t.id : null;
  if (id !== lastTrack) {
    lastTrack = id;
    if (t) td.fire('track', { title: t.title, artist: t.artist, album: t.album });
  }
  if (s.state !== lastState) {
    const first = lastState === undefined;
    lastState = s.state;
    if (!first) td.fire('state', { state: s.state });
  }
}

function updateEq(force) {
  // Effekte ändern sich selten – nur alle 10 Sekunden oder nach einer Aktion abfragen.
  if (!force && Date.now() - eqChecked < 10000) return;
  eqChecked = Date.now();
  const list = api('/api/effects');
  const eq = (list || []).find((f) => f.id === 'equalizer');
  eqOn = !!(eq && eq.enabled);
  td.live.set('eq', { text: eqOn ? 'EQ an' : 'EQ aus', on: eqOn });
}

function poll() {
  try {
    update(api('/api/status'));
    updateEq(false);
    if (offline) td.log('Amber Player wieder erreichbar');
    offline = false;
  } catch (e) {
    if (!offline) td.warn(String(e.message || e));
    offline = true;
    showOffline();
  }
}

td.every(interval, poll);
poll();

/** Aktion ausführen und Anzeigen sofort aktualisieren. */
function act(path) {
  const s = api(path);
  if (s && s.state) update(s);
  return s;
}

// ---------- Aktionen ----------

td.action('toggle', () => {
  const s = act('/api/toggle');
  return s.state === 'playing' ? `▶ ${nowText(s)}` : '⏸ Pause';
});
td.action('play', () => `▶ ${nowText(act('/api/play'))}`);
td.action('pause', () => {
  act('/api/pause');
  return '⏸ Pause';
});
td.action('stop', () => {
  act('/api/stop');
  return '⏹ Gestoppt';
});
td.action('next', () => `⏭ ${nowText(act('/api/next'))}`);
td.action('prev', () => `⏮ ${nowText(act('/api/prev'))}`);

td.action('volume', (p) => {
  let s;
  if (p.mode === 'up') s = act('/api/volume/up');
  else if (p.mode === 'down') s = act('/api/volume/down');
  else s = act(`/api/volume${query({ value: Math.max(0, Math.min(100, Math.round(Number(p.value) || 0))) })}`);
  return `🔊 ${s.volume} %`;
});

td.action('mute', () => (act('/api/mute').muted ? '🔇 Stumm' : '🔊 Ton an'));

td.action('seek', (p) => {
  if (p.mode === 'back') act('/api/seek/back');
  else if (p.mode === 'to') act(`/api/seek${query({ seconds: Math.max(0, Number(p.seconds) || 0) })}`);
  else act('/api/seek/forward');
  return '';
});

td.action('shuffle', (p) => {
  const on = p.mode === 'on' ? 'true' : p.mode === 'off' ? 'false' : '';
  return act(`/api/shuffle${query({ on })}`).shuffle ? '🔀 Zufall an' : 'Zufall aus';
});

td.action('repeat', (p) => {
  const mode = p.mode && p.mode !== 'cycle' ? p.mode : '';
  const s = act(`/api/repeat${query({ mode })}`);
  return `🔁 ${{ off: 'Aus', all: 'Alle', one: 'Diesen Titel' }[s.repeat]}`;
});

td.action('favorite', () => {
  const s = act('/api/favorite');
  return s.track && s.track.favorite ? `❤️ ${s.track.title}` : `🤍 ${s.track ? s.track.title : ''}`;
});

td.action('playlist', (p) => {
  const want = String(p.name || '').trim();
  if (!want) throw new Error('Bitte Name oder ID der Playlist angeben');
  const lists = api('/api/playlists') || [];
  const pl = lists.find((x) => String(x.id) === want) || lists.find((x) => x.name.toLowerCase() === want.toLowerCase());
  if (!pl) throw new Error(`Playlist „${want}“ gibt es nicht`);
  act(`/api/playlists/${pl.id}/play${query({ shuffle: p.shuffle ? 'true' : 'false' })}`);
  return `📃 ${pl.name}`;
});

td.action('music', (p) => {
  const s = act(`/api/play/music${query({
    artist: p.artist, album: p.album, genre: p.genre, search: p.search,
    favorites: p.favorites ? 'true' : '', shuffle: p.shuffle ? 'true' : 'false',
  })}`);
  return `🎵 ${nowText(s)}`;
});

td.action('effect', (p) => {
  const id = p.effect || 'equalizer';
  let path = `/api/effects/${enc(id)}`;
  if (p.mode === 'on') path += '?enabled=true';
  else if (p.mode === 'off') path += '?enabled=false';
  else if (p.mode === 'preset') {
    if (!p.preset) throw new Error('Bitte ein Preset angeben, z. B. Rock');
    path += query({ preset: p.preset });
  } else path += '/toggle';
  const info = api(path);
  updateEq(true);
  return `${info.name}: ${info.enabled ? 'an' : 'aus'}${p.mode === 'preset' ? ` (${p.preset})` : ''}`;
});

td.action('sleep', (p) => {
  const minutes = Math.max(0, Math.round(Number(p.minutes) || 0));
  act(`/api/sleep${minutes ? query({ minutes }) : ''}`);
  return minutes ? `🌙 Pause in ${minutes} Minuten` : '🌙 Sleep-Timer aus';
});

td.action('status', () => {
  const s = api('/api/status');
  const t = s.track || {};
  return {
    title: t.title || '',
    artist: t.artist || '',
    album: t.album || '',
    position: Math.round(s.position_ms / 1000),
    duration: Math.round(s.duration_ms / 1000),
    volume: s.volume,
  };
});

td.log(`Amber-Player-Plugin bereit (${base})`);
