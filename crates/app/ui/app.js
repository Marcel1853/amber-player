'use strict';

const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

// ---------- Hilfsfunktionen ----------

const $ = (sel, el = document) => el.querySelector(sel);
const $$ = (sel, el = document) => [...el.querySelectorAll(sel)];

const esc = (s) =>
  String(s ?? '').replace(/[&<>"']/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' })[c]);

// Windows und Android liefern eigene Protokolle über http://<name>.localhost aus.
const COVER_BASE = /windows|android/i.test(navigator.userAgent) ? 'http://cover.localhost/' : 'cover://localhost/';
const isTouch = matchMedia('(pointer: coarse)').matches;
// Am Handy gibt es keine HTTP-API, keinen MCP-Server und keine Handy-Streams (das Handy ist selbst der Empfänger).
const IS_ANDROID = /android/i.test(navigator.userAgent);
// Gleiche Grenze wie im CSS: schmal (Handy, Tablet hochkant) oder flach mit Touch (Handy quer).
const MOBILE_QUERY = '(max-width: 900px), (pointer: coarse) and (max-height: 520px)';
const isMobile = () => matchMedia(MOBILE_QUERY).matches;
const coverUrl = (id) => `${COVER_BASE}${id}`;

function fmtTime(ms) {
  const s = Math.max(0, Math.floor((ms || 0) / 1000));
  const h = Math.floor(s / 3600);
  const m = Math.floor((s % 3600) / 60);
  const sec = String(s % 60).padStart(2, '0');
  return h ? `${h}:${String(m).padStart(2, '0')}:${sec}` : `${m}:${sec}`;
}

function fmtTotal(ms) {
  const min = Math.round((ms || 0) / 60000);
  if (min < 60) return tx('{m} Min.', { m: min });
  return tx('{h} Std. {m} Min.', { h: Math.floor(min / 60), m: min % 60 });
}

// Anzahl mit Wort; übersetzt über die Schlüssel „1|Wort“ (eins) und „n|Wort“ (mehrere).
const plural = (n, one, many) => `${fmtNum(n)} ${n === 1 ? tx(`1|${one}`) : tx(`n|${many}`)}`;

/**
 * Laufschrift: Passt der Text nicht in sein Feld, gleitet er langsam hin und her
 * (mit kurzer Pause an beiden Enden). Kurze Texte stehen still. Gleicher Text = keine Änderung,
 * damit die Bewegung bei Statusmeldungen nicht neu startet.
 */
const marqueeObserver = new ResizeObserver((entries) => entries.forEach((e) => fitMarquee(e.target)));
function setMarquee(el, text) {
  if (!el) return;
  let inner = el.firstElementChild;
  if (!inner || !inner.classList.contains('mq-inner')) {
    el.textContent = '';
    inner = document.createElement('span');
    inner.className = 'mq-inner';
    el.append(inner);
    el.classList.add('mq-box');
    marqueeObserver.observe(el);
  }
  if (inner.textContent === text) return;
  inner.textContent = text;
  fitMarquee(el);
}
function fitMarquee(el) {
  const inner = el.firstElementChild;
  if (!inner) return;
  el.classList.remove('mq');
  const over = inner.scrollWidth - el.clientWidth;
  if (over > 4) {
    el.style.setProperty('--mq-dist', `${-over - 8}px`);
    // ca. 40 Pixel pro Sekunde, mindestens 6 Sekunden für einen Weg
    el.style.setProperty('--mq-time', `${Math.max(6, (over + 8) / 40 + 3)}s`);
    el.classList.add('mq');
  }
}

function debounce(fn, ms) {
  let t;
  return (...a) => {
    clearTimeout(t);
    t = setTimeout(() => fn(...a), ms);
  };
}

/** Ruft `fn` höchstens alle `ms` auf, der letzte Aufruf geht nie verloren. */
function throttle(fn, ms) {
  let last = 0, timer = null, args = null;
  return (...a) => {
    args = a;
    const wait = ms - (Date.now() - last);
    if (wait <= 0) {
      last = Date.now();
      fn(...args);
    } else if (!timer) {
      timer = setTimeout(() => {
        timer = null;
        last = Date.now();
        fn(...args);
      }, wait);
    }
  };
}

/** Farbverlauf aus einem Namen – für Alben/Interpreten ohne Bild. */
function hueGradient(name) {
  let h = 0;
  for (const c of String(name)) h = (h * 31 + c.codePointAt(0)) % 360;
  return `linear-gradient(135deg, hsl(${h} 65% 52%), hsl(${(h + 50) % 360} 70% 38%))`;
}

const initials = (name) =>
  String(name || '?').split(/\s+/).filter(Boolean).slice(0, 2).map((w) => [...w][0].toUpperCase()).join('');

// ---------- Symbole ----------

const ICONS = {
  check: 'M20 6 9 17l-5-5',
  camera: 'M14.5 4h-5L7 7H4a2 2 0 0 0-2 2v9a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2V9a2 2 0 0 0-2-2h-3l-2.5-3zM12 17a4 4 0 1 0 0-8 4 4 0 0 0 0 8z',
  save: 'M15.2 3a2 2 0 0 1 1.4.6l3.8 3.8a2 2 0 0 1 .6 1.4V19a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2zM17 21v-7a1 1 0 0 0-1-1H8a1 1 0 0 0-1 1v7M7 3v4a1 1 0 0 0 1 1h7',
  wave: 'M4 10v4M8 6v12M12 3v18M16 7v10M20 10v4',
  music: 'M9 18V5l12-2v13M6 21a3 3 0 1 0 0-6 3 3 0 0 0 0 6zM18 19a3 3 0 1 0 0-6 3 3 0 0 0 0 6z',
  disc: 'M12 22a10 10 0 1 0 0-20 10 10 0 0 0 0 20zM12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6z',
  user: 'M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2M12 11a4 4 0 1 0 0-8 4 4 0 0 0 0 8z',
  tag: 'M20.6 13.4l-7.2 7.2a2 2 0 0 1-2.8 0L3 13V3h10l7.6 7.6a2 2 0 0 1 0 2.8zM7.5 7.5h.01',
  heart: 'M20.8 4.6a5.5 5.5 0 0 0-7.8 0L12 5.7l-1-1.1a5.5 5.5 0 0 0-7.8 7.8l1 1.1L12 21l7.8-7.5 1-1.1a5.5 5.5 0 0 0 0-7.8z',
  clock: 'M12 22a10 10 0 1 0 0-20 10 10 0 0 0 0 20zM12 6v6l4 2',
  flame: 'M8.5 14.5A2.5 2.5 0 0 0 11 12c0-1.4-.5-2-1-3-1.1-2.1-.2-4 2-6 .5 2.5 2 4.9 4 6.5 2 1.6 3 3.5 3 5.5a7 7 0 1 1-14 0c0-1.2.4-2.3 1-3.2.2 1.3 1.2 2.7 2.5 2.7z',
  plus: 'M12 5v14M5 12h14',
  import: 'M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M7 10l5 5 5-5M12 15V3',
  export: 'M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M17 8l-5-5-5 5M12 3v12',
  sliders: 'M4 21v-7M4 10V3M12 21v-9M12 8V3M20 21v-5M20 12V3M1 14h6M9 8h6M17 16h6',
  settings: 'M12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6zM19.4 15a1.7 1.7 0 0 0 .3 1.8l.1.1a2 2 0 1 1-2.8 2.8l-.1-.1a1.7 1.7 0 0 0-1.8-.3 1.7 1.7 0 0 0-1 1.5V21a2 2 0 1 1-4 0v-.1a1.7 1.7 0 0 0-1.1-1.5 1.7 1.7 0 0 0-1.8.3l-.1.1a2 2 0 1 1-2.8-2.8l.1-.1a1.7 1.7 0 0 0 .3-1.8 1.7 1.7 0 0 0-1.5-1H3a2 2 0 1 1 0-4h.1a1.7 1.7 0 0 0 1.5-1.1 1.7 1.7 0 0 0-.3-1.8l-.1-.1a2 2 0 1 1 2.8-2.8l.1.1a1.7 1.7 0 0 0 1.8.3H9a1.7 1.7 0 0 0 1-1.5V3a2 2 0 1 1 4 0v.1a1.7 1.7 0 0 0 1 1.5 1.7 1.7 0 0 0 1.8-.3l.1-.1a2 2 0 1 1 2.8 2.8l-.1.1a1.7 1.7 0 0 0-.3 1.8V9a1.7 1.7 0 0 0 1.5 1H21a2 2 0 1 1 0 4h-.1a1.7 1.7 0 0 0-1.5 1z',
  'chevron-left': 'M15 18l-6-6 6-6',
  'chevron-right': 'M9 18l6-6-6-6',
  'chevron-down': 'M6 9l6 6 6-6',
  search: 'M11 19a8 8 0 1 0 0-16 8 8 0 0 0 0 16zM21 21l-4.35-4.35',
  x: 'M18 6L6 18M6 6l12 12',
  shuffle: 'M16 3h5v5M4 20L21 3M21 16v5h-5M15 15l6 6M4 4l5 5',
  repeat: 'M17 1l4 4-4 4M3 11V9a4 4 0 0 1 4-4h14M7 23l-4-4 4-4M21 13v2a4 4 0 0 1-4 4H3',
  play: 'M7 4.5v15a1 1 0 0 0 1.5.86l12.5-7.5a1 1 0 0 0 0-1.72L8.5 3.64A1 1 0 0 0 7 4.5z',
  pause: 'M7 4h3a1 1 0 0 1 1 1v14a1 1 0 0 1-1 1H7a1 1 0 0 1-1-1V5a1 1 0 0 1 1-1zM14 4h3a1 1 0 0 1 1 1v14a1 1 0 0 1-1 1h-3a1 1 0 0 1-1-1V5a1 1 0 0 1 1-1z',
  next: 'M5 5.5v13a1 1 0 0 0 1.6.8L15 13v5.5a.5.5 0 0 0 .5.5h2a.5.5 0 0 0 .5-.5v-13a.5.5 0 0 0-.5-.5h-2a.5.5 0 0 0-.5.5V11L6.6 4.7A1 1 0 0 0 5 5.5z',
  prev: 'M19 5.5v13a1 1 0 0 1-1.6.8L9 13v5.5a.5.5 0 0 1-.5.5h-2a.5.5 0 0 1-.5-.5v-13a.5.5 0 0 1 .5-.5h2a.5.5 0 0 1 .5.5V11l8.4-6.3A1 1 0 0 1 19 5.5z',
  moon: 'M21 12.8A9 9 0 1 1 11.2 3a7 7 0 0 0 9.8 9.8z',
  list: 'M8 6h13M8 12h13M8 18h13M3 6h.01M3 12h.01M3 18h.01',
  volume: 'M11 5L6 9H2v6h4l5 4V5zM15.5 8.5a5 5 0 0 1 0 7M19 5a10 10 0 0 1 0 14',
  'volume-low': 'M11 5L6 9H2v6h4l5 4V5zM15.5 8.5a5 5 0 0 1 0 7',
  mute: 'M11 5L6 9H2v6h4l5 4V5zM23 9l-6 6M17 9l6 6',
  folder: 'M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z',
  trash: 'M3 6h18M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6M10 11v6M14 11v6M9 6V4a1 1 0 0 1 1-1h4a1 1 0 0 1 1 1v2',
  edit: 'M12 20h9M16.5 3.5a2.1 2.1 0 0 1 3 3L7 19l-4 1 1-4z',
  copy: 'M20 9h-9a2 2 0 0 0-2 2v9a2 2 0 0 0 2 2h9a2 2 0 0 0 2-2v-9a2 2 0 0 0-2-2zM5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1',
  refresh: 'M23 4v6h-6M1 20v-6h6M3.5 9a9 9 0 0 1 14.9-3.4L23 10M1 14l4.6 4.4A9 9 0 0 0 20.5 15',
  playlist: 'M3 6h13M3 12h13M3 18h9M17 18a2 2 0 1 0 4 0V9h2',
  'play-next': 'M3 6h11M3 12h11M3 18h7M17 13l4 4-4 4',
  'queue-add': 'M3 6h12M3 12h12M3 18h8M18 15v6M15 18h6',
  arrow: 'M5 12h14M12 5l7 7-7 7',
  activity: 'M22 12h-4l-3 9L9 3l-3 9H2',
  radio: 'M4.9 19.1a10 10 0 0 1 0-14.2M7.8 16.2a6 6 0 0 1 0-8.4M16.2 7.8a6 6 0 0 1 0 8.4M19.1 4.9a10 10 0 0 1 0 14.2M12 13a1 1 0 1 0 0-2 1 1 0 0 0 0 2z',
  star: 'M12 2l3.1 6.3 6.9 1-5 4.9 1.2 6.8-6.2-3.2-6.2 3.2 1.2-6.8-5-4.9 6.9-1z',
  phone: 'M7 2h10a2 2 0 0 1 2 2v16a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2zM11 18h2',
  link: 'M10 14a5 5 0 0 0 7 0l3-3a5 5 0 0 0-7-7l-1 1M14 10a5 5 0 0 0-7 0l-3 3a5 5 0 0 0 7 7l1-1',
  monitor: 'M3 4h18a1 1 0 0 1 1 1v11a1 1 0 0 1-1 1H3a1 1 0 0 1-1-1V5a1 1 0 0 1 1-1zM8 21h8M12 17v4',
  maximize: 'M8 3H5a2 2 0 0 0-2 2v3M21 8V5a2 2 0 0 0-2-2h-3M3 16v3a2 2 0 0 0 2 2h3M16 21h3a2 2 0 0 0 2-2v-3',
  puzzle: 'M19.4 13.4a2.5 2.5 0 1 0 0-2.8V7a1 1 0 0 0-1-1h-3.6a2.5 2.5 0 1 0-2.8 0H8.4a1 1 0 0 0-1 1v3.6a2.5 2.5 0 1 1 0 2.8V17a1 1 0 0 0 1 1h3.6a2.5 2.5 0 1 1 2.8 0h3.6a1 1 0 0 0 1-1z',
  remote: 'M8 2h8a2 2 0 0 1 2 2v16a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2zM12 18h.01M12 6a2 2 0 1 0 0 4 2 2 0 0 0 0-4z',
};

function icon(name, fill = false) {
  return `<svg class="i${fill ? ' fill' : ''}" viewBox="0 0 24 24" aria-hidden="true"><path d="${ICONS[name] || ''}"/></svg>`;
}

function hydrateIcons(root = document) {
  for (const el of $$('[data-icon]', root)) {
    el.outerHTML = icon(el.dataset.icon, el.hasAttribute('data-fill'));
  }
}

const artHtml = (track, cls = 'thumb') =>
  track && track.has_cover
    ? `<img class="${cls}" src="${coverUrl(track.id)}" loading="lazy" alt="" draggable="false">`
    : `<div class="${cls} ph">${icon('music')}</div>`;

// ---------- Akzentfarbe ----------

const ACCENTS = [
  ['#f2a03d', 'Bernstein'], ['#ff7a59', 'Koralle'], ['#e5487a', 'Himbeere'], ['#a37bff', 'Violett'],
  ['#4f9dff', 'Blau'], ['#2ec4b6', 'Türkis'], ['#b5d334', 'Limette'], ['#e8e2d6', 'Elfenbein'],
];

function hexToHsl(hex) {
  const n = parseInt(hex.slice(1), 16);
  const r = ((n >> 16) & 255) / 255, g = ((n >> 8) & 255) / 255, b = (n & 255) / 255;
  const max = Math.max(r, g, b), min = Math.min(r, g, b);
  let h = 0, s = 0;
  const l = (max + min) / 2;
  if (max !== min) {
    const d = max - min;
    s = l > 0.5 ? d / (2 - max - min) : d / (max + min);
    h = max === r ? (g - b) / d + (g < b ? 6 : 0) : max === g ? (b - r) / d + 2 : (r - g) / d + 4;
    h *= 60;
  }
  return [h, s * 100, l * 100];
}

/** Setzt die Akzentfarbe; zweite Verlaufsfarbe, gedämpfte Variante und Schriftfarbe werden abgeleitet. */
function applyAccent(hex = localStorage.getItem('accent') || ACCENTS[0][0]) {
  if (!/^#[0-9a-f]{6}$/i.test(hex)) hex = ACCENTS[0][0];
  const [h, s, l] = hexToHsl(hex);
  const n = parseInt(hex.slice(1), 16);
  const lum = (0.2126 * ((n >> 16) & 255) + 0.7152 * ((n >> 8) & 255) + 0.0722 * (n & 255)) / 255;
  const root = document.documentElement.style;
  root.setProperty('--accent', hex);
  root.setProperty('--accent-2', `hsl(${(h + 330) % 360} ${Math.min(90, s + 5)}% ${Math.max(35, Math.min(62, l - 4))}%)`);
  root.setProperty('--accent-soft', `hsl(${h} ${s}% ${l}% / 0.16)`);
  root.setProperty('--accent-ink', lum > 0.55 ? '#1b130a' : '#ffffff');
  if (typeof viz !== 'undefined' && viz.worker) vizInfo(true);
}

// ---------- Zustand ----------

const state = {
  status: null,
  statusAt: performance.now(),
  view: { name: 'tracks' },
  back: [],
  forward: [],
  playlists: [],
  list: null, // aktuelle Titelliste (TrackList)
  sort: localStorage.getItem('sort') || 'artist',
  renderId: 0,
  seeking: false,
};

// ---------- Meldungen und Dialoge ----------

function toast(msg, type = '') {
  const el = document.createElement('div');
  el.className = `toast ${type}`;
  el.textContent = msg;
  $('#toasts').append(el);
  setTimeout(() => el.remove(), type === 'err' ? 6000 : 3200);
}

async function call(cmd, args) {
  try {
    return await invoke(cmd, args);
  } catch (e) {
    toast(String(e), 'err');
    throw e;
  }
}

function dialog({ title, text = '', input = null, ok = 'OK', danger = false }) {
  return new Promise((resolve) => {
    const modal = $('#modal');
    modal.innerHTML = `
      <div class="dialog" role="dialog">
        <h3>${esc(title)}</h3>
        ${text ? `<p>${esc(text)}</p>` : ''}
        ${input !== null ? `<input id="dlgInput" value="${esc(input)}" spellcheck="false">` : ''}
        <div class="actions">
          <button class="btn" data-r="cancel">Abbrechen</button>
          <button class="btn ${danger ? 'danger' : 'primary'}" data-r="ok">${esc(ok)}</button>
        </div>
      </div>`;
    modal.classList.add('open');
    const inp = $('#dlgInput');
    if (inp) {
      inp.focus();
      inp.select();
    }
    const done = (val) => {
      modal.classList.remove('open');
      modal.innerHTML = '';
      document.removeEventListener('keydown', onKey, true);
      resolve(val);
    };
    const submit = () => done(inp ? inp.value.trim() || null : true);
    const onKey = (e) => {
      if (e.key === 'Escape') { e.stopPropagation(); done(null); }
      if (e.key === 'Enter') { e.stopPropagation(); e.preventDefault(); submit(); }
    };
    document.addEventListener('keydown', onKey, true);
    modal.onclick = (e) => {
      const r = e.target.closest('[data-r]')?.dataset.r;
      if (r === 'ok') submit();
      else if (r === 'cancel' || e.target === modal) done(null);
    };
  });
}

// ---------- Kontextmenü ----------

let openMenu = null;

function closeMenu() {
  openMenu?.remove();
  openMenu = null;
}

/** items: [{ label, icon, action, danger, sub: items }] oder 'sep' */
function buildMenu(items, sheet = false) {
  const menu = document.createElement('div');
  menu.className = sheet ? 'menu sheet' : 'menu';
  for (const it of items) {
    if (it === 'sep') {
      menu.append(document.createElement('hr'));
      continue;
    }
    const b = document.createElement('button');
    if (it.danger) b.className = 'danger';
    b.innerHTML = `${it.icon ? icon(it.icon) : ''}<span>${esc(it.label)}</span>${it.sub ? `<span class="sub-arrow">${icon('chevron-right')}</span>` : ''}`;
    if (it.sub && sheet) {
      // Touch: Untermenü ersetzt die Leiste, mit „Zurück“ an erster Stelle.
      b.addEventListener('click', (e) => {
        e.stopPropagation();
        showMenu(0, 0, [{ label: 'Zurück', icon: 'chevron-left', action: () => showMenu(0, 0, items) }, 'sep', ...it.sub]);
      });
    } else if (it.sub) {
      b.addEventListener('mouseenter', () => {
        $$('.menu', menu).forEach((m) => m.remove());
        $$('button.open', menu).forEach((x) => x.classList.remove('open'));
        b.classList.add('open');
        const sub = buildMenu(it.sub);
        b.append(sub);
        const r = sub.getBoundingClientRect();
        if (r.right > innerWidth) { sub.style.left = 'auto'; sub.style.right = '100%'; }
        if (r.bottom > innerHeight) sub.style.top = `${innerHeight - r.bottom - 6}px`;
      });
      b.addEventListener('click', (e) => e.stopPropagation());
    } else {
      b.addEventListener('mouseenter', () => {
        $$(':scope > button.open', menu).forEach((x) => { x.classList.remove('open'); $$('.menu', x).forEach((m) => m.remove()); });
      });
      b.addEventListener('click', (e) => {
        e.stopPropagation();
        closeMenu();
        it.action?.();
      });
    }
    menu.append(b);
  }
  return menu;
}

function showMenu(x, y, items) {
  closeMenu();
  const sheet = isMobile();
  const menu = buildMenu(items, sheet);
  document.body.append(menu);
  if (sheet) {
    // Handy: Leiste von unten über die ganze Breite, gut mit dem Daumen erreichbar.
    openMenu = menu;
    return;
  }
  const r = menu.getBoundingClientRect();
  menu.style.left = `${Math.min(x, innerWidth - r.width - 8)}px`;
  menu.style.top = `${Math.min(y, innerHeight - r.height - 8)}px`;
  openMenu = menu;
}

document.addEventListener('click', closeMenu);
document.addEventListener('contextmenu', (e) => {
  if (!e.target.closest('input')) e.preventDefault();
});
window.addEventListener('blur', closeMenu);

// ---------- Wiedergabe ----------

const send = (cmd, extra = {}) => call('player_command', { cmd: { cmd, ...extra } }).then(applyStatus).catch(() => {});

function playList(tracks, start = 0, shuffle = undefined) {
  if (!tracks.length) return;
  return call('play_ids', { ids: tracks.map((t) => t.id), start, shuffle: shuffle ?? null }).then(applyStatus).catch(() => {});
}

function shufflePlay(tracks) {
  if (!tracks.length) return;
  return playList(tracks, Math.floor(Math.random() * tracks.length), true);
}

async function toggleFavorite(track) {
  const fav = !track.favorite;
  await call('set_favorite', { id: track.id, favorite: fav });
  track.favorite = fav;
  for (const t of state.list?.tracks || []) if (t.id === track.id) t.favorite = fav;
  state.list?.draw(true);
  if (state.view.name === 'favorites') render();
  toast(fav ? 'Zu Favoriten hinzugefügt' : 'Aus Favoriten entfernt', 'ok');
}

function currentPosition() {
  const s = state.status;
  if (!s) return 0;
  let pos = s.position_ms;
  if (s.state === 'playing') pos += performance.now() - state.statusAt;
  return s.duration_ms ? Math.min(pos, s.duration_ms) : pos;
}

let lastTrackId;
let lastError;

function applyStatus(s) {
  if (!s) return;
  const prev = state.status;
  state.status = s;
  state.statusAt = performance.now();

  const t = s.track;
  setMarquee($('#barTitle'), t ? t.title : tx('Nichts ausgewählt'));
  $('#barArtist').textContent = t ? t.artist || tx('Unbekannter Interpret') : '';
  $('#barAlbum').textContent = t ? [t.album, t.year].filter(Boolean).join(' · ') : tx('Wähle einen Titel aus der Bibliothek');
  const fav = $('#barFav');
  fav.classList.add('fav');
  fav.classList.toggle('on', !!t?.favorite);
  fav.innerHTML = icon('heart', !!t?.favorite);

  const r = !t ? s.radio : null;
  if (r) {
    setMarquee($('#barTitle'), r.title || r.station.name);
    $('#barArtist').textContent = r.station.name;
    $('#barAlbum').textContent = [tx('Internetradio'), r.station.codec, r.station.bitrate ? `${r.station.bitrate} kbps` : ''].filter(Boolean).join(' · ');
  }
  fav.style.visibility = r ? 'hidden' : 'visible';

  const coverKey = t ? `t${t.id}:${t.has_cover}` : r ? `r${r.station.url}` : '';
  if (coverKey !== lastTrackId) {
    lastTrackId = coverKey;
    if (r) {
      $('#barCover').innerHTML = stationLogo(r.station);
      $('#deckGlow').style.backgroundImage = 'none';
    } else {
      $('#barCover').innerHTML = t?.has_cover
        ? `<img src="${coverUrl(t.id)}" alt="" draggable="false">`
        : `<div class="ph">${icon('music')}</div>`;
      $('#deckGlow').style.backgroundImage = t?.has_cover ? `url("${coverUrl(t.id)}")` : 'none';
    }
  }
  document.title = t ? `${t.title} – ${t.artist || 'Amber Player'}` : r ? `${r.title || r.station.name} – ${r.station.name}` : 'Amber Player';
  const radioUrl = r?.station.url;
  if (t?.id !== prev?.track?.id || radioUrl !== (prev?.track ? null : prev?.radio?.station.url) || s.queue_length !== prev?.queue_length || s.shuffle !== prev?.shuffle) renderUpNext();
  if (state.view.name === 'radio') markRadio();

  const playing = s.state === 'playing';
  $('#play').innerHTML = icon(playing ? 'pause' : 'play', true);
  $('#play').title = playing ? 'Pause (Leertaste)' : 'Abspielen (Leertaste)';
  document.body.classList.toggle('playing', playing);

  $('#shuffle').classList.toggle('on', s.shuffle);
  const rep = $('#repeat');
  rep.classList.toggle('on', s.repeat !== 'off');
  rep.innerHTML = icon('repeat') + (s.repeat === 'one' ? '<span class="badge">1</span>' : '');
  rep.title = { off: 'Wiederholen: aus', all: 'Wiederholen: alle', one: 'Wiederholen: diesen Titel' }[s.repeat];

  const vol = $('#volume');
  if (document.activeElement !== vol) setRange(vol, s.volume);
  $('#mute').innerHTML = icon(s.muted || s.volume === 0 ? 'mute' : s.volume < 50 ? 'volume-low' : 'volume');

  const sl = $('#sleep');
  sl.classList.toggle('on', s.sleep_remaining_s != null);
  sl.innerHTML = icon('moon') + (s.sleep_remaining_s != null ? `<span class="sleep-min">${Math.ceil(s.sleep_remaining_s / 60)}</span>` : '');

  $('#dur').textContent = r ? 'LIVE' : fmtTime(s.duration_ms);
  $('#seek').disabled = !t;

  if (s.error && s.error !== lastError) toast(s.error, 'err');
  lastError = s.error;

  if (!prev || prev.track?.id !== t?.id || prev.state !== s.state) state.list?.refresh();
  if (viz.open) {
    vizInfo();
    $('#vizPlay').innerHTML = icon(playing ? 'pause' : 'play', true);
  }
  androidMedia.update(s, prev);
}

function setRange(el, value) {
  el.value = value;
  const pct = ((value - el.min) / (el.max - el.min)) * 100;
  el.style.setProperty('--pct', `${pct}%`);
}

function tickProgress() {
  const s = state.status;
  if (s && !state.seeking) {
    const pos = currentPosition();
    $('#pos').textContent = fmtTime(pos);
    setRange($('#seek'), s.duration_ms ? Math.round((pos / s.duration_ms) * 1000) : 0);
  }
  requestAnimationFrame(tickProgress);
}

// ---------- Titelliste (virtuell) ----------

const ROW = 52;

class TrackList {
  /**
   * opts: { playlistId, showAlbum = true, numbering: 'index' | 'track' }
   */
  constructor(container, tracks, opts = {}) {
    this.tracks = tracks;
    this.opts = { showAlbum: true, numbering: 'index', ...opts };
    this.selected = new Set();
    this.anchor = null;
    this.el = document.createElement('div');
    this.el.className = `tracklist${this.opts.showAlbum ? '' : ' no-album'}`;
    this.el.innerHTML = `
      <div class="tl-head">
        <div class="c-num">#</div><div>Titel</div><div class="c-album">Album</div><div></div><div class="c-dur">${icon('clock')}</div>
      </div>
      <div class="tl-scroll"><div class="tl-rows"></div></div>`;
    $('.c-dur svg', this.el).style.cssText = 'width:15px;height:15px';
    this.scroller = $('.tl-scroll', this.el);
    this.rows = $('.tl-rows', this.el);
    container.append(this.el);
    this.rows.style.height = `${tracks.length * ROW}px`;
    this.scroller.addEventListener('scroll', () => this.draw());
    new ResizeObserver(() => this.draw()).observe(this.scroller);
    this.bind();
    this.draw();
  }

  /** Markierung und laufenden Titel aktualisieren, ohne die Zeilen neu aufzubauen
   *  (sonst erkennt der Browser keinen Doppelklick mehr). */
  refresh() {
    const cur = state.status?.track?.id;
    for (const r of this.rows.children) {
      const i = Number(r.dataset.i);
      r.classList.toggle('selected', this.selected.has(i));
      r.classList.toggle('playing', this.tracks[i]?.id === cur);
    }
  }

  draw(force = false) {
    const top = this.scroller.scrollTop;
    const h = this.scroller.clientHeight || 800;
    const first = Math.max(0, Math.floor(top / ROW) - 8);
    const last = Math.min(this.tracks.length, Math.ceil((top + h) / ROW) + 8);
    if (!force && first === this.first && last === this.last) return;
    this.first = first;
    this.last = last;
    const cur = state.status?.track?.id;
    let html = '';
    for (let i = first; i < last; i++) {
      const t = this.tracks[i];
      const num = this.opts.numbering === 'track' ? t.track_no || '' : i + 1;
      const cls = `row${t.id === cur ? ' playing' : ''}${this.selected.has(i) ? ' selected' : ''}`;
      html += `<div class="${cls}" data-i="${i}" style="top:${i * ROW}px" draggable="${!isTouch}">
        <div class="c-num"><span class="n">${num}</span><span class="play-ico">${icon('play', true)}</span><span class="bars"><span></span><span></span><span></span></span></div>
        <div class="c-title">${artHtml(t)}<div class="txt"><div class="t">${esc(t.title)}</div><div class="a">${esc(t.artist || tx('Unbekannter Interpret'))}</div></div></div>
        <div class="c-album">${esc(t.album)}</div>
        <div><button class="fav-btn${t.favorite ? ' on' : ''}" data-fav title="Favorit">${icon('heart')}</button></div>
        <div class="c-dur">${fmtTime(t.duration_ms)}</div>
      </div>`;
    }
    this.rows.innerHTML = html;
    for (const svg of $$('.play-ico svg', this.rows)) svg.style.cssText = 'width:15px;height:15px';
  }

  indexOf(e) {
    const row = e.target.closest('.row');
    return row ? Number(row.dataset.i) : -1;
  }

  selectedTracks() {
    return [...this.selected].sort((a, b) => a - b).map((i) => this.tracks[i]);
  }

  select(i, e) {
    if (e?.shiftKey && this.anchor !== null) {
      const [a, b] = [Math.min(this.anchor, i), Math.max(this.anchor, i)];
      if (!e.ctrlKey) this.selected.clear();
      for (let k = a; k <= b; k++) this.selected.add(k);
    } else if (e?.ctrlKey || e?.metaKey) {
      this.selected.has(i) ? this.selected.delete(i) : this.selected.add(i);
      this.anchor = i;
    } else {
      this.selected = new Set([i]);
      this.anchor = i;
    }
    this.refresh();
  }

  selectAll() {
    this.selected = new Set(this.tracks.map((_, i) => i));
    this.refresh();
  }

  bind() {
    this.rows.addEventListener('click', (e) => {
      const i = this.indexOf(e);
      if (i < 0) return;
      if (e.target.closest('[data-fav]')) {
        e.stopPropagation();
        toggleFavorite(this.tracks[i]);
        return;
      }
      if (e.target.closest('.play-ico') || isTouch) {
        playList(this.tracks, i);
        return;
      }
      this.select(i, e);
    });
    this.rows.addEventListener('dblclick', (e) => {
      const i = this.indexOf(e);
      if (i >= 0 && !e.target.closest('[data-fav]')) playList(this.tracks, i);
    });
    this.rows.addEventListener('contextmenu', (e) => {
      const i = this.indexOf(e);
      if (i < 0) return;
      if (!this.selected.has(i)) this.select(i);
      trackMenu(e.clientX, e.clientY, this.selectedTracks(), this);
    });

    // Ziehen: in Playlisten der Seitenleiste oder innerhalb einer Playlist umsortieren
    this.rows.addEventListener('dragstart', (e) => {
      const i = this.indexOf(e);
      if (i < 0) return;
      if (!this.selected.has(i)) this.select(i);
      const tracks = this.selectedTracks();
      dragData = { ids: tracks.map((t) => t.id), from: this.selected.size === 1 ? i : null, list: this };
      e.dataTransfer.effectAllowed = 'copyMove';
      e.dataTransfer.setData('text/plain', tracks.map((t) => t.title).join('\n'));
      const ghost = document.createElement('div');
      ghost.className = 'drag-ghost';
      ghost.textContent = tracks.length === 1 ? tracks[0].title : plural(tracks.length, 'Titel', 'Titel');
      document.body.append(ghost);
      e.dataTransfer.setDragImage(ghost, 10, 10);
      showDropTray(true);
      setTimeout(() => ghost.remove(), 0);
    });
    this.rows.addEventListener('dragend', () => {
      dragData = null;
      showDropTray(false);
      $$('.drop-before, .drop-after', this.rows).forEach((r) => r.classList.remove('drop-before', 'drop-after'));
    });
    if (this.opts.playlistId != null) {
      this.rows.addEventListener('dragover', (e) => {
        if (!dragData || dragData.list !== this || dragData.from == null) return;
        const row = e.target.closest('.row');
        if (!row) return;
        e.preventDefault();
        $$('.drop-before, .drop-after', this.rows).forEach((r) => r.classList.remove('drop-before', 'drop-after'));
        const after = e.offsetY > ROW / 2 && e.target === row;
        row.classList.add(after ? 'drop-after' : 'drop-before');
      });
      this.rows.addEventListener('drop', async (e) => {
        if (!dragData || dragData.list !== this || dragData.from == null) return;
        e.preventDefault();
        const row = e.target.closest('.row');
        if (!row) return;
        let to = Number(row.dataset.i);
        if (row.classList.contains('drop-after')) to += 1;
        const from = dragData.from;
        if (to > from) to -= 1;
        if (to === from) return;
        await call('move_in_playlist', { id: this.opts.playlistId, from, to });
        render();
      });
    }
  }
}

let dragData = null;

function trackMenu(x, y, tracks, list) {
  const one = tracks.length === 1 ? tracks[0] : null;
  const ids = tracks.map((t) => t.id);
  const plItems = [
    { label: 'Neue Playlist …', icon: 'plus', action: () => newPlaylist(ids) },
    ...(state.playlists.length ? ['sep'] : []),
    ...state.playlists.map((p) => ({
      label: p.name,
      icon: 'playlist',
      action: async () => {
        await call('add_to_playlist', { id: p.id, trackIds: ids });
        toast(tx('{n} zu „{name}“ hinzugefügt', { n: plural(ids.length, 'Titel', 'Titel'), name: p.name }), 'ok');
        loadPlaylists();
      },
    })),
  ];
  const items = [
    { label: 'Abspielen', icon: 'play', action: () => (one && list ? playList(list.tracks, list.tracks.indexOf(one)) : playList(tracks)) },
    {
      label: 'Als Nächstes spielen',
      icon: 'play-next',
      action: async () => {
        for (const id of [...ids].reverse()) await send('play_next', { id });
        toast('Wird als Nächstes gespielt', 'ok');
      },
    },
    {
      label: 'Zur Warteschlange',
      icon: 'queue-add',
      action: async () => {
        for (const id of ids) await send('enqueue', { id });
        toast(tx('{n} eingereiht', { n: plural(ids.length, 'Titel', 'Titel') }), 'ok');
      },
    },
    { label: 'Zu Playlist hinzufügen', icon: 'playlist', sub: plItems },
    'sep',
  ];
  if (one) {
    items.push({ label: one.favorite ? 'Aus Favoriten entfernen' : 'Zu Favoriten', icon: 'heart', action: () => toggleFavorite(one) });
    if (one.album) items.push({ label: 'Zum Album', icon: 'disc', action: () => navigate({ name: 'album', album: one.album, artist: one.album_artist || one.artist }) });
    if (one.artist) items.push({ label: 'Zum Interpreten', icon: 'user', action: () => navigate({ name: 'artist', artist: one.album_artist || one.artist }) });
  } else {
    items.push({
      label: 'Alle zu Favoriten',
      icon: 'heart',
      action: async () => {
        for (const t of tracks) await call('set_favorite', { id: t.id, favorite: true });
        tracks.forEach((t) => (t.favorite = true));
        list?.draw(true);
        toast('Zu Favoriten hinzugefügt', 'ok');
      },
    });
  }
  if (list?.opts.playlistId != null) {
    items.push('sep', {
      label: 'Aus Playlist entfernen',
      icon: 'trash',
      danger: true,
      action: () => removeFromPlaylist(list),
    });
  }
  showMenu(x, y, items);
}

async function removeFromPlaylist(list) {
  const positions = [...list.selected].sort((a, b) => b - a);
  for (const position of positions) {
    await call('remove_from_playlist', { id: list.opts.playlistId, position });
  }
  toast(tx('{n} entfernt', { n: plural(positions.length, 'Titel', 'Titel') }), 'ok');
  loadPlaylists();
  render();
}

// ---------- Navigation ----------

function sameView(a, b) {
  return JSON.stringify(a) === JSON.stringify(b);
}

function navigate(view, { push = true } = {}) {
  if (push && !sameView(view, state.view)) {
    state.back.push(state.view);
    state.forward = [];
  }
  state.view = view;
  closeDeck();
  if (view.name !== 'search') $('#search').value = '';
  render();
}

function goBack() {
  if (!state.back.length) return;
  state.forward.push(state.view);
  state.view = state.back.pop();
  render();
}

function goForward() {
  if (!state.forward.length) return;
  state.back.push(state.view);
  state.view = state.forward.pop();
  render();
}

function updateNav() {
  const v = state.view;
  const parent = { album: 'albums', artist: 'artists', genre: 'genres', playlist: 'playlists', most: 'recent' }[v.name] || v.name;
  for (const b of $$('[data-view], [data-nav]')) b.classList.toggle('active', (b.dataset.view || b.dataset.nav) === parent);
  $('#back').disabled = !state.back.length;
  $('#forward').disabled = !state.forward.length;
}

// ---------- Ansichten ----------

const viewEl = () => $('#view');

function head({ kind, title, sub = '', art = '', round = false, actions = '' }) {
  return `<div class="view-head${art ? ' big' : ''}">
    ${art ? `<div class="art${round ? ' round' : ''}">${art}</div>` : ''}
    <div class="meta2">
      ${kind ? `<div class="kind">${esc(kind)}</div>` : ''}
      <h1>${esc(title)}</h1>
      ${sub ? `<div class="sub">${sub}</div>` : ''}
      ${actions ? `<div class="actions">${actions}</div>` : ''}
    </div>
  </div>`;
}

const playButtons = `
  <button class="btn primary" data-act="play">${icon('play', true)} Abspielen</button>
  <button class="btn" data-act="shuffle">${icon('shuffle')} Zufällig</button>`;

function emptyState({ ico = 'music', title, text, actions = '' }) {
  return `<div class="empty"><div class="big-ico">${icon(ico)}</div><h2>${esc(title)}</h2><p>${esc(text)}</p>${actions ? `<div class="actions">${actions}</div>` : ''}</div>`;
}

async function render() {
  const id = ++state.renderId;
  const v = state.view;
  closeMenu();
  updateNav();
  state.list = null;
  const el = viewEl();
  const fresh = () => id === state.renderId;

  const renderers = {
    tracks: renderTracks,
    favorites: renderTracks,
    recent: renderTracks,
    most: renderTracks,
    search: renderTracks,
    albums: renderAlbums,
    album: renderTracks,
    artists: renderArtists,
    artist: renderTracks,
    genres: renderGenres,
    genre: renderTracks,
    playlist: renderPlaylist,
    playlists: renderPlaylists,
    radio: renderRadio,
    pc: renderPc,
    online: renderOnline,
    effects: renderEffects,
    settings: renderSettings,
  };
  try {
    await (renderers[v.name] || renderTracks)(el, v, fresh);
  } catch (e) {
    if (fresh()) el.innerHTML = emptyState({ ico: 'x', title: 'Fehler', text: String(e) });
  }
}

/**
 * Bereiche und Einstellungen, die die Suche zusätzlich findet: [Name, Symbol, Ziel, Suchwörter].
 * Suchwörter auf Deutsch und Englisch; der übersetzte Name zählt immer mit.
 */
const JUMPS = [
  ['PC', 'monitor', { name: 'pc' }, 'pc computer koppeln verbinden pairing connect fernsteuerung remote qr streamen', () => IS_ANDROID],
  ['Effekte', 'sliders', { name: 'effects' }, 'effekte klang equalizer eq hall reverb bass ton tone balance mono stereo limiter echo effects sound'],
  ['Einstellungen', 'settings', { name: 'settings' }, 'einstellungen settings optionen options'],
  ['Musikordner', 'folder', { name: 'settings' }, 'musikordner ordner bibliothek einlesen scan folder library'],
  ['Sprache', 'settings', { name: 'settings' }, 'sprache language langue idioma'],
  ['Leiste unten', 'list', { name: 'settings', focus: 'nav' }, 'leiste navigation menü bar tabs anpassen', () => IS_ANDROID || isMobile()],
  ['Plugins', 'puzzle', { name: 'settings' }, 'plugins plugin paket package installieren install erweiterte'],
  ['Fernsteuerung', 'remote', { name: 'settings' }, 'fernsteuerung api mcp stream deck touchdeck token heimnetz handy koppeln', () => !IS_ANDROID],
  ['Radio', 'radio', { name: 'radio' }, 'radio sender station internetradio'],
  ['Playlisten', 'playlist', { name: 'playlists' }, 'playlisten playlist liste'],
  ['Favoriten', 'heart', { name: 'favorites' }, 'favoriten favorites herz'],
  ['Verlauf', 'clock', { name: 'recent' }, 'verlauf history zuletzt meistgespielt'],
];

function searchJumps(q) {
  const t = q.trim().toLowerCase();
  if (t.length < 2) return '';
  const hits = JUMPS.filter(([label, , , words, ok]) => (!ok || ok()) && `${words} ${label.toLowerCase()} ${tx(label).toLowerCase()}`.split(/\s+/).some((w) => w.startsWith(t)));
  if (!hits.length) return '';
  return `<div class="jumps"><span class="k">${esc(tx('Gehe zu'))}</span>${hits
    .map(([label, ic], i) => `<button class="chip-btn" data-jump="${JUMPS.indexOf(hits[i])}">${icon(ic)}${esc(tx(label))}</button>`)
    .join('')}</div>`;
}

function addJumps(el, q) {
  const html = searchJumps(q);
  if (!html) return;
  $('.view-head', el)?.insertAdjacentHTML('afterend', html);
  for (const b of $$('[data-jump]', el)) b.addEventListener('click', () => navigate({ ...JUMPS[Number(b.dataset.jump)][2] }));
}

/** Titel-Ansichten: alle, Favoriten, Verlauf, Suche, Album, Interpret, Genre. */
async function renderTracks(el, v, fresh) {
  const cfg = {
    tracks: { kind: 'Bibliothek', title: 'Titel', query: { sort: state.sort }, sortable: true },
    favorites: { kind: 'Für dich', title: 'Favoriten', query: { favorites: true, sort: 'artist' } },
    recent: { kind: 'Verlauf', title: 'Zuletzt gespielt', query: { sort: 'recent', limit: 300 }, history: true },
    most: { kind: 'Verlauf', title: 'Meistgespielt', query: { sort: 'most_played', limit: 300 }, history: true },
    search: { kind: 'Suche', title: `„${v.q}“`, query: { search: v.q, sort: 'artist', limit: 1000 } },
    album: { kind: 'Album', title: v.album, query: { album: v.album, artist: v.artist || null, sort: 'album' }, numbering: 'track' },
    artist: { kind: 'Interpret', title: v.artist, query: { artist: v.artist, sort: 'artist' } },
    genre: { kind: 'Genre', title: v.genre, query: { genre: v.genre, sort: 'artist' } },
  }[v.name] || { kind: 'Bibliothek', title: 'Titel', query: { sort: state.sort }, sortable: true };

  const tracks = await invoke('tracks', { query: cfg.query });
  if (!fresh()) return;

  if (!tracks.length) {
    if (v.name === 'tracks') {
      const stats = await invoke('library_stats');
      if (!fresh()) return;
      if (!stats.folders.length) {
        el.innerHTML = emptyState({
          ico: 'folder',
          title: 'Willkommen bei Amber Player',
          text: 'Wähle den Ordner mit deiner Musik. Amber Player liest ihn ein und hält die Bibliothek automatisch aktuell.',
          actions: `<button class="btn primary" data-act="add-folder">${icon('folder')} Musikordner wählen</button>
                    <button class="btn" data-act="default-folder">Standard-Musikordner verwenden</button>`,
        });
        bindActs(el, {});
        return;
      }
      el.innerHTML = emptyState({ ico: 'refresh', title: stats.scanning ? 'Bibliothek wird eingelesen …' : 'Keine Titel gefunden', text: stats.scanning ? 'Gleich geht es los.' : 'In den Musikordnern wurden keine Audiodateien gefunden.' });
      return;
    }
    const texts = {
      favorites: ['heart', 'Noch keine Favoriten', 'Klicke bei einem Titel auf das Herz, um ihn hier zu sammeln.'],
      recent: ['clock', 'Noch nichts gespielt', 'Hier erscheinen die zuletzt gehörten Titel.'],
      most: ['flame', 'Noch nichts gespielt', 'Hier erscheinen deine meistgehörten Titel.'],
      search: ['search', 'Nichts gefunden', 'Versuche einen anderen Suchbegriff.'],
    }[v.name] || ['music', 'Keine Titel', ''];
    el.innerHTML = head({ kind: cfg.kind, title: cfg.title, actions: cfg.history ? historySeg(v.name) : '' }) + emptyState({ ico: texts[0], title: texts[1], text: texts[2] });
    bindHistorySeg(el);
    if (v.name === 'search') addJumps(el, v.q);
    return;
  }

  const total = tracks.reduce((s, t) => s + t.duration_ms, 0);
  let sub = `${plural(tracks.length, 'Titel', 'Titel')} · ${fmtTotal(total)}`;
  let art = '';
  let round = false;
  if (v.name === 'album') {
    const withCover = tracks.find((t) => t.has_cover);
    art = withCover ? `<img src="${coverUrl(withCover.id)}" alt="">` : `<div class="initials" style="background:${hueGradient(v.album)}">${esc(initials(v.album))}</div>`;
    const artist = v.artist || tracks[0].album_artist || tracks[0].artist;
    const year = tracks.find((t) => t.year)?.year;
    sub = `${artist ? `<a data-artist="${esc(artist)}">${esc(artist)}</a> · ` : ''}${year ? `${year} · ` : ''}${sub}`;
  } else if (v.name === 'artist') {
    round = true;
    art = `<div class="initials" style="background:${hueGradient(v.artist)}">${esc(initials(v.artist))}</div>`;
  }

  let actions = playButtons + (cfg.history ? historySeg(v.name) : '');
  if (cfg.sortable) {
    actions += `<select class="select" id="sortSel" title="Sortierung">
      ${[['artist', 'Interpret'], ['title', 'Titel'], ['album', 'Album'], ['added', 'Zuletzt hinzugefügt']]
        .map(([k, l]) => `<option value="${k}"${state.sort === k ? ' selected' : ''}>Sortiert nach ${l}</option>`).join('')}
    </select>`;
  }
  el.innerHTML = head({ kind: cfg.kind, title: cfg.title, sub, art, round, actions });
  if (v.name === 'search') addJumps(el, v.q);
  const list = new TrackList(el, tracks, { showAlbum: v.name !== 'album', numbering: cfg.numbering || 'index' });
  state.list = list;
  bindActs(el, { tracks });
  bindHistorySeg(el);
  $('#sortSel', el)?.addEventListener('change', (e) => {
    state.sort = e.target.value;
    localStorage.setItem('sort', state.sort);
    render();
  });
}

const historySeg = (cur) => `<span class="seg" data-hist>
  <button data-hv="recent" class="${cur === 'recent' ? 'on' : ''}">Zuletzt gespielt</button>
  <button data-hv="most" class="${cur === 'most' ? 'on' : ''}">Meistgespielt</button></span>`;

function bindHistorySeg(el) {
  $('[data-hist]', el)?.addEventListener('click', (e) => {
    const hv = e.target.closest('[data-hv]')?.dataset.hv;
    if (hv && hv !== state.view.name) navigate({ name: hv }, { push: false });
  });
}

function bindActs(el, { tracks, playlist }) {
  el.addEventListener('click', async (e) => {
    const a = e.target.closest('[data-act]')?.dataset.act;
    const artist = e.target.closest('[data-artist]')?.dataset.artist;
    if (artist) navigate({ name: 'artist', artist });
    if (!a) return;
    switch (a) {
      case 'play': playList(tracks, 0, false); break;
      case 'shuffle': shufflePlay(tracks); break;
      case 'add-folder': addFolder(); break;
      case 'default-folder': addDefaultFolder(); break;
      case 'rename': renamePlaylist(playlist); break;
      case 'phone': phoneStream({ playlistId: playlist.id }); break;
      case 'export': exportPlaylist(playlist); break;
      case 'delete': deletePlaylist(playlist); break;
    }
  });
}

async function renderAlbums(el, v, fresh) {
  const albums = await invoke('albums');
  if (!fresh()) return;
  if (!albums.length) {
    el.innerHTML = head({ kind: 'Bibliothek', title: 'Alben' }) + emptyState({ ico: 'disc', title: 'Keine Alben', text: 'Titel mit Album-Tag erscheinen hier.' });
    return;
  }
  el.innerHTML = `${head({ kind: 'Bibliothek', title: 'Alben', sub: plural(albums.length, 'Album', 'Alben') })}
    <div class="scroll pad"><div class="grid">
      ${albums.map((a, i) => `
        <div class="card" data-i="${i}">
          <div class="cover">${a.cover_track_id ? `<img src="${coverUrl(a.cover_track_id)}" loading="lazy" alt="">` : `<div class="initials" style="background:${hueGradient(a.name)}">${esc(initials(a.name))}</div>`}</div>
          <div class="name">${esc(a.name)}</div>
          <div class="info">${esc(a.artist || 'Unbekannt')}${a.year ? ` · ${a.year}` : ''}</div>
          <button class="play-fab" data-play title="Abspielen">${icon('play', true)}</button>
        </div>`).join('')}
    </div></div>`;
  $('.grid', el).addEventListener('click', async (e) => {
    const card = e.target.closest('.card');
    if (!card) return;
    const a = albums[Number(card.dataset.i)];
    if (e.target.closest('[data-play]')) {
      const tracks = await invoke('tracks', { query: { album: a.name, artist: a.artist || null, sort: 'album' } });
      playList(tracks, 0, false);
    } else {
      navigate({ name: 'album', album: a.name, artist: a.artist });
    }
  });
}

async function renderArtists(el, v, fresh) {
  const artists = await invoke('artists');
  if (!fresh()) return;
  if (!artists.length) {
    el.innerHTML = head({ kind: 'Bibliothek', title: 'Interpreten' }) + emptyState({ ico: 'user', title: 'Keine Interpreten', text: '' });
    return;
  }
  el.innerHTML = `${head({ kind: 'Bibliothek', title: 'Interpreten', sub: plural(artists.length, 'Interpret', 'Interpreten') })}
    <div class="scroll pad"><div class="grid">
      ${artists.map((a, i) => `
        <div class="card round" data-i="${i}">
          <div class="cover"><div class="initials" style="background:${hueGradient(a.name)}">${esc(initials(a.name))}</div></div>
          <div class="name">${esc(a.name)}</div>
          <div class="info">${plural(a.track_count, 'Titel', 'Titel')}</div>
          <button class="play-fab" data-play title="Abspielen">${icon('play', true)}</button>
        </div>`).join('')}
    </div></div>`;
  $('.grid', el).addEventListener('click', async (e) => {
    const card = e.target.closest('.card');
    if (!card) return;
    const a = artists[Number(card.dataset.i)];
    if (e.target.closest('[data-play]')) {
      const tracks = await invoke('tracks', { query: { artist: a.name, sort: 'artist' } });
      playList(tracks, 0, false);
    } else {
      navigate({ name: 'artist', artist: a.name });
    }
  });
}

async function renderGenres(el, v, fresh) {
  const genres = await invoke('genres');
  if (!fresh()) return;
  el.innerHTML = `${head({ kind: 'Bibliothek', title: 'Genres', sub: plural(genres.length, 'Genre', 'Genres') })}
    <div class="scroll pad">${genres.length ? `<div class="chips">
      ${genres.map((g, i) => `<button class="chip" data-i="${i}"><span class="dot" style="background:${hueGradient(g.name)}"></span>${esc(g.name)}<small>${plural(g.track_count, 'Titel', 'Titel')}</small></button>`).join('')}
    </div>` : emptyState({ ico: 'tag', title: 'Keine Genres', text: 'Titel mit Genre-Tag erscheinen hier.' })}</div>`;
  $('.chips', el)?.addEventListener('click', (e) => {
    const chip = e.target.closest('.chip');
    if (chip) navigate({ name: 'genre', genre: genres[Number(chip.dataset.i)].name });
  });
}

async function renderPlaylist(el, v, fresh) {
  const [tracks, playlists] = await Promise.all([invoke('playlist_tracks', { id: v.id }), invoke('playlists')]);
  if (!fresh()) return;
  const pl = playlists.find((p) => p.id === v.id);
  if (!pl) {
    navigate({ name: 'tracks' }, { push: false });
    return;
  }
  const cover = tracks.find((t) => t.has_cover);
  const art = cover ? `<img src="${coverUrl(cover.id)}" alt="">` : `<div class="initials" style="background:${hueGradient(pl.name)}">${icon('playlist')}</div>`;
  const actions = `${tracks.length ? playButtons : ''}
    ${IS_ANDROID ? '' : `<button class="btn" data-act="phone" title="Als Stream fürs Handy (Poweramp)">${icon('phone')} Aufs Handy</button>`}
    <button class="btn icon-only" data-act="rename" title="Umbenennen">${icon('edit')}</button>
    <button class="btn icon-only" data-act="export" title="Als M3U exportieren">${icon('export')}</button>
    <button class="btn icon-only danger" data-act="delete" title="Löschen">${icon('trash')}</button>`;
  el.innerHTML = head({
    kind: 'Playlist',
    title: pl.name,
    sub: `${plural(pl.track_count, 'Titel', 'Titel')} · ${fmtTotal(pl.duration_ms)}`,
    art,
    actions,
  });
  const initialsSvg = $('.art .initials svg', el);
  if (initialsSvg) initialsSvg.style.cssText = 'width:64px;height:64px;color:#fff';
  if (tracks.length) {
    state.list = new TrackList(el, tracks, { playlistId: pl.id });
  } else {
    el.insertAdjacentHTML('beforeend', emptyState({ ico: 'playlist', title: 'Noch leer', text: isTouch ? 'Einen Titel lange drücken und „Zu Playlist hinzufügen“ wählen.' : 'Ziehe Titel auf die Playlist in der Seitenleiste oder nutze „Zu Playlist hinzufügen“ im Rechtsklick-Menü.' }));
  }
  bindActs(el, { tracks, playlist: pl });
}

// ---------- Playlisten ----------

async function loadPlaylists() {
  state.playlists = await invoke('playlists');
  if (state.view.name === 'playlists') render();
}

async function renderPlaylists(el, v, fresh) {
  const lists = await invoke('playlists');
  if (!fresh()) return;
  state.playlists = lists;
  const actions = `<button class="btn primary" data-act="new-pl">${icon('plus')} Neue Playlist</button>
    <button class="btn" data-act="import-pl">${icon('import')} M3U importieren</button>`;
  el.innerHTML = head({ kind: 'Sammlung', title: 'Playlisten', sub: plural(lists.length, 'Playlist', 'Playlisten'), actions }) +
    `<div class="scroll pad">${lists.length ? `<div class="pl-list">${lists.map((p) => `
      <div class="pl-item" data-id="${p.id}">
        <div class="tile" style="background:${hueGradient(p.name)}">${icon('playlist')}</div>
        <div class="txt"><div class="name">${esc(p.name)}</div><div class="info">${plural(p.track_count, 'Titel', 'Titel')} · ${fmtTotal(p.duration_ms)}</div></div>
      </div>`).join('')}</div>`
      : emptyState({ ico: 'playlist', title: 'Noch keine Playlisten', text: 'Lege eine an oder ziehe Titel aus der Bibliothek hierher – beim Ziehen erscheint unten eine Ablage.' })}</div>`;
  el.addEventListener('click', (e) => {
    const act = e.target.closest('[data-act]')?.dataset.act;
    if (act === 'new-pl') return newPlaylist();
    if (act === 'import-pl') return importPlaylist();
    const item = e.target.closest('.pl-item');
    if (item) navigate({ name: 'playlist', id: Number(item.dataset.id) });
  });
  el.addEventListener('contextmenu', (e) => {
    const item = e.target.closest('.pl-item');
    if (item) playlistMenu(e.clientX, e.clientY, lists.find((p) => p.id === Number(item.dataset.id)));
  });
}

function playlistMenu(x, y, pl) {
  showMenu(x, y, [
    { label: 'Abspielen', icon: 'play', action: async () => playList(await invoke('playlist_tracks', { id: pl.id }), 0, false) },
    { label: 'Zufällig abspielen', icon: 'shuffle', action: async () => shufflePlay(await invoke('playlist_tracks', { id: pl.id })) },
    'sep',
    { label: 'Umbenennen', icon: 'edit', action: () => renamePlaylist(pl) },
    { label: 'Als M3U exportieren', icon: 'export', action: () => exportPlaylist(pl) },
    'sep',
    { label: 'Löschen', icon: 'trash', danger: true, action: () => deletePlaylist(pl) },
  ]);
}

/** Ablage, die beim Ziehen von Titeln unten erscheint: Ziel ist eine Playlist oder „Neue Playlist“. */
function showDropTray(open) {
  const tray = $('#dropTray');
  if (!open) {
    tray.classList.remove('open');
    return;
  }
  tray.innerHTML = `<span class="lbl">Ablegen in:</span>
    ${state.playlists.map((p) => `<span class="tray-pl" data-id="${p.id}">${icon('playlist')}${esc(p.name)}</span>`).join('')}
    <span class="tray-pl" data-id="new">${icon('plus')}Neue Playlist</span>`;
  tray.classList.add('open');
}

function bindDropTray() {
  const tray = $('#dropTray');
  tray.addEventListener('dragover', (e) => {
    const b = e.target.closest('.tray-pl');
    if (!b || !dragData) return;
    e.preventDefault();
    e.dataTransfer.dropEffect = 'copy';
    $$('.tray-pl.drop', tray).forEach((x) => x !== b && x.classList.remove('drop'));
    b.classList.add('drop');
  });
  tray.addEventListener('dragleave', (e) => e.target.closest('.tray-pl')?.classList.remove('drop'));
  tray.addEventListener('drop', async (e) => {
    const b = e.target.closest('.tray-pl');
    if (!b || !dragData) return;
    e.preventDefault();
    const ids = dragData.ids;
    showDropTray(false);
    if (b.dataset.id === 'new') return newPlaylist(ids);
    const pl = state.playlists.find((p) => p.id === Number(b.dataset.id));
    await call('add_to_playlist', { id: pl.id, trackIds: ids });
    toast(tx('{n} zu „{name}“ hinzugefügt', { n: plural(ids.length, 'Titel', 'Titel'), name: pl.name }), 'ok');
    await loadPlaylists();
    if (state.view.name === 'playlist' && state.view.id === pl.id) render();
  });
}

async function newPlaylist(trackIds = null) {
  const name = await dialog({ title: 'Neue Playlist', input: '', ok: 'Anlegen' });
  if (!name) return;
  const id = await call('create_playlist', { name, trackIds });
  await loadPlaylists();
  if (trackIds) toast(tx('Playlist „{name}“ angelegt', { name }), 'ok');
  else navigate({ name: 'playlist', id });
}

async function renamePlaylist(pl) {
  const name = await dialog({ title: 'Playlist umbenennen', input: pl.name, ok: 'Speichern' });
  if (!name || name === pl.name) return;
  await call('rename_playlist', { id: pl.id, name });
  await loadPlaylists();
  if (state.view.name === 'playlist') render();
}

async function deletePlaylist(pl) {
  const ok = await dialog({ title: tx('„{name}“ löschen?', { name: pl.name }), text: 'Die Playlist wird entfernt. Die Musikdateien bleiben erhalten.', ok: 'Löschen', danger: true });
  if (!ok) return;
  await call('delete_playlist', { id: pl.id });
  await loadPlaylists();
  if (state.view.name === 'playlist' && state.view.id === pl.id) navigate({ name: 'playlists' }, { push: false });
  toast('Playlist gelöscht', 'ok');
}

async function exportPlaylist(pl) {
  if (await call('export_playlist', { id: pl.id })) toast('Playlist exportiert', 'ok');
}

async function importPlaylist() {
  const id = await call('import_playlist');
  if (id == null) return;
  await loadPlaylists();
  navigate({ name: 'playlist', id });
  toast('Playlist importiert', 'ok');
}

// ---------- Effekte ----------

function fmtVal(v, p) {
  if (p.id === 'balance') {
    const n = Math.round(Math.abs(v));
    return n === 0 ? tx('Mitte') : `${v < 0 ? 'L' : 'R'} ${n} %`;
  }
  const n = Number.isInteger(p.step) ? Math.round(v) : v.toFixed(1);
  const sign = p.unit === 'dB' && v > 0 ? '+' : '';
  return `${sign}${Number(n).toLocaleString(LOCALE, { maximumFractionDigits: 2 })} ${p.unit}`;
}

/**
 * Herkunft eines Plugins aus seiner Quelle: `paket-id/datei` (Plugin-Paket) oder nur `datei`
 * (einzelne Datei im Plugin-Ordner). Liefert Anzeigetext, Tooltip und Gruppenschlüssel.
 */
function pluginOrigin(source, packages = []) {
  if (!source) return null;
  const slash = source.indexOf('/');
  const pkg = slash > 0 ? packages.find((p) => p.id === source.slice(0, slash)) : null;
  if (pkg) {
    const label = pkg.author ? `${tx(pkg.name)} · ${pkg.author}` : tx(pkg.name);
    const title = [
      tx('Plugin-Paket „{name}“', { name: tx(pkg.name) }),
      pkg.version && tx('Version {v}', { v: pkg.version }),
      pkg.author && tx('von {name}', { name: pkg.author }),
      pkg.license && tx('Lizenz: {name}', { name: pkg.license }),
    ].filter(Boolean).join(' · ');
    return { key: `p:${pkg.id}`, label, title };
  }
  return { key: 'file', label: tx('Eigene Datei'), title: source };
}

/** Etiketten „PC“ und „Handy“: wo ein Plugin läuft. */
function platformTags(platforms = ['pc', 'phone']) {
  const names = { pc: ['PC', 'monitor'], phone: ['Handy', 'phone'] };
  return `<span class="plat">${platforms
    .filter((p) => names[p])
    .map((p) => `<span title="${esc(tx('Läuft auf: {name}', { name: tx(names[p][0]) }))}">${icon(names[p][1])}${esc(tx(names[p][0]))}</span>`)
    .join('')}</span>`;
}

// ---------- Plugins: am PC Ordner öffnen, am Handy Datei importieren ----------

/** Knopf für einen Plugin-Ordner: am Handy „Importieren“, am PC „Ordner öffnen“. */
function plugBtn(kind, label = 'Ordner öffnen', cls = 'btn small') {
  // Handy: nur Installieren (kein Zugriff auf den App-Ordner). PC: Installieren und Ordner öffnen.
  return IS_ANDROID
    ? `<button class="${cls}" data-plugdir="${kind}">${icon('plus')} Installieren</button>`
    : `<button class="${cls}" data-plugimport="${kind}">${icon('plus')} Installieren</button>
       <button class="${cls}" data-plugdir="${kind}">${icon('folder')} ${label}</button>`;
}

/**
 * ZIP-Datei im Browser entpacken (nur Textdateien, wie sie Plugin-Pakete enthalten).
 * Nutzt den eingebauten Entpacker (DecompressionStream) – keine zusätzliche Bibliothek.
 * Liegt alles in einem gemeinsamen Ordner („telefon/plugin.json“), wird er weggelassen.
 */
async function unzipText(buffer) {
  const dv = new DataView(buffer);
  let eocd = -1;
  for (let i = buffer.byteLength - 22; i >= Math.max(0, buffer.byteLength - 65557); i--) {
    if (dv.getUint32(i, true) === 0x06054b50) {
      eocd = i;
      break;
    }
  }
  if (eocd < 0) throw new Error(tx('Das ist keine ZIP-Datei'));
  const count = dv.getUint16(eocd + 10, true);
  let p = dv.getUint32(eocd + 16, true);
  const dec = new TextDecoder();
  const files = {};
  for (let n = 0; n < count; n++) {
    if (dv.getUint32(p, true) !== 0x02014b50) throw new Error(tx('Die ZIP-Datei ist beschädigt'));
    const method = dv.getUint16(p + 10, true);
    const size = dv.getUint32(p + 20, true);
    const nameLen = dv.getUint16(p + 28, true);
    const extraLen = dv.getUint16(p + 30, true);
    const commentLen = dv.getUint16(p + 32, true);
    const local = dv.getUint32(p + 42, true);
    const name = dec.decode(new Uint8Array(buffer, p + 46, nameLen));
    p += 46 + nameLen + extraLen + commentLen;
    if (name.endsWith('/') || name.startsWith('__MACOSX/') || name.split('/').pop().startsWith('.')) continue;
    const start = local + 30 + dv.getUint16(local + 26, true) + dv.getUint16(local + 28, true);
    const data = new Uint8Array(buffer, start, size);
    let bytes;
    if (method === 0) bytes = data;
    else if (method === 8) bytes = new Uint8Array(await new Response(new Blob([data]).stream().pipeThrough(new DecompressionStream('deflate-raw'))).arrayBuffer());
    else throw new Error(tx('Nicht unterstütztes ZIP-Format'));
    files[name] = dec.decode(bytes);
  }
  const names = Object.keys(files);
  const prefix = names.length && names.every((x) => x.includes('/')) ? names[0].split('/')[0] + '/' : '';
  if (prefix && names.every((x) => x.startsWith(prefix))) {
    return Object.fromEntries(names.map((x) => [x.slice(prefix.length), files[x]]));
  }
  return files;
}

/** Plugin-Paket (ZIP) installieren. */
/**
 * Paket installieren. Mit `update` (installiertes Paket: id, name, version) muss die Datei
 * dasselbe Paket sein – es wird samt allen Teilen ersetzt.
 */
async function installPackage(file, update = null, ask = false) {
  const files = await unzipText(await file.arrayBuffer());
  // Online-Quellen dürfen ins Internet – vorher zeigen, wohin, und bestätigen lassen.
  let manifest = {};
  try {
    manifest = JSON.parse(files['plugin.json'] || '{}');
  } catch {}
  if (!manifest.id) throw new Error(tx('Das ist kein Plugin-Paket für Amber Player'));
  if (ask) {
    const parts = packageParts({
      effects: (manifest.contents || []).filter((c) => c.type === 'effect').length || (manifest.type === 'effect' ? 1 : 0),
      visualizers: (manifest.contents || []).filter((c) => c.type === 'visualizer').length || (manifest.type === 'visualizer' ? 1 : 0),
      sources: (manifest.contents || []).filter((c) => c.type === 'source').length || (manifest.type === 'source' ? 1 : 0),
    });
    addPluginTexts([manifest]);
    const ok = await dialog({
      title: tx('Plugin „{name}“ installieren?', { name: tx(manifest.name || manifest.id) }),
      text: [parts, manifest.version && tx('Version {v}', { v: manifest.version }), manifest.author && tx('von {name}', { name: manifest.author }), manifest.license && tx('Lizenz: {name}', { name: manifest.license })].filter(Boolean).join(' · '),
      ok: tx('Installieren'),
    });
    if (!ok) return;
  }
  if (update && manifest.id !== update.id) {
    toast(tx('Diese Datei ist nicht „{name}“, sondern „{other}“', { name: tx(update.name), other: manifest.name || manifest.id || '?' }), 'err');
    return;
  }
  const sources = (manifest.contents || []).filter((c) => c.type === 'source').length || (manifest.type === 'source' ? 1 : 0);
  if (sources) {
    const hosts = (manifest.permissions?.network || []).join(', ') || '–';
    const ok = await dialog({
      title: tx('„{name}“ möchte ins Internet', { name: manifest.name || manifest.id || '?' }),
      text: tx('Das Paket enthält eine Online-Quelle. Sie darf nur diese Adressen erreichen: {hosts}. Installiere es nur, wenn du der Quelle vertraust. Aktiv wird sie erst, wenn „Erweiterte Plugins erlauben“ eingeschaltet ist.', { hosts }),
      ok: tx('Installieren'),
    });
    if (!ok) return;
  }
  const m = await call('install_plugin_package', { files });
  if (m.visualizers && (await vizEnsure())) await vizLoad();
  if (update) {
    toast(update.version && m.version && update.version !== m.version
      ? tx('„{name}“ aktualisiert: {from} → {to}', { name: tx(m.name), from: update.version, to: m.version })
      : tx('„{name}“ neu installiert', { name: tx(m.name) }), 'ok');
    setTimeout(render, 300);
    return;
  }
  toast(
    m.effects || m.sources
      ? tx('„{name}“ installiert ({parts}) – Klangeffekte und Online-Quellen wirken nach dem nächsten Start der App', { name: m.name, parts: packageParts(m) })
      : tx('„{name}“ installiert ({parts})', { name: m.name, parts: packageParts(m) }),
    'ok',
  );
  setTimeout(render, 300);
}

/** „2 Klangeffekte · 1 Visualisierung“ */
function packageParts(p) {
  return [
    p.effects ? plural(p.effects, 'Klangeffekt', 'Klangeffekte') : '',
    p.visualizers ? plural(p.visualizers, 'Visualisierung', 'Visualisierungen') : '',
    p.sources ? plural(p.sources, 'Online-Quelle', 'Online-Quellen') : '',
  ]
    .filter(Boolean)
    .join(' · ');
}

function plugAction(kind) {
  if (!IS_ANDROID) return call('open_plugin_dir', { kind }).catch(() => {});
  return importPlugin(kind);
}

/** Plugin-Datei auswählen und übernehmen (Android: kein Dateimanager-Zugriff auf den App-Ordner). */
/** Knopf „Plugin installieren“: nimmt nur Plugin-Pakete (.amberplugin/.zip) – egal, was drin ist. */
function pkgInstallBtn(cls = 'btn primary') {
  return `<button class="${cls}" data-pkginstall title="${esc(tx('Plugin-Paket (.amberplugin) mit Klangeffekten, Visualisierungen oder Online-Quellen'))}">${icon('puzzle')} ${esc(tx('Plugin installieren'))}</button>`;
}

function pickPackage() {
  const input = document.createElement('input');
  input.type = 'file';
  input.accept = '.amberplugin,.zip,application/zip,*/*';
  input.addEventListener('change', async () => {
    const file = input.files?.[0];
    if (!file) return;
    if (!/\.(amberplugin|zip)$/i.test(file.name)) {
      toast(tx('Bitte ein Plugin-Paket wählen (Datei endet auf .amberplugin oder .zip)'), 'err');
      return;
    }
    try {
      await installPackage(file);
    } catch (e) {
      if (e instanceof Error) toast(e.message, 'err');
    }
  });
  input.click();
}

/** Paket teilen: am Handy über das Android-Teilen-Menü, am PC als Datei speichern. */
async function sharePackage(pkg) {
  // Teilen ist immer möglich; wer es bekommt, sieht vor dem Installieren Autor und Lizenz.
  try {
    const path = await call('export_plugin_package', { id: pkg.id });
    if (!path) return;
    if (IS_ANDROID) {
      await invoke('plugin:amber-mobile|share_file', { path, title: tx('Plugin „{name}“', { name: tx(pkg.name) }) });
    } else {
      toast(tx('Gespeichert: {path}', { path }), 'ok');
    }
  } catch (e) {
    if (e instanceof Error) toast(e.message, 'err');
  }
}

/** Geteilte Datei (Base64) empfangen: nachfragen und installieren. */
async function receivePackage(file) {
  if (!file?.data) return;
  try {
    const bin = atob(file.data);
    const bytes = new Uint8Array(bin.length);
    for (let i = 0; i < bin.length; i++) bytes[i] = bin.charCodeAt(i);
    await installPackage(new File([bytes], file.name || 'plugin.amberplugin'), null, true);
  } catch (e) {
    toast(tx('Das ist kein Plugin-Paket für Amber Player'), 'err');
  }
}

/** Neue Version eines installierten Pakets auswählen und einspielen. */
function updatePackage(pkg) {
  const input = document.createElement('input');
  input.type = 'file';
  input.accept = '.amberplugin,.zip,application/zip,*/*';
  input.addEventListener('change', async () => {
    const file = input.files?.[0];
    if (!file) return;
    try {
      await installPackage(file, pkg);
    } catch (e) {
      if (e instanceof Error) toast(e.message, 'err');
    }
  });
  input.click();
}

function importPlugin(kind) {
  const input = document.createElement('input');
  input.type = 'file';
  // Android kennt für .json/.js oft keinen passenden Dateityp – darum alles anbieten und selbst prüfen.
  input.accept = kind === 'effects' ? '.amberplugin,.zip,application/zip,.json,application/json,*/*' : '.amberplugin,.zip,application/zip,.js,text/javascript,*/*';
  input.addEventListener('change', async () => {
    const file = input.files?.[0];
    if (!file) return;
    try {
      if (/\.(amberplugin|zip)$/i.test(file.name)) {
        await installPackage(file);
        return;
      }
      const content = await file.text();
      const name = await call('import_plugin', { kind, fileName: file.name, content });
      if (kind === 'visualizers') {
        if (await vizEnsure()) await vizLoad();
        toast(tx('„{name}“ importiert', { name }), 'ok');
        setTimeout(render, 300);
      } else {
        toast(tx('„{name}“ importiert – wirkt nach dem nächsten Start der App', { name }), 'ok');
      }
    } catch (e) {
      // Fehler von call() sind schon angezeigt; Entpackfehler hier melden.
      if (e instanceof Error) toast(e.message, 'err');
    }
  });
  input.click();
}

async function renderEffects(el, v, fresh) {
  const [effects, info, pk] = await Promise.all([
    invoke('effects_list'),
    invoke('effects_info'),
    invoke('plugin_packages').catch(() => ({ packages: [] })),
  ]);
  if (!fresh()) return;
  fxPackages = pk.packages || [];
  addPluginTexts(fxPackages);
  const actions = `${pkgInstallBtn()}${plugBtn('effects', 'Plugin-Ordner öffnen', 'btn')}
    <span class="hint" style="margin:0">${esc(IS_ANDROID ? tx('Eigenes Klang-Plugin (.json) auswählen; es wirkt nach dem nächsten Start der App.') : tx('Eigene Klang-Plugins als .json hineinlegen, dann Amber Player neu starten.'))} ${wikiLink('effects', tx('Anleitung im Wiki'))}</span>`;
  const errors = info.errors.length
    ? `<div class="plugin-errors"><b>Nicht geladen:</b>${info.errors.map((e) => `<div>${esc(e)}</div>`).join('')}</div>`
    : '';
  el.innerHTML = `${head({ kind: 'Klang', title: 'Effekte', sub: 'Plugins wirken sofort auf die laufende Wiedergabe und werden gespeichert.', actions })}
    <div class="scroll pad">${errors}<div class="fx-list">${effects.map(fxCard).join('')}</div></div>`;
  for (const b of $$('[data-plugdir]', el)) b.addEventListener('click', () => plugAction(b.dataset.plugdir));
  for (const b of $$('[data-plugimport]', el)) b.addEventListener('click', () => importPlugin(b.dataset.plugimport));

  const byId = Object.fromEntries(effects.map((f) => [f.id, f]));
  const pushValues = throttle((id) => invoke('effect_set', { id, enabled: null, values: byId[id].values }).catch((e) => toast(String(e), 'err')), 60);

  el.addEventListener('input', (e) => {
    const inp = e.target.closest('input[type=range][data-p]');
    if (!inp) return;
    const card = inp.closest('.fx');
    const fx = byId[card.dataset.id];
    const p = Number(inp.dataset.p);
    fx.values[p] = Number(inp.value);
    setRange(inp, inp.value);
    $(`[data-val="${p}"]`, card).textContent = fmtVal(fx.values[p], fx.params[p]);
    const preset = $('select', card);
    if (preset) preset.value = '';
    $('[data-fx="delete"]', card)?.setAttribute('hidden', '');
    pushValues(fx.id);
  });
  el.addEventListener('dblclick', (e) => {
    // Doppelklick auf einen Regler setzt ihn auf den Standardwert.
    const inp = e.target.closest('input[type=range][data-p]');
    if (!inp) return;
    const fx = byId[inp.closest('.fx').dataset.id];
    inp.value = fx.params[Number(inp.dataset.p)].default;
    inp.dispatchEvent(new Event('input', { bubbles: true }));
  });
  el.addEventListener('change', async (e) => {
    const card = e.target.closest('.fx');
    if (!card) return;
    const id = card.dataset.id;
    if (e.target.matches('[data-enable]')) {
      const info = await call('effect_set', { id, enabled: e.target.checked, values: null });
      byId[id].enabled = info.enabled;
      card.classList.toggle('off', !info.enabled);
    } else if (e.target.matches('select') && e.target.value) {
      const info = await call('effect_preset', { id, preset: e.target.value.slice(2) });
      redrawFx(id, info, e.target.value);
    }
  });
  // Neu zeichnen und die gewählte Voreinstellung stehen lassen
  const redrawFx = (id, info, selected = '') => {
    Object.assign(byId[id], info);
    $(`.fx[data-id="${id}"]`, el).outerHTML = fxCard(byId[id]);
    const card = $(`.fx[data-id="${id}"]`, el);
    const sel = $('select', card);
    if (sel && [...sel.options].some((o) => o.value === selected)) sel.value = selected;
    $('[data-fx="delete"]', card)?.toggleAttribute('hidden', !selected.startsWith('u:'));
    for (const inp of $$('input[type=range]', card)) setRange(inp, inp.value);
  };
  el.addEventListener('click', async (e) => {
    const b = e.target.closest('[data-fx]');
    if (!b) return;
    const card = b.closest('.fx');
    const id = card.dataset.id;
    const selected = $('select', card)?.value || '';
    if (b.dataset.fx === 'save') {
      const name = await dialog({
        title: tx('Voreinstellung speichern'),
        text: tx('Die aktuellen Werte von „{name}“ werden unter diesem Namen gespeichert.', { name: tx(byId[id].name) }),
        input: selected.startsWith('u:') ? selected.slice(2) : '',
        ok: tx('Speichern'),
      });
      if (!name) return;
      const info = await call('effect_save_preset', { id, name });
      redrawFx(id, info, `u:${name.trim()}`);
      toast(tx('Voreinstellung „{name}“ gespeichert', { name: name.trim() }), 'ok');
    } else if (b.dataset.fx === 'reset') {
      redrawFx(id, await call('effect_reset', { id }));
    } else if (b.dataset.fx === 'delete' && selected.startsWith('u:')) {
      const name = selected.slice(2);
      if (!(await dialog({ title: tx('Voreinstellung löschen?'), text: `„${name}“`, ok: tx('Löschen'), danger: true }))) return;
      redrawFx(id, await call('effect_delete_preset', { id, name }));
    }
  });
  for (const inp of $$('input[type=range]', el)) setRange(inp, inp.value);
}

let fxPackages = [];

function fxCard(fx) {
  const origin = pluginOrigin(fx.source, fxPackages);
  const isEq = fx.id === 'equalizer';
  const own = fx.user_presets || [];
  const opts = (list, prefix) => list.map((p) => `<option value="${prefix}${esc(p.name)}">${esc(p.name)}</option>`).join('');
  const presets = fx.presets.length || own.length
    ? `<select class="select"><option value="">${esc(tx('Preset wählen …'))}</option>${
        own.length ? `<optgroup label="${esc(tx('Eigene'))}">${opts(own, 'u:')}</optgroup>` : ''}${
        fx.presets.length ? `<optgroup label="${esc(tx('Mitgeliefert'))}">${opts(fx.presets, 'b:')}</optgroup>` : ''}</select>`
    : '';
  const body = isEq
    ? `<div class="eq">${fx.params.map((p, i) => `
        <div class="eq-band">
          <span class="val" data-val="${i}">${fmtVal(fx.values[i], p)}</span>
          <div class="slot"><input type="range" data-p="${i}" min="${p.min}" max="${p.max}" step="${p.step}" value="${fx.values[i]}" title="Doppelklick: 0 dB"></div>
          <span class="f">${esc(p.name.replace(' Hz', '').replace(' kHz', 'k'))}</span>
        </div>`).join('')}</div>`
    : fx.params.map((p, i) => `
        <div class="param">
          <label>${esc(p.name)}</label>
          <input type="range" data-p="${i}" min="${p.min}" max="${p.max}" step="${p.step}" value="${fx.values[i]}" title="Doppelklick: Standardwert">
          <span class="val" data-val="${i}">${fmtVal(fx.values[i], p)}</span>
        </div>`).join('');
  return `<div class="fx${fx.enabled ? '' : ' off'}" data-id="${fx.id}">
    <div class="fx-head">
      <label class="switch" title="An/Aus"><input type="checkbox" data-enable ${fx.enabled ? 'checked' : ''}><span></span></label>
      <div class="grow"><h3>${esc(fx.name)}${origin ? `<span class="src-badge" data-raw title="${esc(origin.title)}">${esc(origin.label)}</span>` : ''}</h3><p>${esc(fx.description)}</p></div>
      ${presets}
    </div>
    <div class="fx-body">${body}</div>
    ${fx.params.length ? `<div class="fx-foot">
      <button class="btn small" data-fx="save">${icon('save')} ${esc(tx('Speichern'))}</button>
      <button class="btn small" data-fx="reset">${icon('refresh')} ${esc(tx('Zurücksetzen'))}</button>
      <button class="btn small" data-fx="delete" hidden>${icon('trash')} ${esc(tx('Preset löschen'))}</button>
    </div>` : ''}
  </div>`;
}

// ---------- Einstellungen ----------

function copyField(text) {
  return `<div class="copy"><code title="${esc(text)}">${esc(text)}</code><button data-copy="${esc(text)}" title="Kopieren">${icon('copy')}</button></div>`;
}

async function renderSettings(el, v, fresh) {
  const [folders, api, stats, covers, packages, streams, firewall] = await Promise.all([
    invoke('folders'),
    invoke('api_info'),
    invoke('library_stats'),
    invoke('cover_settings'),
    invoke('plugin_packages'),
    IS_ANDROID ? { lan: false, links: [] } : invoke('stream_links'),
    IS_ANDROID ? { active: false } : invoke('firewall_status'),
  ]);
  if (!fresh()) return;
  addPluginTexts(packages.packages);
  const theme = localStorage.getItem('theme') || 'dark';
  const accent = localStorage.getItem('accent') || ACCENTS[0][0];
  const ex = (path) => `${api.url}${path}${api.lan ? (path.includes('?') ? '&' : '?') + `token=${api.token}` : ''}`;
  const mcpCmd = api.lan
    ? `claude mcp add --transport http amber-player ${api.mcp_url} --header "Authorization: Bearer ${api.token}"`
    : `claude mcp add --transport http amber-player ${api.mcp_url}`;

  el.innerHTML = `${head({ kind: 'Amber Player', title: 'Einstellungen' })}
  <div class="scroll pad"><div class="settings">
    ${updates.info?.newer ? `<div class="box update-box">
      <h3>${esc(tx('Neue Version {v} verfügbar', { v: updates.info.latest }))}</h3>
      <p>${esc(tx('Du hast Version {v}. Deine Bibliothek, Playlisten und Einstellungen bleiben beim Update erhalten.', { v: updates.info.current }))}</p>
      <div class="actions">
        <button class="btn primary" data-act="update-download">${icon('import')} ${esc(tx('Herunterladen'))}</button>
        ${IS_ANDROID ? '' : `<span class="hint" style="margin:0">${esc(tx('Öffnet die Release-Seite mit .deb, .rpm und AppImage'))}</span>`}
      </div>
    </div>` : ''}

    <div class="box">
      <h3>Musikordner</h3>
      <p>${esc(tx('{n} in der Bibliothek. Neue und geänderte Dateien werden beim Start automatisch erkannt.', { n: plural(stats.tracks, 'Titel', 'Titel') }))}</p>
      ${folders.map((f) => `<div class="folder">${icon('folder')}<span class="path" title="${esc(f)}">${esc(f)}</span><button class="btn small danger" data-remove="${esc(f)}">Entfernen</button></div>`).join('') || '<p class="hint">Noch kein Ordner gewählt.</p>'}
      <div class="actions">
        <button class="btn primary" data-act="add-folder">${icon('plus')} Ordner hinzufügen</button>
        <button class="btn" data-act="rescan">${icon('refresh')} Neu einlesen</button>
      </div>
    </div>

    ${IS_ANDROID ? '' : `<div class="box">
      <h3>Fernsteuerung &amp; MCP</h3>
      <p>${esc(tx('Steuere Amber Player über Stream Deck, TouchDeck, Home Assistant, KI-Assistenten oder die Handy-App.'))} ${wikiLink('api', tx('Anleitung im Wiki'))}</p>
      <div class="actions" style="margin:4px 0 14px"><button class="btn primary" data-act="pair">${icon('phone')} Handy koppeln</button></div>
      <div class="kv">
        <span class="k">Status</span>
        <span>${api.running ? '<span class="status-dot"></span>Läuft' : `<span class="status-dot bad"></span>${esc(api.error || 'Gestoppt')}`}</span>
        <span class="k">API-Adresse</span>${copyField(api.url + '/api')}
        <span class="k">MCP-Server</span>${copyField(api.mcp_url)}
        <span class="k">Claude Code</span>${copyField(mcpCmd)}
      </div>
      <div class="kv">
        <span class="k">Im Heimnetz</span>
        <span style="display:flex;align-items:center;gap:12px">
          <label class="switch"><input type="checkbox" id="apiLan" ${api.lan ? 'checked' : ''}><span></span></label>
          <span class="hint" style="margin:0">Andere Geräte (Handy, Tablet, Stream Deck an anderem PC) dürfen zugreifen – nur mit Token.</span>
        </span>
        <span class="k">Port</span>
        <span style="display:flex;gap:10px;align-items:center"><input class="field" id="apiPort" type="number" min="1024" max="65535" value="${api.port}">
          <button class="btn small" id="apiSave">Speichern</button></span>
        ${api.lan ? `<span class="k">Token</span><span style="display:flex;gap:10px;align-items:center">${copyField(api.token)}<button class="btn small" id="apiToken">Neu erzeugen</button></span>` : ''}
      </div>
      <h3 style="margin-top:18px">Beispiele für Stream-Deck-Tasten</h3>
      <p class="hint" style="margin:4px 0 8px">Jede Adresse funktioniert per GET und POST, z. B. mit „Website öffnen“ im Hintergrund oder einem HTTP-Request-Plugin.</p>
      <table class="urls">
        ${[
          ['Play / Pause', '/api/toggle'],
          ['Nächster Titel', '/api/next'],
          ['Voriger Titel', '/api/prev'],
          ['Lauter / Leiser', '/api/volume/up'],
          ['Lautstärke 30 %', '/api/volume?value=30'],
          ['Stumm', '/api/mute'],
          ['Zufallswiedergabe', '/api/shuffle'],
          ['Favorit (laufender Titel)', '/api/favorite'],
          ['Favoriten zufällig', '/api/play/music?favorites=true&shuffle=true'],
          ['Equalizer an/aus', '/api/effects/equalizer/toggle'],
          ['EQ-Preset „Bass“', '/api/effects/equalizer?preset=Bass'],
          ['Cover als Tastenbild', '/api/cover/current'],
          ['Status (JSON)', '/api/status'],
        ].map(([l, p]) => `<tr><td>${esc(l)}</td><td>${copyField(ex(p))}</td></tr>`).join('')}
      </table>
    </div>

    <div class="box">
      <h3>Aufs Handy streamen</h3>
      <p>Hör deine Musik vom PC auf dem Handy – ohne Dateien zu kopieren: Link aufs Handy (QR-Code), die .m3u-Datei herunterladen und mit Poweramp öffnen – nicht unter „URL hinzufügen“ eintragen, dort liest Poweramp keine Listen. Handy und PC müssen im selben WLAN sein.</p>
      ${streams.lan && firewall.active && !firewall.opened
        ? `<div class="plugin-errors" style="border-color:var(--accent)"><b style="color:var(--accent)">Firewall:</b> ${esc(tx('Auf diesem PC ist die Firewall (ufw) aktiv. Damit das Handy durchkommt, muss Port {port} fürs Heimnetz freigegeben werden.', { port: firewall.port }))}
             <div style="font-family:var(--sans);margin-top:10px"><button class="btn small primary" data-act="firewall">Firewall fürs Heimnetz freigeben</button></div></div>`
        : ''}
      ${streams.lan
        ? streams.links.map((l, i) => `<div class="stream-row"><b title="${esc(l.name)}">${esc(l.name)}</b>${copyField(l.url)}<button class="btn small" data-qr="${i}">QR-Code</button></div>`).join('')
        : `<p class="hint">Dafür muss Amber Player im Heimnetz erreichbar sein (geschützt mit einem Token, Programme auf diesem PC brauchen keins).</p>
           <button class="btn primary" data-act="lan-on">${icon('phone')} Im Heimnetz freigeben und neu starten</button>`}
    </div>`}

    <div class="box">
      <h3>Cover</h3>
      <p>Fehlende Cover sucht Amber Player automatisch im Internet (Deezer, MusicBrainz). Dafür werden nur Interpret, Titel und Album gesendet. Übernommen wird ein Cover nur, wenn Interpret und Titel wirklich passen.</p>
      <div class="kv">
        <span class="k">Automatisch laden</span>
        <span style="display:flex;gap:12px;align-items:center">
          <label class="switch"><input type="checkbox" id="coverOn" ${covers.enabled ? 'checked' : ''}><span></span></label>
          <button class="btn small" id="coverAgain">${icon('refresh')} Fehlende erneut suchen</button>
        </span>
      </div>
    </div>

    <div class="box">
      <h3>Plugins</h3>
      <p>${tx('Klangeffekte und Visualisierungen lassen sich erweitern – eine Anleitung steht in {file}.', { file: wikiLink('plugins', tx('Wiki')) })}</p>
      <div class="pkg-install">${pkgInstallBtn()}<span class="hint">${esc(tx('Für alle Plugin-Pakete (.amberplugin) – Klangeffekte, Visualisierungen und Online-Quellen in einem.'))}</span></div>
      <div class="kv">
        <span class="k">Klangeffekte</span>
        <span style="display:flex;gap:10px;align-items:center">${plugBtn('effects')}<span class="hint" style="margin:0">${esc(tx('.amberplugin-Paket oder .json, gelten nach Neustart'))}</span></span>
        ${IS_ANDROID ? '' : `        <span class="k">Quellen (Online)</span>
        <span style="display:flex;gap:10px;align-items:center"><button class="btn small" data-plugdir="sources">${icon('folder')} Ordner öffnen</button><span class="hint" style="margin:0">${online.sources.length ? esc(online.sources.map((s) => s.name).join(', ')) : esc(tx('keine installiert'))} · ${esc(tx('gelten nach Neustart'))}</span></span>`}
        <span class="k">Visualisierungen</span>
        <span style="display:flex;gap:10px;align-items:center">${plugBtn('visualizers')}<button class="btn small" id="vizReload">${icon('refresh')} Neu laden</button><span class="hint" style="margin:0">${esc(tx('.amberplugin-Paket oder .js'))}</span></span>
      </div>
      <div class="kv">
        <span class="k">${esc(tx('Erweiterte Plugins'))}</span>
        <span style="display:flex;align-items:center;gap:12px">
          <label class="switch"><input type="checkbox" id="extendedOn"><span></span></label>
          <span class="hint" style="margin:0">${esc(tx('Erlaubt Online-Quellen aus Plugin-Paketen, ins Internet zu gehen – nur zu den Adressen, die das Paket angibt. Gilt nach dem nächsten Start der App.'))}</span>
        </span>
      </div>
      <h3 style="margin-top:18px">Installierte Plugins</h3>
      ${packages.packages.length
        ? `<div class="pkg-list">${packages.packages.map((p) => `<div class="pkg">
            <div class="pkg-meta" data-raw><b>${esc(tx(p.name))}</b>${p.version ? ` <small>${esc(p.version)}</small>` : ''}${platformTags(p.platforms)}
              <span>${esc([packageParts(p), p.author, p.license || (p.license_file ? tx('eigene Lizenz') : tx('keine Lizenz angegeben')), p.license_file ? '' : tx('ohne Lizenzdatei')].filter(Boolean).join(' · '))}</span></div>
            <button class="btn small" data-share="${esc(p.id)}" title="${esc(IS_ANDROID ? tx('Mit anderen teilen – Quick Share, Bluetooth, Messenger …') : tx('Als .amberplugin-Datei speichern, z. B. zum Weitergeben'))}">${icon('export')} ${esc(IS_ANDROID ? tx('Teilen') : tx('Exportieren'))}</button>
            <button class="btn small" data-update="${esc(p.id)}" title="${esc(tx('Neue Version dieses Pakets auswählen – ersetzt die alte samt allen Teilen'))}">${icon('refresh')} ${esc(tx('Aktualisieren'))}</button>
            <button class="btn small danger" data-unplug="${esc(`${p.id}|${p.name}|${p.visualizers ? 1 : 0}`)}">${esc(tx('Entfernen'))}</button>
          </div>`).join('')}</div>`
        : `<p class="hint">${esc(tx('Noch keine Plugin-Pakete installiert. Ein Paket (.amberplugin) enthält einen oder mehrere Klangeffekte und Visualisierungen mit eigener Lizenz.'))}</p>`}
      ${packages.errors.length ? `<div class="plugin-errors"><b>${esc(tx('Nicht geladen:'))}</b>${packages.errors.map((e) => `<div>${esc(e)}</div>`).join('')}</div>` : ''}
      ${viz.list.length ? `<div class="plug-list">${viz.list.map((v) => {
        const o = pluginOrigin(v.source, packages.packages);
        return `<div><b data-raw>${esc(tx(v.name))}</b><span data-raw>${esc(o ? o.label : tx('Mitgeliefert'))}${v.author ? ` · ${esc(v.author)}` : ''}</span></div>`;
      }).join('')}</div>` : ''}
    </div>

    <div class="box">
      <h3>Darstellung</h3>
      <div class="kv">
        <span class="k">${esc(tx('Visualisierung'))}</span>
        <span><select class="select" id="vizHideSel">
          ${[3, 6, 10, 20].map((n) => `<option value="${n}">${esc(tx('Bedienelemente nach {n} s ausblenden', { n }))}</option>`).join('')}
          <option value="0">${esc(tx('Bedienelemente nie ausblenden'))}</option>
        </select>
        <span class="hint">${esc(tx('Antippen blendet sie ein, nochmal antippen sofort wieder aus.'))}</span></span>
      </div>
      ${IS_ANDROID || isMobile() ? `<div class="kv" id="navBox">
        <span class="k">${esc(tx('Leiste unten'))}</span>
        <span><span class="nav-pick">${navChoice().map((id, i) => `<select class="select" data-slot="${i}">${Object.keys(NAV_ITEMS)
          .filter(navAvailable)
          .map((k) => `<option value="${k}"${k === id ? ' selected' : ''}>${esc(tx(NAV_ITEMS[k][0]))}</option>`)
          .join('')}</select>`).join('')}</span>
          <span class="hint">${esc(tx('Welche vier Bereiche unten stehen. Alles andere findest du unter „Mehr“.'))}</span></span>
      </div>` : ''}
      <div class="kv">
        <span class="k">Sprache</span>
        <span><select class="select" id="langSel">
          <option value="system">${esc(tx('Systemsprache'))}</option>
          ${LANGUAGES.map(([c, n]) => `<option value="${c}">${esc(n)}</option>`).join('')}
        </select></span>
        <span class="k">Akzentfarbe</span>
        <span class="swatches" id="swatches">
          ${ACCENTS.map(([c, n]) => `<button class="swatch${accent === c ? ' on' : ''}" data-accent="${c}" title="${n}" style="background:${c}"></button>`).join('')}
          <label class="color-pick" title="Eigene Farbe" style="${ACCENTS.some(([c]) => c === accent) ? '' : `background:${accent};outline-style:solid`}">${icon('plus')}<input type="color" id="accentPick" value="${accent}"></label>
        </span>
        <span class="k">Design</span>
        <span class="seg" id="themeSeg">
          <button data-theme="dark" class="${theme === 'dark' ? 'on' : ''}">Dunkel</button>
          <button data-theme="light" class="${theme === 'light' ? 'on' : ''}">Hell</button>
          <button data-theme="system" class="${theme === 'system' ? 'on' : ''}">System</button>
        </span>
        ${IS_ANDROID ? `<span class="k">Vollbild</span>
        <span style="display:flex;align-items:center;gap:12px">
          <label class="switch"><input type="checkbox" id="fullscreenOn"><span></span></label>
          <span class="hint" style="margin:0">Status- und Navigationsleiste ausblenden (Wischen vom Rand zeigt sie kurz)</span>
        </span>` : ''}
      </div>
    </div>

    <div class="box">
      <h3>Tastenkürzel</h3>
      <table class="urls">
        <tr><td>Leertaste</td><td>Abspielen / Pause</td></tr>
        <tr><td>← / →</td><td>10 Sekunden zurück / vor</td></tr>
        <tr><td>Strg + ← / →</td><td>Voriger / nächster Titel</td></tr>
        <tr><td>Strg + ↑ / ↓</td><td>Lauter / leiser</td></tr>
        <tr><td>M</td><td>Stumm</td></tr>
        <tr><td>S / R</td><td>Zufall / Wiederholen</td></tr>
        <tr><td>Strg + F</td><td>Suchen</td></tr>
        <tr><td>V</td><td>Visualisierung an/aus (darin: F = Vollbild, 1–9 = Motiv)</td></tr>
        <tr><td>Strg + A</td><td>Alle Titel markieren</td></tr>
        <tr><td>Entf</td><td>Markierte Titel aus Playlist entfernen</td></tr>
        <tr><td>Medientasten</td><td>werden vom System an Amber Player weitergegeben</td></tr>
      </table>
    </div>
    <div class="update-row">
      <label class="switch" title="${esc(tx('Automatisch nach Updates suchen'))}"><input type="checkbox" id="updateAuto" ${updateCheckEnabled() ? 'checked' : ''}><span></span></label>
      <span class="hint" style="margin:0">${esc(tx('Automatisch nach Updates suchen (einmal am Tag bei GitHub)'))}</span>
      <button class="btn small" data-act="update-check">${icon('refresh')} ${esc(tx('Jetzt prüfen'))}</button>
    </div>
    <p class="app-version">${wikiLink('home', tx('Hilfe und Anleitung im Wiki'))}</p>
    <p class="app-version" id="appVersion" data-raw></p>
  </div></div>`;
  // Versionsnummer ganz unten (kommt aus der App selbst, wie beim Installationspaket)
  window.__TAURI__.app
    ?.getVersion()
    .then((v) => {
      const box = $('#appVersion', el);
      if (box) box.textContent = `Amber Player · ${tx('Version {v}', { v })} · ${IS_ANDROID ? 'Android' : 'Linux'}`;
    })
    .catch(() => {});

  bindActs(el, {});
  el.addEventListener('click', async (e) => {
    const copy = e.target.closest('[data-copy]')?.dataset.copy;
    if (copy) {
      await navigator.clipboard.writeText(copy).then(() => toast('Kopiert', 'ok')).catch(() => toast('Kopieren nicht möglich', 'err'));
      return;
    }
    const remove = e.target.closest('[data-remove]')?.dataset.remove;
    if (remove) {
      const ok = await dialog({ title: 'Ordner entfernen?', text: `${remove}\n\n${tx('Die Titel verschwinden aus der Bibliothek, die Dateien bleiben erhalten.')}`, ok: 'Entfernen', danger: true });
      if (ok) {
        await call('remove_folder', { path: remove });
        render();
      }
      return;
    }
    if (e.target.closest('[data-act="rescan"]')) {
      invoke('rescan');
      return;
    }
    const sw = e.target.closest('[data-accent]')?.dataset.accent;
    if (sw) {
      localStorage.setItem('accent', sw);
      applyAccent(sw);
      $$('.swatch', el).forEach((b) => b.classList.toggle('on', b.dataset.accent === sw));
      $('.color-pick', el).style.cssText = '';
      return;
    }
    const th = e.target.closest('#themeSeg button')?.dataset.theme;
    if (th) {
      localStorage.setItem('theme', th);
      applyTheme();
      $$('#themeSeg button', el).forEach((b) => b.classList.toggle('on', b.dataset.theme === th));
    }
  });
  const saveApi = async (newToken = false) => {
    const info = await call('api_configure', { lan: $('#apiLan', el).checked, port: Number($('#apiPort', el).value), newToken });
    if (info.restart_required) toast('Gespeichert – gilt nach dem nächsten Start von Amber Player', 'ok');
    render();
  };
  $('#accentPick', el).addEventListener('input', (e) => {
    const c = e.target.value;
    localStorage.setItem('accent', c);
    applyAccent(c);
    $$('.swatch', el).forEach((b) => b.classList.remove('on'));
    $('.color-pick', el).style.cssText = `background:${c};outline-style:solid`;
  });
  for (const b of $$('[data-qr]', el)) b.addEventListener('click', () => streamDialog(streams.links[Number(b.dataset.qr)]));
  $('[data-act="lan-on"]', el)?.addEventListener('click', enableLan);
  $('[data-act="pair"]', el)?.addEventListener('click', pairDialog);
  const extBox = $('#extendedOn', el);
  if (extBox) {
    invoke('extended_plugins').then((on) => (extBox.checked = !!on)).catch(() => {});
    extBox.addEventListener('change', async () => {
      if (extBox.checked) {
        const ok = await dialog({
          title: tx('Erweiterte Plugins erlauben?'),
          text: tx('Online-Quellen aus Plugin-Paketen dürfen dann ins Internet, jeweils nur zu den Adressen, die sie bei der Installation angegeben haben. Sie laufen abgeschottet ohne Zugriff auf deine Dateien. Erlaube das nur für Pakete, denen du vertraust.'),
          ok: tx('Erlauben'),
        });
        if (!ok) {
          extBox.checked = false;
          return;
        }
      }
      try {
        await call('set_extended_plugins', { on: extBox.checked });
        toast(tx('Gespeichert – gilt nach dem nächsten Start von Amber Player'), 'ok');
      } catch {}
    });
  }
  $('[data-act="update-download"]', el)?.addEventListener('click', downloadUpdate);
  $('[data-act="update-check"]', el)?.addEventListener('click', () => checkUpdates(true));
  $('#updateAuto', el)?.addEventListener('change', (e) => {
    try {
      localStorage.setItem('updateCheck', e.target.checked ? 'on' : 'off');
    } catch {}
  });
  const langSel = $('#langSel', el);
  try {
    langSel.value = localStorage.getItem('lang') || 'system';
  } catch {
    langSel.value = 'system';
  }
  langSel.addEventListener('change', () => {
    try {
      localStorage.setItem('lang', langSel.value);
    } catch {}
    // Neu laden, damit alle Texte in der neuen Sprache erscheinen.
    location.reload();
  });
  for (const sel of $$('#navBox select', el)) {
    sel.addEventListener('change', () => {
      const list = navChoice();
      const slot = Number(sel.dataset.slot);
      const other = list.indexOf(sel.value);
      // schon in einem anderen Platz? dann tauschen
      if (other >= 0 && other !== slot) list[other] = list[slot];
      list[slot] = sel.value;
      setNavChoice(list);
      $$('#navBox select', el).forEach((x, i) => (x.value = list[i]));
      toast(tx('Leiste gespeichert'), 'ok');
    });
  }
  if (v.focus === 'nav') $('#navBox', el)?.scrollIntoView({ block: 'center' });
  const hideSel = $('#vizHideSel', el);
  if (hideSel) {
    hideSel.value = String(vizHideMs() / 1000);
    hideSel.addEventListener('change', () => {
      try {
        localStorage.setItem('vizHide', hideSel.value);
      } catch {}
    });
  }
  const fsBox = $('#fullscreenOn', el);
  if (fsBox) {
    invoke('plugin:amber-mobile|get_fullscreen').then((r) => (fsBox.checked = r?.on !== false)).catch(() => {});
    fsBox.addEventListener('change', () => invoke('plugin:amber-mobile|set_fullscreen', { on: fsBox.checked }).catch((e) => toast(String(e), 'err')));
  }
  $('[data-act="firewall"]', el)?.addEventListener('click', async () => {
    await call('open_firewall');
    toast('Firewall fürs Heimnetz freigegeben', 'ok');
    render();
  });
  $('#coverOn', el).addEventListener('change', (e) => call('set_cover_download', { enabled: e.target.checked }).catch(() => {}));
  $('#coverAgain', el).addEventListener('click', () => call('refetch_covers').then(() => toast('Cover-Suche gestartet', 'ok')).catch(() => {}));
  for (const b of $$('[data-plugdir]', el)) b.addEventListener('click', () => plugAction(b.dataset.plugdir));
  for (const b of $$('[data-plugimport]', el)) b.addEventListener('click', () => importPlugin(b.dataset.plugimport));
  for (const b of $$('[data-share]', el)) {
    b.addEventListener('click', () => sharePackage(packages.packages.find((p) => p.id === b.dataset.share)));
  }
  for (const b of $$('[data-update]', el)) {
    b.addEventListener('click', () => updatePackage(packages.packages.find((p) => p.id === b.dataset.update)));
  }
  for (const b of $$('[data-unplug]', el)) {
    b.addEventListener('click', async () => {
      const [id, name, hasViz] = b.dataset.unplug.split('|');
      const ok = await dialog({ title: tx('„{name}“ entfernen?', { name }), text: tx('Das Plugin wird gelöscht. Klangeffekte verschwinden nach dem nächsten Start der App.'), ok: tx('Entfernen'), danger: true });
      if (!ok) return;
      try {
        await call('remove_plugin_package', { id });
        if (hasViz === '1' && (await vizEnsure())) await vizLoad();
        toast(tx('„{name}“ entfernt', { name }), 'ok');
        render();
      } catch {}
    });
  }
  $('#vizReload', el).addEventListener('click', async () => {
    if (!(await vizEnsure())) return;
    await vizLoad();
    setTimeout(render, 300);
    toast('Visualisierungen neu geladen', 'ok');
  });
  $('#apiSave', el)?.addEventListener('click', () => saveApi());
  $('#apiLan', el)?.addEventListener('change', () => saveApi());
  $('#apiToken', el)?.addEventListener('click', () => saveApi(true));
}

async function addFolder() {
  const path = await call('add_folder');
  if (path) {
    toast(tx('Ordner hinzugefügt: {path}', { path }), 'ok');
    render();
  }
}

async function addDefaultFolder() {
  const path = await call('add_default_folder');
  if (path) {
    toast(tx('Ordner hinzugefügt: {path}', { path }), 'ok');
    render();
  } else {
    toast('Kein Standard-Musikordner gefunden', 'err');
  }
}

function applyTheme() {
  const t = localStorage.getItem('theme') || 'dark';
  const dark = t === 'system' ? matchMedia('(prefers-color-scheme: dark)').matches : t === 'dark';
  document.documentElement.dataset.theme = dark ? 'dark' : 'light';
}

// ---------- Warteschlange und „Läuft gerade“ ----------

function queueItem(t) {
  return `<div class="q-item">${artHtml(t)}<div class="txt"><div class="t">${esc(t.title)}</div><div class="a">${esc(t.artist)}</div></div><span class="d">${fmtTime(t.duration_ms)}</span></div>`;
}

const renderUpNext = debounce(async () => {
  const r = !state.status?.track ? state.status?.radio : null;
  if (r) {
    const saved = await invoke('stations').catch(() => []);
    $('#queueList').innerHTML = saved.length
      ? saved.map((st) => `<div class="q-item${st.url === r.station.url ? ' current' : ''}" data-station="${st.id}" style="cursor:pointer">
          <div class="thumb" style="overflow:hidden;border-radius:6px">${stationLogo(st)}</div>
          <div class="txt"><div class="t">${esc(st.name)}</div><div class="a">${esc(st.tags || st.country)}</div></div></div>`).join('')
      : '<div class="q-empty">Speichere Sender mit ★, dann wechselst du hier und mit Weiter/Zurück zwischen ihnen.</div>';
    return;
  }
  const up = await invoke('upcoming', { limit: 60 }).catch(() => []);
  $('#queueList').innerHTML = up.length
    ? up.map((t) => queueItem(t)).join('')
    : '<div class="q-empty">Danach ist Schluss.</div>';
}, 80);





// ---------- Sleep-Timer ----------

function sleepMenu() {
  const r = $('#sleep').getBoundingClientRect();
  const active = state.status?.sleep_remaining_s != null;
  const items = [15, 30, 45, 60, 90].map((m) => ({ label: tx('In {m} Minuten pausieren', { m }), icon: 'moon', action: () => send('sleep_timer', { minutes: m }).then(() => toast(tx('Sleep-Timer: {m} Minuten', { m }), 'ok')) }));
  if (active) items.push('sep', { label: 'Timer ausschalten', icon: 'x', action: () => send('sleep_timer', { minutes: null }) });
  showMenu(r.right + 8, r.top - 120, items);
}

// ---------- Ereignisse der Leiste ----------

function bindBar() {
  $('#play').addEventListener('click', () => send('toggle'));
  $('#next').addEventListener('click', () => send('next'));
  $('#prev').addEventListener('click', () => send('prev'));
  $('#shuffle').addEventListener('click', () => send('set_shuffle', { on: null }));
  $('#repeat').addEventListener('click', () => send('set_repeat', { mode: null }));
  $('#mute').addEventListener('click', () => send('toggle_mute'));
  $('#queueList').addEventListener('click', async (e) => {
    const id = e.target.closest('[data-station]')?.dataset.station;
    if (!id) return;
    const saved = await invoke('stations');
    const st = saved.find((x) => x.id === Number(id));
    if (st) playStation(st);
  });
  $('#clearQueue').addEventListener('click', async () => {
    await send('clear_up_next');
    renderUpNext();
  });
  $('#sleep').addEventListener('click', (e) => {
    e.stopPropagation();
    sleepMenu();
  });
  $('#barCover').addEventListener('click', () => {
    const t = state.status?.track;
    if (t?.album) navigate({ name: 'album', album: t.album, artist: t.album_artist || t.artist });
  });
  $('#barFav').addEventListener('click', () => state.status?.track && toggleFavorite(state.status.track).then(() => send('refresh')));
  $('#barArtist').addEventListener('click', () => {
    const t = state.status?.track;
    if (t?.artist) navigate({ name: 'artist', artist: t.album_artist || t.artist });
  });

  const seek = $('#seek');
  seek.addEventListener('input', () => {
    state.seeking = true;
    setRange(seek, seek.value);
    const d = state.status?.duration_ms || 0;
    $('#pos').textContent = fmtTime((seek.value / 1000) * d);
  });
  seek.addEventListener('change', async () => {
    const d = state.status?.duration_ms || 0;
    await send('seek', { ms: Math.round((seek.value / 1000) * d) });
    state.seeking = false;
  });

  const vol = $('#volume');
  const setVol = throttle((v) => send('set_volume', { volume: v }), 80);
  vol.addEventListener('input', () => {
    setRange(vol, vol.value);
    setVol(Number(vol.value));
  });
  vol.addEventListener('wheel', (e) => {
    e.preventDefault();
    const v = Math.max(0, Math.min(100, Number(vol.value) + (e.deltaY < 0 ? 5 : -5)));
    setRange(vol, v);
    setVol(v);
  }, { passive: false });
}

// ---------- Online-Quellen (Quellen-Plugins) ----------

const online = { sources: [], source: localStorage.getItem('onlineSource') || '', query: '', results: [], busy: false };

async function loadSources() {
  const info = await invoke('sources_list').catch(() => ({ sources: [], errors: [] }));
  online.sources = info.sources;
  online.errors = info.errors;
  if (!online.sources.some((s) => s.id === online.source)) online.source = online.sources[0]?.id || '';
  $('#onlineTab').hidden = !online.sources.length;
  if ($('#bottomNav').childElementCount) renderBottomNav();
}

async function renderOnline(el, v, fresh) {
  const src = online.sources.find((s) => s.id === online.source);
  if (!src) {
    el.innerHTML = head({ kind: 'Online', title: 'Online' }) + emptyState({ ico: 'puzzle', title: 'Kein Quellen-Plugin installiert', text: 'Quellen-Plugins kommen in den Ordner „sources“ (Einstellungen › Plugins).' });
    return;
  }
  const chips = online.sources.length > 1
    ? `<span class="seg">${online.sources.map((s) => `<button data-src="${esc(s.id)}" class="${s.id === src.id ? 'on' : ''}">${esc(s.icon || '')} ${esc(s.name)}</button>`).join('')}</span>`
    : '';
  const playlistBtn = src.features.includes('playlist')
    ? `<button class="btn" data-act="import">${icon('import')} ${esc(tx('{name}-Playlist übernehmen', { name: src.name }))}</button>` : '';
  const actions = `<div class="radio-tools">${chips}
      <input class="field" id="onlineQ" placeholder="Auf ${esc(src.name)} suchen …" value="${esc(online.query)}" spellcheck="false">
      <button class="btn primary" data-act="search">${icon('search')} Suchen</button>${playlistBtn}</div>`;
  el.innerHTML = head({ kind: 'Online', title: src.name, sub: `${esc((src.description || '').replace(/\.?$/, '.'))} ${tx('Gespielte Titel landen im Verlauf, Favoriten und Playlisten funktionieren wie gewohnt.')}`, actions });
  const body = document.createElement('div');
  body.style.cssText = 'display:flex;flex-direction:column;flex:1;min-height:0';
  el.append(body);
  const show = () => {
    body.innerHTML = '';
    if (online.busy) {
      body.innerHTML = `<div class="empty"><div class="big-ico">${icon('search')}</div><h2>Suche läuft …</h2><p>${esc(tx('{name} wird gefragt – das dauert ein paar Sekunden.', { name: src.name }))}</p></div>`;
    } else if (online.results.length) {
      state.list = new TrackList(body, online.results, { showAlbum: false });
    } else {
      body.innerHTML = emptyState({ ico: 'search', title: tx('{name} durchsuchen', { name: src.name }), text: 'Interpret oder Titel eingeben und Enter drücken. Doppelklick spielt ab, Rechtsklick für Warteschlange, Favoriten und Playlisten.' });
    }
  };
  const search = async () => {
    const q = $('#onlineQ', el).value.trim();
    if (!q || online.busy) return;
    online.query = q;
    online.busy = true;
    show();
    try {
      online.results = await invoke('source_search', { source: src.id, query: q });
    } catch (e) {
      online.results = [];
      toast(String(e), 'err');
    }
    online.busy = false;
    if (fresh()) show();
  };
  $('#onlineQ', el).addEventListener('keydown', (e) => e.key === 'Enter' && search());
  el.addEventListener('click', async (e) => {
    const s = e.target.closest('[data-src]')?.dataset.src;
    if (s) {
      online.source = s;
      localStorage.setItem('onlineSource', s);
      online.results = [];
      return render();
    }
    const act = e.target.closest('[data-act]')?.dataset.act;
    if (act === 'search') search();
    if (act === 'import') importOnlinePlaylist(src);
  });
  show();
}

async function importOnlinePlaylist(src) {
  const url = await dialog({ title: tx('{name}-Playlist übernehmen', { name: src.name }), text: 'Link der Playlist einfügen (z. B. aus „Teilen › Link kopieren“). Die Titel werden gestreamt, nicht gespeichert.', input: '', ok: 'Übernehmen' });
  if (!url) return;
  toast('Playlist wird gelesen …');
  const id = await call('source_import', { source: src.id, url });
  await loadPlaylists();
  toast('Playlist übernommen', 'ok');
  navigate({ name: 'playlist', id });
}

// ---------- Internetradio ----------

const COUNTRIES = [['', 'Alle Länder'], ['DE', 'Deutschland'], ['AT', 'Österreich'], ['CH', 'Schweiz'], ['NL', 'Niederlande'], ['GB', 'Großbritannien'], ['US', 'USA'], ['FR', 'Frankreich'], ['IT', 'Italien'], ['ES', 'Spanien'], ['PL', 'Polen'], ['TR', 'Türkei']];

/** Senderlogo oder – wenn keins da ist bzw. nicht lädt – ein farbiges Radiosymbol. */
function stationLogo(st) {
  const ph = `<div class="ph-radio" style="background:${hueGradient(st.name)}">${icon('radio')}</div>`;
  if (!st.favicon) return ph;
  return `<img src="${esc(st.favicon)}" alt="" loading="lazy" referrerpolicy="no-referrer" draggable="false"
    onerror="this.outerHTML=decodeURIComponent('${encodeURIComponent(ph)}')">`;
}

const radio = { results: [], saved: [] };

function playStation(st) {
  toast(tx('Verbinde mit „{name}“ …', { name: st.name }));
  return send('play_station', { station: st });
}

function markRadio() {
  const url = !state.status?.track ? state.status?.radio?.station.url : null;
  for (const el of $$('[data-url]')) el.classList.toggle('on', el.dataset.url === url);
}

async function renderRadio(el, v, fresh) {
  radio.saved = await invoke('stations');
  if (!fresh()) return;
  const country = localStorage.getItem('radioCountry') ?? 'DE';
  const actions = `<div class="radio-tools">
      <input class="field" id="radioQ" placeholder="Sender suchen …" spellcheck="false">
      <select class="select" id="radioC">${COUNTRIES.map(([c, n]) => `<option value="${c}"${c === country ? ' selected' : ''}>${n}</option>`).join('')}</select>
      <button class="btn" data-act="own">${icon('plus')} Eigene Adresse</button>
    </div>`;
  el.innerHTML = head({ kind: 'Internetradio', title: 'Radio', sub: 'Über 50.000 Sender aus radio-browser.info. Mit ★ speichern – Weiter/Zurück wechselt dann zwischen deinen Sendern.', actions }) +
    `<div class="scroll pad"><div id="radioMine"></div>
      <div class="sec-title">Sender finden <small id="radioInfo"></small></div>
      <div class="st-list" id="radioResults"></div></div>`;
  renderMine();
  const search = async () => {
    const q = $('#radioQ', el).value.trim();
    const c = $('#radioC', el).value;
    localStorage.setItem('radioCountry', c);
    $('#radioInfo', el).textContent = 'lädt …';
    try {
      radio.results = await invoke('radio_search', { query: q, country: c });
    } catch (e) {
      radio.results = [];
      $('#radioInfo', el).textContent = '';
      $('#radioResults', el).innerHTML = `<p class="hint">${esc(String(e))}</p>`;
      return;
    }
    if (!fresh()) return;
    $('#radioInfo', el).textContent = q ? tx('{n} Treffer', { n: fmtNum(radio.results.length) }) : tx('beliebteste Sender');
    renderResults();
  };
  $('#radioQ', el).addEventListener('input', debounce(search, 350));
  $('#radioC', el).addEventListener('change', search);
  el.addEventListener('click', async (e) => {
    if (e.target.closest('[data-act="own"]')) return addOwnStation();
    const star = e.target.closest('[data-star]');
    if (star) {
      e.stopPropagation();
      return toggleSaved(radio.results[Number(star.dataset.star)]);
    }
    const res = e.target.closest('[data-res]');
    if (res) return playStation(radio.results[Number(res.dataset.res)]);
    const mine = e.target.closest('[data-mine]');
    if (mine) return playStation(radio.saved.find((s) => s.id === Number(mine.dataset.mine)));
  });
  el.addEventListener('contextmenu', (e) => {
    const mine = e.target.closest('[data-mine]');
    if (!mine) return;
    const st = radio.saved.find((s) => s.id === Number(mine.dataset.mine));
    showMenu(e.clientX, e.clientY, [
      { label: 'Abspielen', icon: 'play', action: () => playStation(st) },
      { label: 'Aus „Meine Sender“ entfernen', icon: 'trash', danger: true, action: () => toggleSaved(st) },
    ]);
  });
  search();
}

function renderMine() {
  const box = $('#radioMine');
  if (!box) return;
  box.innerHTML = `<div class="sec-title">Meine Sender <small>${radio.saved.length ? (isTouch ? 'Lange drücken zum Entfernen' : 'Rechtsklick zum Entfernen') : ''}</small></div>` +
    (radio.saved.length
      ? `<div class="st-grid">${radio.saved.map((st) => `<div class="st-card" data-mine="${st.id}" data-url="${esc(st.url)}" title="${esc(st.name)}">
          <div class="logo">${stationLogo(st)}</div>
          <div class="name">${esc(st.name)}</div><div class="info">${esc(st.tags || st.country || st.url)}</div></div>`).join('')}</div>`
      : '<p class="hint">Noch keine Sender gespeichert – klick unten bei einem Sender auf ★.</p>');
  markRadio();
}

function renderResults() {
  const box = $('#radioResults');
  if (!box) return;
  const savedUrls = new Set(radio.saved.map((s) => s.url));
  box.innerHTML = radio.results.length
    ? radio.results.map((st, i) => `<div class="st-row" data-res="${i}" data-url="${esc(st.url)}" style="cursor:pointer">
        <div class="logo">${stationLogo(st)}</div>
        <div style="min-width:0"><div class="t">${esc(st.name)}</div><div class="a">${esc([st.country, st.codec, st.bitrate ? `${st.bitrate} kbps` : ''].filter(Boolean).join(' · '))}</div></div>
        <span class="tags">${esc(st.tags)}</span>
        <button class="star-btn${savedUrls.has(st.url) ? ' on' : ''}" data-star="${i}" title="${esc(tx('In „Meine Sender“ speichern'))}">${icon('star')}</button></div>`).join('')
    : '<p class="hint">Keine Sender gefunden.</p>';
  markRadio();
}

async function toggleSaved(st) {
  const existing = radio.saved.find((s) => s.url === st.url);
  if (existing) {
    await call('remove_station', { id: existing.id });
    toast(tx('„{name}“ entfernt', { name: st.name }), 'ok');
  } else {
    await call('add_station', { station: { ...st, id: 0 } });
    toast(tx('„{name}“ gespeichert', { name: st.name }), 'ok');
  }
  radio.saved = await invoke('stations');
  renderMine();
  renderResults();
  renderUpNext();
}

async function addOwnStation() {
  const url = await dialog({ title: 'Eigene Sender-Adresse', text: 'Stream-Adresse des Senders (beginnt mit http:// oder https://, auch .m3u oder .pls).', input: '', ok: 'Weiter' });
  if (!url) return;
  const name = await dialog({ title: 'Name des Senders', input: '', ok: 'Speichern' });
  await call('add_station', { station: { id: 0, name: name || url, url } });
  radio.saved = await invoke('stations');
  renderMine();
  toast('Sender gespeichert', 'ok');
}

// ---------- Streams fürs Handy ----------

/** Zeigt Adresse und QR-Code einer Stream-Liste für Poweramp. */
async function streamDialog(link) {
  const svg = await invoke('qr_svg', { text: link.url });
  const modal = $('#modal');
  modal.innerHTML = `<div class="dialog wide" role="dialog">
      <h3>${esc(tx('„{name}“ aufs Handy', { name: link.name }))}</h3>
      <div class="qr-wrap"><div class="qr">${svg}</div>
        <ol class="steps">
          <li>${esc(tx('QR-Code mit der Handykamera scannen und den Link öffnen – die Datei „{name}.m3u“ wird heruntergeladen.', { name: link.name }))}</li>
          <li>${tx('Die Datei antippen und <b>mit Poweramp öffnen</b>.')}</li>
          <li>${tx('Die Titel erscheinen in Poweramp unter <b>Streams</b> und spielen direkt vom PC.')}</li>
        </ol></div>
      <p class="hint" style="color:var(--accent)">${tx('Wichtig: Die Adresse <b>nicht</b> in Poweramp unter „URL hinzufügen“ eintragen – dort kann Poweramp keine Listen lesen, nur einzelne Streams.')}</p>
      ${copyField(link.url)}
      <p class="hint">Handy und PC müssen im selben WLAN sein, und Amber Player muss laufen.</p>
      <div class="actions"><button class="btn primary" data-r="ok">Fertig</button></div>
    </div>`;
  modal.classList.add('open');
  modal.onclick = async (e) => {
    const copy = e.target.closest('[data-copy]')?.dataset.copy;
    if (copy) {
      await navigator.clipboard.writeText(copy).then(() => toast('Kopiert', 'ok')).catch(() => {});
      return;
    }
    if (e.target.closest('[data-r]') || e.target === modal) {
      modal.classList.remove('open');
      modal.innerHTML = '';
    }
  };
}

/** Am PC: QR-Code zum Koppeln der Handy-App. */
async function pairDialog() {
  const info = await call('pair_link');
  if (!info.lan) return enableLan();
  const svg = await invoke('qr_svg', { text: info.link });
  const modal = $('#modal');
  modal.innerHTML = `<div class="dialog wide" role="dialog">
      <h3>Handy-App koppeln</h3>
      <div class="qr-wrap"><div class="qr">${svg}</div>
        <ol class="steps">
          <li>Amber Player aufs Handy installieren.</li>
          <li>Diesen QR-Code mit der Handykamera scannen und „Öffnen“ wählen – die App verbindet sich selbst.</li>
          <li>${tx('In der App unter <b>Mehr → PC</b> den PC steuern oder Musik vom PC aufs Handy holen.')}</li>
        </ol></div>
      <p class="hint">Klappt das Scannen nicht, den Link in der App unter „PC“ einfügen:</p>
      ${copyField(info.link)}
      <p class="hint">Handy und PC müssen im selben WLAN sein. Der Link enthält das Token – nicht weitergeben.</p>
      <div class="actions"><button class="btn primary" data-r="ok">Fertig</button></div>
    </div>`;
  modal.classList.add('open');
  modal.onclick = async (e) => {
    const copy = e.target.closest('[data-copy]')?.dataset.copy;
    if (copy) {
      await navigator.clipboard.writeText(copy).then(() => toast('Kopiert', 'ok')).catch(() => {});
      return;
    }
    if (e.target.closest('[data-r]') || e.target === modal) {
      modal.classList.remove('open');
      modal.innerHTML = '';
    }
  };
}

async function phoneStream({ playlistId }) {
  const info = await call('stream_links');
  if (!info.lan) return enableLan();
  const link = info.links.find((l) => l.playlist_id === playlistId);
  if (link) streamDialog(link);
}

async function enableLan() {
  const ok = await dialog({
    title: 'Im Heimnetz freigeben?',
    text: 'Damit dein Handy die Musik abrufen kann, muss Amber Player im Heimnetz erreichbar sein. Geräte im Netz brauchen dafür ein Token, das automatisch in den Adressen steckt. Amber Player startet dafür einmal neu.',
    ok: 'Freigeben und neu starten',
  });
  if (!ok) return;
  const api = await call('api_info');
  await call('api_configure', { lan: true, port: api.port, newToken: false });
  const fw = await invoke('firewall_status');
  if (fw.active && !fw.opened) {
    toast('Bitte im Passwortfenster die Firewall-Freigabe bestätigen');
    await invoke('open_firewall').catch((e) => toast(String(e), 'err'));
  }
  await invoke('restart_app');
}

// ---------- Visualisierung ----------

const BUILTIN_VIZ = ['balken', 'welle', 'ring', 'nebel'];

const viz = {
  open: false,
  worker: null,
  list: [],
  current: localStorage.getItem('viz') || 'balken',
  pending: false,
  last: 0,
  idleTimer: null,
  trackId: undefined,
};

/** CSS-Farbe → [r, g, b] (über eine Zeichenfläche, versteht auch hsl()). */
function cssRgb(value) {
  const c = document.createElement('canvas').getContext('2d');
  c.fillStyle = '#000';
  c.fillStyle = value;
  const v = c.fillStyle;
  if (v.startsWith('#')) {
    const n = parseInt(v.slice(1), 16);
    return [(n >> 16) & 255, (n >> 8) & 255, n & 255];
  }
  return (v.match(/[\d.]+/g) || [0, 0, 0]).slice(0, 3).map(Number);
}

async function vizEnsure() {
  if (viz.worker) return true;
  const canvas = $('#vizCanvas');
  if (!canvas.transferControlToOffscreen) {
    $('#visual').insertAdjacentHTML('beforeend', '<div class="viz-msg">Diese Systemversion unterstützt keine Visualisierungen.</div>');
    return false;
  }
  const code = await (await fetch('viz-worker.js')).text();
  viz.worker = new Worker(URL.createObjectURL(new Blob([code], { type: 'text/javascript' })));
  viz.worker.onmessage = (e) => {
    const m = e.data;
    if (m.type === 'list') {
      viz.list = m.list;
      for (const err of m.errors) toast(`${tx('Visualisierung')} ${err.source || ''}: ${err.message}`, 'err');
      vizModes();
      viz.worker.postMessage({ type: 'use', id: viz.current });
      vizResize();
      vizInfo(true);
    } else if (m.type === 'using') {
      viz.current = m.id;
      vizModes();
    } else if (m.type === 'error') {
      const meta = viz.list.find((x) => x.id === m.id);
      toast(tx('Visualisierung „{name}“ hat einen Fehler: {msg}', { name: meta?.name || m.id, msg: m.message }), 'err');
    }
  };
  const off = canvas.transferControlToOffscreen();
  viz.worker.postMessage({ type: 'init', canvas: off }, [off]);
  new ResizeObserver(vizResize).observe($('#visual'));
  await vizLoad();
  return true;
}

/** Mitgelieferte und eigene Visualisierungen (neu) laden. */
async function vizLoad() {
  if (!viz.worker) return;
  const builtin = await Promise.all(
    BUILTIN_VIZ.map(async (n) => ({ source: '', code: await (await fetch(`visualizers/${n}.js`)).text() })),
  );
  const [own, pk] = await Promise.all([
    invoke('visualizer_plugins'),
    invoke('plugin_packages').catch(() => ({ packages: [] })),
  ]);
  viz.dir = own.dir;
  viz.packages = pk.packages || [];
  addPluginTexts(viz.packages);
  viz.worker.postMessage({ type: 'load', plugins: [...builtin, ...own.plugins.map((p) => ({ source: p.file, code: p.code }))] });
}

function vizStars() {
  try {
    const v = JSON.parse(localStorage.getItem('vizStars') || '[]');
    return Array.isArray(v) ? v : [];
  } catch {
    return [];
  }
}

function vizToggleStar(id) {
  const stars = vizStars();
  const i = stars.indexOf(id);
  if (i >= 0) stars.splice(i, 1);
  else stars.push(id);
  try {
    localStorage.setItem('vizStars', JSON.stringify(stars));
  } catch {}
  vizModes();
  vizFilter();
}

function vizModes() {
  const stars = vizStars();
  // Gruppen: Favoriten (mit Stern), mitgeliefert, je Plugin-Paket, einzelne Dateien
  const groups = new Map([
    ['stars', { label: tx('Favoriten'), title: '', items: [] }],
    ['builtin', { label: tx('Mitgeliefert'), title: '', items: [] }],
  ]);
  viz.list.forEach((v, i) => {
    const o = pluginOrigin(v.source, viz.packages);
    const key = stars.includes(v.id) ? 'stars' : o ? o.key : 'builtin';
    if (!groups.has(key)) groups.set(key, { label: o.label, title: o.title, items: [] });
    groups.get(key).items.push({ v, i, o });
  });
  const cur = viz.list.find((v) => v.id === viz.current) || viz.list[0];
  const curOrigin = cur && pluginOrigin(cur.source, viz.packages);
  const item = ({ v, i, o }, starred) => {
    const sub = [starred ? (o ? o.label : tx('Mitgeliefert')) : '', v.author].filter(Boolean).join(' · ');
    const find = [v.name, tx(v.name), v.author, o?.label, v.description].filter(Boolean).join(' ').toLowerCase();
    return `
    <div class="viz-item${v.id === cur?.id ? ' on' : ''}" data-viz="${esc(v.id)}" data-find="${esc(find)}" role="menuitemradio" aria-checked="${v.id === cur?.id}" title="${esc(tx(v.description || ''))}">
      <span class="k">${i < 9 ? i + 1 : ''}</span>
      <span class="n" data-raw>${esc(tx(v.name))}${sub ? `<small>${esc(sub)}</small>` : ''}</span>
      ${v.id === cur?.id ? icon('check') : ''}
      <button class="viz-star${starred ? ' on' : ''}" data-star="${esc(v.id)}" title="${esc(starred ? tx('Stern entfernen') : tx('Mit Stern nach oben'))}">${icon('star')}</button>
    </div>`;
  };
  const menu = [...groups.entries()]
    .filter(([, g]) => g.items.length)
    .map(([key, g]) => `<div class="viz-group"><div class="h" data-raw title="${esc(g.title)}">${key === 'stars' ? icon('star') : ''}${esc(g.label)}</div>${g.items.map((x) => item(x, key === 'stars')).join('')}</div>`)
    .join('');
  const box = $('#vizModes');
  const query = viz.query || '';
  const auto = vizAutoCfg();
  box.innerHTML = `
    <button class="viz-pick" aria-haspopup="menu" aria-expanded="${box.classList.contains('open')}" title="${esc(tx('Visualisierung wählen'))}">
      <span class="dot">${icon('activity')}</span>
      <span class="txt"><b data-raw>${esc(tx(cur?.name || ''))}</b><small data-raw title="${esc(curOrigin?.title || '')}">${esc(curOrigin ? curOrigin.label : tx('Mitgeliefert'))}</small></span>
      ${icon('chevron-down')}
    </button>
    <div class="viz-menu" role="menu">
      <label class="viz-search">${icon('search')}<input type="search" placeholder="${esc(tx('Suchen …'))}" value="${esc(query)}" spellcheck="false" autocomplete="off"></label>
      <div class="viz-groups">${menu}</div>
      <p class="viz-none" hidden>${esc(tx('Nichts gefunden.'))}</p>
      <div class="viz-auto">
        <span class="lbl">${icon('shuffle')} ${esc(tx('Automatisch wechseln'))}</span>
        <select data-auto="mode">
          ${[['off', 'Aus'], ['order', 'Der Reihe nach'], ['shuffle', 'Zufällig'], ['stars', 'Zufällig aus Favoriten']]
            .map(([k, l]) => `<option value="${k}"${auto.mode === k ? ' selected' : ''}>${esc(tx(l))}</option>`).join('')}
        </select>
        <select data-auto="every"${auto.mode === 'off' ? ' disabled' : ''}>
          ${[[15, '15 s'], [30, '30 s'], [60, '1 min'], [120, '2 min'], [300, '5 min'], [0, tx('bei jedem neuen Titel')]]
            .map(([k, l]) => `<option value="${k}"${auto.every === k ? ' selected' : ''}>${esc(l)}</option>`).join('')}
        </select>
      </div>
    </div>`;
  vizFilter();
}

/** Einträge nach dem Suchtext ein-/ausblenden (ohne neu zu zeichnen – das Feld behält den Fokus). */
function vizFilter() {
  const box = $('#vizModes');
  const q = (viz.query || '').trim().toLowerCase();
  let shown = 0;
  for (const g of $$('.viz-group', box)) {
    let n = 0;
    for (const it of $$('.viz-item', g)) {
      const ok = !q || it.dataset.find.includes(q);
      it.hidden = !ok;
      n += ok;
    }
    g.hidden = !n;
    shown += n;
  }
  const none = $('.viz-none', box);
  if (none) none.hidden = shown > 0;
}

function vizMenu(open) {
  const box = $('#vizModes');
  const on = open ?? !box.classList.contains('open');
  box.classList.toggle('open', on);
  $('.viz-pick', box)?.setAttribute('aria-expanded', String(on));
  if (on) {
    $('.viz-item.on', box)?.scrollIntoView({ block: 'nearest' });
    if (!isTouch) $('.viz-search input', box)?.focus();
    vizWake();
  } else if (viz.query) {
    viz.query = '';
    $('.viz-search input', box) && ($('.viz-search input', box).value = '');
    vizFilter();
  }
}

function vizUse(id) {
  viz.current = id;
  try { localStorage.setItem('viz', id); } catch {}
  vizModes();
  viz.worker?.postMessage({ type: 'use', id });
}

function vizResize() {
  if (!viz.worker) return;
  const r = $('#vizCanvas').getBoundingClientRect();
  viz.worker.postMessage({ type: 'resize', width: r.width, height: r.height, dpr: devicePixelRatio || 1 });
}

/** Titel, Cover und Akzentfarben an die Plugins geben. */
async function vizInfo(force = false) {
  if (!viz.worker) return;
  const t = state.status?.track;
  const rad = !t ? state.status?.radio : null;
  const key = t ? t.id : rad ? `${rad.station.url}|${rad.title}` : null;
  if (!force && key === viz.trackId) return;
  const changed = viz.trackId != null && key !== viz.trackId;
  viz.trackId = key;
  if (changed && !force) {
    const auto = vizAutoCfg();
    if (auto.mode !== 'off' && auto.every === 0) vizAutoNext();
  }
  setMarquee($('#vizTitle'), t?.title || (rad ? rad.title || rad.station.name : ''));
  $('#vizArtist').textContent = t ? [t.artist, t.album].filter(Boolean).join(' · ') : rad ? `${rad.station.name} · ${tx('Internetradio')}` : '';
  let cover = null;
  if (t?.has_cover) {
    try {
      const img = new Image();
      img.crossOrigin = 'anonymous';
      img.src = coverUrl(t.id);
      await img.decode();
      cover = await createImageBitmap(img);
    } catch (e) {
      cover = null;
    }
  } else if (rad?.station.favicon) {
    // Radio: Senderlogo in die Mitte (z. B. beim Ring). Geladen über den Kern, weil viele
    // Sender-Server fremde Seiten aussperren; als Blob ist das Bild dann „sauber“ nutzbar.
    let url = '';
    try {
      const buf = await invoke('station_logo', { url: rad.station.favicon });
      url = URL.createObjectURL(new Blob([buf]));
      const img = new Image();
      img.src = url;
      await img.decode();
      cover = await createImageBitmap(img);
    } catch (e) {
      cover = null;
    } finally {
      if (url) URL.revokeObjectURL(url);
    }
    if (viz.trackId !== key) return; // inzwischen anderer Sender
  }
  const css = getComputedStyle(document.documentElement);
  const info = {
    title: t?.title || rad?.title || rad?.station.name || '',
    artist: t?.artist || rad?.station.name || '',
    album: t?.album || '',
    cover,
    accent: cssRgb(css.getPropertyValue('--accent').trim()),
    accent2: cssRgb(css.getPropertyValue('--accent-2').trim()),
  };
  viz.worker.postMessage({ type: 'info', info }, cover ? [cover] : []);
}

function vizLoop(now) {
  if (!viz.open) return;
  requestAnimationFrame(vizLoop);
  if (viz.pending || !viz.worker) return;
  viz.pending = true;
  const dt = Math.min(0.1, (now - (viz.last || now)) / 1000);
  viz.last = now;
  const playing = state.status?.state === 'playing';
  invoke('visual_frame', { bands: 64, waveLen: 1024 })
    .then((f) => {
      if (!playing) {
        f.bands = f.bands.map(() => 0);
        f.wave = f.wave.map(() => 0);
        f.level = 0;
      }
      viz.worker.postMessage({ type: 'frame', frame: { ...f, time: now / 1000, dt, playing } });
    })
    .catch(() => {})
    .finally(() => (viz.pending = false));
}

/** Nach wie vielen Millisekunden die Bedienelemente über der Visualisierung ausblenden (0 = nie). */
function vizHideMs() {
  let sec = 6;
  try {
    const v = localStorage.getItem('vizHide');
    if (v !== null && !Number.isNaN(Number(v))) sec = Number(v);
  } catch {}
  return sec * 1000;
}

/** Ausblenden verschieben, solange das Auswahlmenü offen ist. */
function vizLater(fn) {
  const ms = vizHideMs();
  if (!ms) return null;
  const run = () => ($('#vizModes').classList.contains('open') ? (viz.idleTimer = setTimeout(run, ms)) : fn());
  return setTimeout(run, ms);
}

function vizWake() {
  $('#visual').classList.remove('idle');
  clearTimeout(viz.idleTimer);
  viz.idleTimer = vizLater(() => $('#visual').classList.add('idle'));
}

async function toggleViz(open = !viz.open, layer = 'full') {
  if (open && !(await vizEnsure())) {
    $('#visual').classList.add('open');
    viz.open = true;
    vizLayer(layer);
    return;
  }
  viz.open = open;
  $('#visual').classList.toggle('open', open);
  $('#vizBtn').classList.toggle('on', open);
  vizLayer(layer);
  vizAutoStart();
  if (open) {
    vizResize();
    vizInfo(true);
    vizWake();
    viz.last = 0;
    requestAnimationFrame(vizLoop);
  } else if (IS_ANDROID) {
    // Systemleisten wieder so, wie es in den Einstellungen steht
    if (viz.bars !== undefined) {
      viz.bars = undefined;
      const r = await invoke('plugin:amber-mobile|get_fullscreen').catch(() => null);
      invoke('plugin:amber-mobile|set_fullscreen', { on: r?.on !== false, remember: false }).catch(() => {});
    }
  } else {
    const win = window.__TAURI__.window?.getCurrentWindow?.();
    if (win && (await win.isFullscreen().catch(() => false))) win.setFullscreen(false);
  }
}

/** Handy in „Läuft gerade“: aus → hinter den Bedienelementen → Vollbild → aus (wie bei Poweramp). */
function vizCycle() {
  if (!(isMobile() && document.body.classList.contains('deck-open'))) return toggleViz();
  if (!viz.open) return toggleViz(true, 'behind');
  if (viz.layer === 'behind') return vizLayer('full');
  return toggleViz(false);
}

/** `behind`: Visualisierung liegt hinter „Läuft gerade“, `full`: eigene Ansicht darüber. */
function vizLayer(layer) {
  viz.layer = layer;
  document.body.classList.toggle('viz-behind', viz.open && layer === 'behind');
  document.body.classList.toggle('viz-full', viz.open && layer === 'full');
  vizPlaceCanvas();
  vizDimSoon();
  if (viz.open) {
    vizWake();
    vizResize();
  }
}

/**
 * Hinter „Läuft gerade“ zeichnet die Visualisierung dort, wo sonst das Cover ist:
 * hochkant oben über die ganze Breite, quer in der linken Spalte.
 */
function vizPlaceCanvas() {
  const c = $('#vizCanvas').style;
  c.top = c.left = c.width = c.height = '';
  if (viz.open && viz.layer === 'behind') {
    const r = $('#deck .disc-wrap').getBoundingClientRect();
    if (r.height) {
      if (r.right < innerWidth * 0.6) {
        c.width = `${Math.round(r.right + 24)}px`;
      } else {
        c.height = `${Math.round(r.bottom + 16)}px`;
      }
    }
  }
  vizResize();
}

/** Bedienelemente über der Visualisierung nach ein paar Sekunden ausblenden. */
function vizDimSoon() {
  clearTimeout(viz.dimTimer);
  document.body.classList.remove('viz-dim');
  if (viz.open && viz.layer === 'behind') {
    viz.dimTimer = vizLater(() => document.body.classList.add('viz-dim'));
  }
}

/** Nächste oder vorige Visualisierung, mit kurzer Namensanzeige. */
function vizStep(dir) {
  if (!viz.list?.length) return;
  const i = viz.list.findIndex((v) => v.id === viz.current);
  vizShow(viz.list[(i + dir + viz.list.length) % viz.list.length]);
}

/** Visualisierung wechseln und ihren Namen kurz einblenden. */
function vizShow(next) {
  vizUse(next.id);
  const label = $('#vizName');
  label.textContent = tx(next.name);
  label.classList.add('show');
  clearTimeout(viz.nameTimer);
  viz.nameTimer = setTimeout(() => label.classList.remove('show'), 1400);
}

// ---------- Automatischer Wechsel (wie „Nach Dauer“/„Zufällig“ bei anderen Playern)

/** { mode: 'off' | 'order' | 'shuffle' | 'stars', every: Sekunden (0 = bei jedem neuen Titel) } */
function vizAutoCfg() {
  try {
    const c = JSON.parse(localStorage.getItem('vizAuto') || 'null');
    if (c && typeof c.mode === 'string') return { mode: c.mode, every: Number(c.every) || 0 };
  } catch {}
  return { mode: 'off', every: 60 };
}

function setVizAuto(cfg) {
  try {
    localStorage.setItem('vizAuto', JSON.stringify(cfg));
  } catch {}
  vizAutoStart();
}

/** Nächste Visualisierung nach der gewählten Art. */
function vizAutoNext() {
  const cfg = vizAutoCfg();
  if (cfg.mode === 'off' || !viz.list?.length) return;
  const stars = vizStars();
  let pool = cfg.mode === 'stars' ? viz.list.filter((v) => stars.includes(v.id)) : viz.list;
  if (pool.length < 2) pool = viz.list;
  if (pool.length < 2) return;
  const i = pool.findIndex((v) => v.id === viz.current);
  let next;
  if (cfg.mode === 'order') {
    next = pool[(i + 1) % pool.length];
  } else {
    // zufällig, aber nie dieselbe noch einmal
    do next = pool[Math.floor(Math.random() * pool.length)];
    while (next.id === viz.current);
  }
  vizShow(next);
}

function vizAutoStart() {
  clearInterval(viz.autoTimer);
  const cfg = vizAutoCfg();
  if (viz.open && cfg.mode !== 'off' && cfg.every > 0) viz.autoTimer = setInterval(vizAutoNext, cfg.every * 1000);
}

async function toggleFullscreen() {
  if (IS_ANDROID) {
    // Am Handy gibt es kein Fenster: Status- und Navigationsleiste ein-/ausblenden, nur vorübergehend
    try {
      if (viz.bars === undefined) viz.bars = (await invoke('plugin:amber-mobile|get_fullscreen'))?.on !== false;
      viz.bars = !viz.bars;
      await invoke('plugin:amber-mobile|set_fullscreen', { on: viz.bars, remember: false });
    } catch (e) {
      toast(tx('Vollbild nicht möglich: {e}', { e: e?.message || String(e) }), 'err');
    }
    return;
  }
  const win = window.__TAURI__.window?.getCurrentWindow?.();
  if (!win) return;
  try {
    await win.setFullscreen(!(await win.isFullscreen()));
  } catch (e) {
    toast(tx('Vollbild nicht möglich: {e}', { e: e?.message || String(e) }), 'err');
  }
}


// ---------- PC: Fernsteuern und vom PC streamen (Handy) ----------

const pc = { timer: null, coverId: null, coverUrl: '', playlists: [], results: [] };

/** Kopplung über einen Link (QR-Code am PC) oder eine eingetippte Adresse. */
async function pcPair(link) {
  try {
    await call('pc_pair', { link });
    toast('Mit dem PC verbunden', 'ok');
    navigate({ name: 'pc' });
  } catch {
    // Meldung zeigt call() an.
  }
}

/**
 * Am Touchscreen springt ein Regler schon beim Aufsetzen des Fingers an die Stelle. Wer nur über
 * ihn hinweg scrollen wollte (Finger bewegt sich senkrecht), bekommt den alten Wert zurück.
 */
function guardRangesOnTouch() {
  if (!isTouch) return;
  let g = null;
  document.addEventListener('pointerdown', (e) => {
    const r = e.target.closest?.('input[type=range]');
    g = r ? { r, x: e.clientX, y: e.clientY, v: r.value, scroll: false } : null;
  }, true);
  document.addEventListener('pointermove', (e) => {
    if (!g || g.scroll) return;
    const dx = Math.abs(e.clientX - g.x);
    const dy = Math.abs(e.clientY - g.y);
    if (dy > 10 && dy > dx * 1.2) g.scroll = true;
  }, true);
  document.addEventListener('pointercancel', () => g && (g.scroll = true), true);
  const restore = (e) => {
    if (!g || !g.scroll || e.target !== g.r) return;
    e.stopImmediatePropagation();
    if (e.type === 'change' || e.type === 'pointerup') {
      const { r, v } = g;
      g = null;
      if (r.value !== v) {
        r.value = v;
        // alten Wert wieder an alle melden (z. B. Lautstärke zurück)
        r.dispatchEvent(new Event('input', { bubbles: true }));
      }
    }
  };
  document.addEventListener('input', restore, true);
  document.addEventListener('change', restore, true);
  document.addEventListener('pointercancel', (e) => {
    if (g?.r && g.r.value !== g.v) {
      const { r, v } = g;
      g = null;
      r.value = v;
      r.dispatchEvent(new Event('input', { bubbles: true }));
    }
  }, true);
}

/** Titel aus Online-Quellen des PCs (z. B. eigene Quellen-Plugins) kann das Handy nicht abspielen. */
const pcPlayable = (t) => !String(t?.path || '').startsWith('plugin://');

/** QR-Code vom PC mit der Kamera scannen und koppeln (Android). */
async function scanPairCode() {
  const P = 'plugin:barcode-scanner|';
  try {
    let perm = await invoke(`${P}check_permissions`).catch(() => null);
    if (perm?.camera !== 'granted') perm = await invoke(`${P}request_permissions`);
    if (perm?.camera !== 'granted') {
      toast(tx('Ohne Kamera-Erlaubnis kann Amber Player nicht scannen'), 'err');
      return;
    }
    document.body.classList.add('scanning');
    const r = await invoke(`${P}scan`, { windowed: true, formats: ['QR_CODE'] });
    document.body.classList.remove('scanning');
    const text = String(r?.content || '').trim();
    if (!text.startsWith('amberplayer://') && !/^https?:\/\//.test(text)) {
      toast(tx('Das ist kein Kopplungs-Code von Amber Player'), 'err');
      return;
    }
    pcPair(text);
  } catch (e) {
    document.body.classList.remove('scanning');
    if (!/cancel/i.test(String(e?.message || e))) toast(tx('Scannen nicht möglich: {e}', { e: e?.message || String(e) }), 'err');
  }
}

async function renderPc(el, v, fresh) {
  clearInterval(pc.timer);
  const info = await invoke('pc_info');
  if (!fresh()) return;
  if (!info) {
    el.innerHTML = `${head({ kind: 'Verbindung', title: 'PC' })}
      <div class="scroll pad"><div class="settings"><div class="box">
        <h3>Mit dem PC koppeln</h3>
        <p>${tx('Am PC in Amber Player unter <b>Einstellungen → Fernsteuerung</b> „Handy koppeln“ wählen und den QR-Code mit der Handykamera abscannen – dann öffnet sich diese App und verbindet sich.')}</p>
        ${IS_ANDROID ? `<button class="btn primary pc-scan" id="pcScanBtn">${icon('camera')} ${esc(tx('QR-Code scannen'))}</button>` : ''}
        <p class="hint">Oder den Kopplungs-Link bzw. die Adresse hier einfügen:</p>
        <div style="display:flex;gap:10px;flex-wrap:wrap"><input class="field" id="pcLink" placeholder="amberplayer://pair?… oder 192.168.178.35:7878" spellcheck="false" style="flex:1;min-width:200px">
        <button class="btn primary" id="pcPairBtn">${icon('link')} Verbinden</button></div>
        <p class="hint">${esc(tx('PC und Handy müssen im selben WLAN sein, und am PC muss „Im Heimnetz“ eingeschaltet sein.'))} ${wikiLink('pc', tx('Anleitung im Wiki'))}</p>
      </div></div></div>`;
    $('#pcScanBtn', el)?.addEventListener('click', scanPairCode);
    $('#pcPairBtn', el).addEventListener('click', () => {
      const link = $('#pcLink', el).value.trim();
      if (link) pcPair(link);
    });
    return;
  }

  // „In die Leiste“ nur, wo es die Leiste unten gibt (Handy, Tablet hochkant)
  const inBar = !isMobile() || navChoice().includes('pc');
  const actions = inBar ? '' : `<button class="btn small" id="pcToBar">${icon('plus')} ${esc(tx('In die Leiste'))}</button>`;
  pc.tab = pc.tab || 'now';
  el.innerHTML = `${head({ kind: 'Verbunden mit', title: 'PC', sub: esc(info.url), actions })}
    <div class="scroll pad pc-view">
      <div class="pc-hero" id="pcHero"><p class="hint">${esc(tx('Verbinde …'))}</p></div>
      <div class="pc-tabs" id="pcTabs">
        ${[['now', 'Läuft', 'queue-add'], ['music', 'Musik', 'music'], ['fx', 'Effekte', 'sliders'], ['plugins', 'Plugins', 'puzzle']]
          .map(([k, label, ic]) => `<button data-tab="${k}"${pc.tab === k ? ' class="on"' : ''}>${icon(ic)}<span>${esc(tx(label))}</span></button>`)
          .join('')}
      </div>
      <div id="pcPane"></div>
      <div class="actions" style="margin-top:26px"><button class="btn small danger" id="pcUnpair">Verbindung trennen</button></div>
    </div>`;

  $('#pcToBar', el)?.addEventListener('click', () => {
    const list = navChoice();
    list[NAV_SLOTS - 1] = 'pc';
    setNavChoice(list);
    $('#pcToBar', el).remove();
    toast(tx('PC steht jetzt unten in der Leiste – ändern unter Einstellungen → Darstellung'), 'ok');
  });

  // ---------- Läuft gerade am PC (oben, bleibt stehen)
  const hero = $('#pcHero', el);
  let last = null;
  let dragging = false;
  const send = async (path) => {
    try {
      const s = await invoke('pc_post', { path });
      if (s && s.state) drawHero(s);
      return s;
    } catch (e) {
      toast(String(e), 'err');
    }
  };
  const buildHero = () => {
    hero.innerHTML = `
      <div class="pc-bg"></div>
      <div class="pc-top">
        <div class="pc-cover"></div>
        <div class="pc-meta"><div class="t"></div><div class="a"></div></div>
        <button class="icon-btn pc-fav" data-pc="/api/favorite" title="${esc(tx('Favorit'))}">${icon('heart')}</button>
      </div>
      <div class="pc-seek"><input type="range" min="0" max="1000" value="0" step="1"><div class="times"><span class="p">0:00</span><span class="d">0:00</span></div></div>
      <div class="pc-ctl">
        <button class="ctl" data-pc="/api/shuffle" data-k="shuffle" title="${esc(tx('Zufallswiedergabe'))}">${icon('shuffle')}</button>
        <button class="ctl" data-pc="/api/prev" title="${esc(tx('Zurück'))}">${icon('prev', true)}</button>
        <button class="ctl play" data-pc="/api/toggle" title="Play/Pause">${icon('play', true)}</button>
        <button class="ctl" data-pc="/api/next" title="${esc(tx('Weiter'))}">${icon('next', true)}</button>
        <button class="ctl" data-pc="/api/repeat" data-k="repeat" title="${esc(tx('Wiederholen'))}">${icon('repeat')}</button>
      </div>
      <div class="pc-vols">
        <label class="pc-vol"><span class="lbl">Amber</span>${icon('volume')}<input type="range" min="0" max="100" id="pcVol"><span class="n" id="pcVolN"></span></label>
        <label class="pc-vol" id="pcSysRow" hidden><span class="lbl">${esc(tx('PC gesamt'))}</span>${icon('monitor')}<input type="range" min="0" max="100" id="pcSys"><span class="n" id="pcSysN"></span></label>
      </div>`;
    for (const b of $$('[data-pc]', hero)) b.addEventListener('click', () => send(b.dataset.pc));
    const seek = $('.pc-seek input', hero);
    seek.addEventListener('pointerdown', () => (dragging = true));
    seek.addEventListener('input', () => {
      setRange(seek, seek.value);
      if (last?.duration_ms) $('.pc-seek .p', hero).textContent = fmtTime((seek.value / 1000) * last.duration_ms);
    });
    seek.addEventListener('change', async () => {
      if (last?.duration_ms) await send(`/api/seek?seconds=${((seek.value / 1000) * last.duration_ms / 1000).toFixed(1)}`);
      dragging = false;
    });
    const vol = $('#pcVol', hero);
    vol.addEventListener('pointerdown', () => (dragging = true));
    vol.addEventListener('input', () => {
      setRange(vol, vol.value);
      $('#pcVolN', hero).textContent = vol.value;
    });
    vol.addEventListener('change', async () => {
      await send(`/api/volume?value=${vol.value}`);
      dragging = false;
    });
    const sys = $('#pcSys', hero);
    sys.addEventListener('input', () => {
      setRange(sys, sys.value);
      $('#pcSysN', hero).textContent = sys.value;
    });
    sys.addEventListener('change', () => invoke('pc_post', { path: `/api/system/volume?value=${sys.value}` }).catch((e) => toast(String(e), 'err')));
  };
  const drawHero = async (s) => {
    if (!fresh() || !s) return;
    if (!$('.pc-top', hero)) buildHero();
    const trackChanged = (last?.track?.id ?? last?.radio?.title) !== (s.track?.id ?? s.radio?.title);
    last = s;
    const t = s.track;
    const r = !t ? s.radio : null;
    $('.pc-meta .t', hero).textContent = t ? t.title : r ? r.title || r.station.name : tx('Nichts läuft');
    $('.pc-meta .a', hero).textContent = t ? t.artist || tx('Unbekannter Interpret') : r ? `${r.station.name} · ${tx('Internetradio')}` : '';
    if (t?.has_cover && pc.coverId !== t.id) {
      pc.coverId = t.id;
      try {
        const buf = await invoke('pc_cover', { id: t.id });
        if (pc.coverUrl) URL.revokeObjectURL(pc.coverUrl);
        pc.coverUrl = URL.createObjectURL(new Blob([buf]));
      } catch {
        pc.coverUrl = '';
      }
    } else if (!t?.has_cover) {
      pc.coverId = null;
      pc.coverUrl = '';
    }
    $('.pc-cover', hero).innerHTML = pc.coverUrl ? `<img src="${pc.coverUrl}" alt="">` : icon(r ? 'radio' : 'monitor');
    $('.pc-bg', hero).style.backgroundImage = pc.coverUrl ? `url("${pc.coverUrl}")` : '';
    $('.pc-ctl .play', hero).innerHTML = icon(s.state === 'playing' ? 'pause' : 'play', true);
    $('[data-k="shuffle"]', hero).classList.toggle('on', !!s.shuffle);
    $('[data-k="repeat"]', hero).classList.toggle('on', s.repeat && s.repeat !== 'off');
    $('[data-k="repeat"]', hero).innerHTML = icon(s.repeat === 'one' ? 'repeat' : 'repeat') + (s.repeat === 'one' ? '<b class="one">1</b>' : '');
    $('.pc-fav', hero).classList.toggle('on', !!t?.favorite);
    $('.pc-fav', hero).hidden = !t;
    if (!dragging) {
      const seek = $('.pc-seek input', hero);
      seek.disabled = !s.duration_ms;
      const v = s.duration_ms ? Math.round((s.position_ms / s.duration_ms) * 1000) : 0;
      seek.value = v;
      setRange(seek, v);
      $('.pc-seek .p', hero).textContent = s.duration_ms ? fmtTime(s.position_ms) : r ? 'LIVE' : '0:00';
      $('.pc-seek .d', hero).textContent = s.duration_ms ? fmtTime(s.duration_ms) : '';
      const vol = $('#pcVol', hero);
      vol.value = s.volume;
      setRange(vol, s.volume);
      $('#pcVolN', hero).textContent = s.volume;
    }
    if (trackChanged && pc.tab === 'now') drawPane();
  };
  const pollSys = async () => {
    try {
      const sv = await invoke('pc_get', { path: '/api/system/volume' });
      const row = $('#pcSysRow', hero);
      if (!row) return;
      row.hidden = !sv.available;
      if (sv.available && document.activeElement !== $('#pcSys', hero)) {
        $('#pcSys', hero).value = sv.volume;
        setRange($('#pcSys', hero), sv.volume);
        $('#pcSysN', hero).textContent = sv.volume;
      }
    } catch {
      // ältere PC-Version ohne Systemlautstärke
    }
  };
  let tick = 0;
  const poll = async () => {
    if (!fresh() || document.hidden) return;
    try {
      await drawHero(await invoke('pc_get', { path: '/api/status' }));
      if (tick++ % 3 === 0) pollSys();
    } catch (e) {
      if (fresh()) hero.innerHTML = `<p class="hint">${esc(String(e))}</p>`;
    }
  };
  poll();
  pc.timer = setInterval(() => (fresh() ? poll() : clearInterval(pc.timer)), 2000);

  // ---------- Bereiche
  const pane = $('#pcPane', el);
  $('#pcTabs', el).addEventListener('click', (e) => {
    const b = e.target.closest('[data-tab]');
    if (!b) return;
    pc.tab = b.dataset.tab;
    for (const x of $$('#pcTabs button', el)) x.classList.toggle('on', x === b);
    drawPane();
  });

  // Titelliste: Tippen = auf dem Handy abspielen, Monitor-Knopf = am PC abspielen.
  const trackRows = (tracks) =>
    tracks.length
      ? `<div class="pc-list">${tracks
          .map((t, i) => `<div class="pc-row" data-i="${i}">
            <div class="pc-row-meta"><b>${esc(t.title)}</b><span>${esc(t.artist || tx('Unbekannter Interpret'))}</span></div>
            <span class="dur">${fmtTime(t.duration_ms)}</span>
            <button class="icon-btn" data-onpc="${t.id}" title="Am PC abspielen">${icon('monitor')}</button>
          </div>`)
          .join('')}</div>`
      : `<p class="hint">${esc(tx('Nichts gefunden.'))}</p>`;
  const bindRows = (box, tracks, album) => {
    for (const row of $$('.pc-row', box)) {
      row.addEventListener('click', (e) => {
        const onPc = e.target.closest('[data-onpc]');
        if (onPc) {
          send(`/api/play/track/${onPc.dataset.onpc}`);
          toast('Läuft jetzt am PC', 'ok');
          return;
        }
        call('pc_play_here', { tracks, start: Number(row.dataset.i), album }).then(applyStatus).catch(() => {});
      });
    }
  };

  const paneNow = async () => {
    pane.innerHTML = `<div class="sec-title">${esc(tx('Als Nächstes am PC'))}</div><div id="pcQueue"><p class="hint">${esc(tx('lädt …'))}</p></div>`;
    try {
      const q = (await invoke('pc_get', { path: '/api/queue' })).filter(pcPlayable);
      if (!fresh() || pc.tab !== 'now') return;
      $('#pcQueue', pane).innerHTML = q.length ? trackRows(q) : `<p class="hint">${esc(tx('Danach ist Schluss.'))}</p>`;
      bindRows($('#pcQueue', pane), q, 'Vom PC');
    } catch (e) {
      $('#pcQueue', pane).innerHTML = `<p class="hint">${esc(String(e))}</p>`;
    }
  };

  const paneMusic = async () => {
    pane.innerHTML = `
      <div class="radio-tools" style="margin-bottom:12px"><input class="field" id="pcQ" placeholder="${esc(tx('Titel, Interpret, Album auf dem PC suchen …'))}" spellcheck="false"></div>
      <div id="pcResults"></div>
      <div class="sec-title">${esc(tx('Playlisten am PC'))}</div>
      <div id="pcPlaylists"><p class="hint">${esc(tx('lädt …'))}</p></div>`;
    const results = $('#pcResults', pane);
    const search = debounce(async () => {
      const q = $('#pcQ', pane)?.value.trim();
      if (!q) return (results.innerHTML = '');
      try {
        const tracks = (await invoke('pc_get', { path: `/api/search?q=${encodeURIComponent(q)}&limit=100` })).filter(pcPlayable);
        if (fresh() && pc.tab === 'music') {
          results.innerHTML = trackRows(tracks);
          bindRows(results, tracks, 'Vom PC');
        }
      } catch (e) {
        results.innerHTML = `<p class="hint">${esc(String(e))}</p>`;
      }
    }, 300);
    $('#pcQ', pane).addEventListener('input', search);
    const plBox = $('#pcPlaylists', pane);
    try {
      // Playlisten nur aus Online-Titeln (von Quellen-Plugins am PC) weglassen – die spielt das Handy nicht
      pc.playlists = (await invoke('pc_get', { path: '/api/playlists' })).filter((p) => !(p.track_count > 0 && p.online_count >= p.track_count));
      if (!fresh() || pc.tab !== 'music') return;
      plBox.innerHTML = pc.playlists.length
        ? `<div class="pc-list">${pc.playlists
            .map((p) => `<div class="pc-row pc-pl" data-id="${p.id}">
              <div class="pc-row-meta"><b>${esc(p.name)}</b><span>${plural(p.track_count, 'Titel', 'Titel')}</span></div>
              <button class="btn small" data-here="${p.id}">${icon('phone')} Hier</button>
              <button class="btn small" data-there="${p.id}">${icon('monitor')} Am PC</button>
            </div>`)
            .join('')}</div>`
        : '<p class="hint">Am PC gibt es noch keine Playlisten.</p>';
      for (const b of $$('[data-there]', plBox)) {
        b.addEventListener('click', () => {
          send(`/api/playlists/${b.dataset.there}/play`);
          toast('Playlist läuft am PC', 'ok');
        });
      }
      for (const b of $$('[data-here]', plBox)) {
        b.addEventListener('click', async () => {
          try {
            const data = await invoke('pc_get', { path: `/api/playlists/${b.dataset.here}` });
            data.tracks = data.tracks.filter(pcPlayable);
            if (!data.tracks.length) return toast('Die Playlist ist leer');
            await call('pc_play_here', { tracks: data.tracks, start: 0, album: data.playlist.name }).then(applyStatus);
            toast(tx('„{name}“ läuft auf dem Handy', { name: data.playlist.name }), 'ok');
          } catch (e) {
            toast(String(e), 'err');
          }
        });
      }
    } catch (e) {
      plBox.innerHTML = `<p class="hint">${esc(String(e))}</p>`;
    }
  };

  const paneFx = async () => {
    pane.innerHTML = `<p class="hint">${esc(tx('Klangeffekte am PC – wirken sofort auf die Musik dort.'))}</p><div class="pc-fx" id="pcFx"><p class="hint">${esc(tx('lädt …'))}</p></div>`;
    try {
      const [list, pk] = await Promise.all([
        invoke('pc_get', { path: '/api/effects' }),
        invoke('pc_get', { path: '/api/plugins' }).catch(() => ({ packages: [] })),
      ]);
      if (!fresh() || pc.tab !== 'fx') return;
      const box = $('#pcFx', pane);
      const card = (f) => {
        const o = pluginOrigin(f.source, pk.packages);
        const presets = [...(f.user_presets || []), ...f.presets];
        return `<div class="pc-fx-row${f.enabled ? '' : ' off'}" data-id="${esc(f.id)}">
          <label class="switch"><input type="checkbox" ${f.enabled ? 'checked' : ''}><span></span></label>
          <div class="pc-row-meta"><b>${esc(f.name)}</b><span data-raw>${esc(o ? o.label : tx('Mitgeliefert'))}</span></div>
          ${presets.length ? `<select class="select"><option value="">${esc(tx('Preset wählen …'))}</option>${presets.map((p) => `<option>${esc(p.name)}</option>`).join('')}</select>` : ''}
        </div>`;
      };
      box.innerHTML = list.map(card).join('');
      box.addEventListener('change', async (e) => {
        const row = e.target.closest('.pc-fx-row');
        if (!row) return;
        const id = encodeURIComponent(row.dataset.id);
        try {
          let info;
          if (e.target.matches('input[type=checkbox]')) info = await invoke('pc_post', { path: `/api/effects/${id}?enabled=${e.target.checked}` });
          else if (e.target.value) info = await invoke('pc_post', { path: `/api/effects/${id}?preset=${encodeURIComponent(e.target.value)}` });
          if (info) {
            row.classList.toggle('off', !info.enabled);
            $('input[type=checkbox]', row).checked = info.enabled;
          }
        } catch (err) {
          toast(String(err), 'err');
        }
      });
    } catch (e) {
      $('#pcFx', pane).innerHTML = `<p class="hint">${esc(String(e))}</p>`;
    }
  };

  const panePlugins = async () => {
    pane.innerHTML = `<p class="hint">${esc(tx('Plugin-Pakete vom PC aufs Handy holen – mit allen Teilen und der Lizenz.'))}</p><div id="pcPlugs"><p class="hint">${esc(tx('lädt …'))}</p></div>`;
    try {
      const [remote, local] = await Promise.all([invoke('pc_get', { path: '/api/plugins' }), invoke('plugin_packages')]);
      if (!fresh() || pc.tab !== 'plugins') return;
      const box = $('#pcPlugs', pane);
      const mine = Object.fromEntries((local.packages || []).map((p) => [p.id, p]));
      if (!remote.packages.length) {
        box.innerHTML = `<p class="hint">${esc(tx('Am PC sind keine Plugin-Pakete installiert.'))}</p>`;
        return;
      }
      box.innerHTML = `<div class="pkg-list">${remote.packages.map((p) => {
        const have = mine[p.id];
        const phone = (p.platforms || ['pc', 'phone']).includes('phone');
        const state = !phone ? tx('nur PC') : !have ? '' : have.version === p.version ? tx('auf dem Handy') : tx('Handy hat {v}', { v: have.version || '?' });
        const btn = !phone
          ? ''
          : `<button class="btn small${have && have.version === p.version ? '' : ' primary'}" data-get="${esc(p.id)}">${icon('phone')} ${esc(have ? (have.version === p.version ? tx('Neu holen') : tx('Aktualisieren')) : tx('Aufs Handy'))}</button>`;
        return `<div class="pkg${phone ? '' : ' off'}">
          <div class="pkg-meta" data-raw><b>${esc(p.name)}</b>${p.version ? ` <small>${esc(p.version)}</small>` : ''}${platformTags(p.platforms)}
            <span>${esc([packageParts(p), p.author, p.license, state].filter(Boolean).join(' · '))}</span></div>
          ${btn}
        </div>`;
      }).join('')}</div>`;
      for (const b of $$('[data-get]', box)) {
        b.addEventListener('click', async () => {
          const p = remote.packages.find((x) => x.id === b.dataset.get);
          if (p.sources) {
            const ok = await dialog({
              title: tx('„{name}“ möchte ins Internet', { name: p.name }),
              text: tx('Das Paket enthält eine Online-Quelle. Sie darf nur diese Adressen erreichen: {hosts}. Installiere es nur, wenn du der Quelle vertraust. Aktiv wird sie erst, wenn „Erweiterte Plugins erlauben“ eingeschaltet ist.', { hosts: (p.permissions?.network || []).join(', ') }),
              ok: tx('Installieren'),
            });
            if (!ok) return;
          }
          b.disabled = true;
          try {
            const m = await call('pc_install_plugin', { id: p.id });
            if (m.visualizers) await vizLoad();
            toast(m.effects || m.sources
              ? tx('„{name}“ installiert – wirkt nach dem nächsten Start der App', { name: m.name })
              : tx('„{name}“ installiert', { name: m.name }), 'ok');
            panePlugins();
          } catch {
            b.disabled = false;
          }
        });
      }
    } catch (e) {
      $('#pcPlugs', pane).innerHTML = `<p class="hint">${esc(String(e))}</p>`;
    }
  };

  const drawPane = () => ({ now: paneNow, music: paneMusic, fx: paneFx, plugins: panePlugins })[pc.tab]();
  drawPane();

  $('#pcUnpair', el).addEventListener('click', async () => {
    const ok = await dialog({ title: 'Verbindung trennen?', text: 'Titel vom PC lassen sich danach erst nach neuem Koppeln abspielen.', ok: 'Trennen', danger: true });
    if (!ok) return;
    await call('pc_unpair');
    render();
  });
}

// ---------- Wiki: Anleitungen auf GitHub ----------

const WIKI = 'https://github.com/Marcel1853/amber-player/wiki/';
/** Seiten im Wiki: [deutsch, englisch] – andere Sprachen bekommen die englische Seite. */
const WIKI_PAGES = {
  home: ['Home', 'English'],
  plugins: ['Plugin-Pakete', 'Plugin-Packages'],
  effects: ['Klang-Plugin-erstellen', 'Creating-a-Sound-Plugin'],
  visualizers: ['Visualisierung-erstellen', 'Creating-a-Visualizer'],
  api: ['Fernsteuerung-und-API', 'Remote-Control-and-API'],
  pc: ['Mit-dem-PC-verbinden', 'Connecting-to-your-PC'],
};

/** Link auf eine Wiki-Seite (öffnet im Browser). */
function wikiLink(key, label) {
  return `<a href="#" class="wiki-link" data-wiki="${key}">${esc(label)}</a>`;
}

/** Adresse von Amber Player auf GitHub im Browser öffnen – am Handy übers Android-Plugin. */
async function openGithub(url) {
  try {
    if (IS_ANDROID) await invoke('plugin:amber-mobile|open_url', { url });
    else await invoke('open_release_url', { url });
  } catch (e) {
    toast(String(e), 'err');
  }
}

document.addEventListener('click', (e) => {
  const a = e.target.closest('[data-wiki]');
  if (!a) return;
  e.preventDefault();
  const page = WIKI_PAGES[a.dataset.wiki] || WIKI_PAGES.home;
  openGithub(WIKI + encodeURIComponent(LANG === 'de' ? page[0] : page[1]));
});

// ---------- Updates: neue Version auf GitHub ----------

const updates = { info: null };

/** Automatische Update-Prüfung an/aus (Standard: an). */
function updateCheckEnabled() {
  try {
    return localStorage.getItem('updateCheck') !== 'off';
  } catch {
    return true;
  }
}

/**
 * Bei GitHub nach einer neuen Version fragen – automatisch höchstens einmal am Tag,
 * von Hand (manual) jederzeit.
 */
async function checkUpdates(manual = false) {
  if (!manual) {
    if (!updateCheckEnabled()) return;
    try {
      const last = Number(localStorage.getItem('updateLast') || 0);
      if (Date.now() - last < 20 * 3600 * 1000 && !localStorage.getItem('updateNewer')) return;
      localStorage.setItem('updateLast', String(Date.now()));
    } catch {}
  }
  let info;
  try {
    info = await invoke('check_update');
  } catch (e) {
    if (manual) toast(String(e), 'err');
    return;
  }
  updates.info = info;
  try {
    if (info.newer) localStorage.setItem('updateNewer', info.latest);
    else localStorage.removeItem('updateNewer');
  } catch {}
  $$('[data-view="settings"]').forEach((b) => b.classList.toggle('has-update', !!info.newer));
  if (info.newer) {
    toast(tx('Neue Version {v} verfügbar – unter Einstellungen herunterladen', { v: info.latest }), 'ok');
  } else if (manual) {
    toast(tx('Du hast die neueste Version ({v})', { v: info.current }), 'ok');
  }
  if (state.view.name === 'settings') render();
}

/** Update herunterladen: Handy → APK über den Browser, PC → Release-Seite mit den Paketen. */
async function downloadUpdate() {
  const info = updates.info;
  if (!info) return;
  await openGithub(IS_ANDROID ? info.apk || info.page : info.page);
  if (IS_ANDROID) toast(tx('Download startet im Browser – danach die APK antippen und installieren'), 'ok');
}

// ---------- Android: Benachrichtigung, Sperrbildschirm, Medientasten ----------

const androidMedia = {
  coverFor: null,
  cover: '',
  sent: '',

  /** Cover klein als Base64-JPEG für die Benachrichtigung. */
  async loadCover(t) {
    const key = t ? `${t.id}:${t.has_cover}` : '';
    if (key === this.coverFor) return this.cover;
    this.coverFor = key;
    this.cover = '';
    if (!t?.has_cover) return '';
    try {
      const img = new Image();
      img.crossOrigin = 'anonymous';
      img.src = coverUrl(t.id);
      await img.decode();
      const c = document.createElement('canvas');
      c.width = c.height = 256;
      c.getContext('2d').drawImage(img, 0, 0, 256, 256);
      this.cover = c.toDataURL('image/jpeg', 0.8).split(',')[1] || '';
    } catch {
      this.cover = '';
    }
    return this.cover;
  },

  async update(s, prev) {
    if (!IS_ANDROID) return;
    const t = s.track;
    const r = !t ? s.radio : null;
    if (!t && !r) {
      if (this.sent) invoke('plugin:amber-mobile|stop_media').catch(() => {});
      this.sent = '';
      return;
    }
    const playing = s.state === 'playing';
    // Nur bei Titel-/Zustandswechsel oder Sprung senden, nicht bei jedem Fortschritt.
    const jumped = prev && Math.abs((s.position_ms || 0) - (prev.position_ms || 0)) > 5000;
    const key = `${t?.id ?? r?.station.url}|${playing}|${t?.favorite}|${r?.title}`;
    if (key === this.sent && !jumped) return;
    this.sent = key;
    const cover = t ? await this.loadCover(t) : '';
    invoke('plugin:amber-mobile|update_media', {
      title: t ? t.title : r.title || r.station.name,
      artist: t ? t.artist || tx('Unbekannter Interpret') : r.station.name,
      album: t ? t.album || '' : tx('Internetradio'),
      playing,
      positionMs: Math.round(s.position_ms || 0),
      durationMs: Math.round(s.duration_ms || 0),
      cover,
    }).catch((e) => console.warn('Benachrichtigung:', e));
  },

  /** Beim Start: Rechte abfragen, Medientasten verbinden. */
  async init() {
    if (!IS_ANDROID) return;
    try {
      const perms = await invoke('plugin:amber-mobile|check_permissions');
      const missing = Object.entries(perms || {}).filter(([, v]) => v !== 'granted').map(([k]) => k);
      if (missing.length) {
        const res = await invoke('plugin:amber-mobile|request_permissions', { permissions: missing });
        if (res?.audio === 'granted' && perms.audio !== 'granted') {
          call('add_default_folder').catch(() => {});
        }
      }
    } catch (e) {
      console.warn('Berechtigungen:', e);
    }
    try {
      await window.__TAURI__.core.addPluginListener('amber-mobile', 'file', (e) => receivePackage(e));
      const shared = await invoke('plugin:amber-mobile|take_shared_file');
      if (shared?.file) receivePackage(shared.file);
    } catch (e) {
      console.warn('Geteilte Datei:', e);
    }
    try {
      await window.__TAURI__.core.addPluginListener('amber-mobile', 'pair', (e) => e.link && pcPair(e.link));
      const pending = await invoke('plugin:amber-mobile|take_pair_link');
      if (pending?.link) pcPair(pending.link);
    } catch (e) {
      console.warn('Koppeln:', e);
    }
    try {
      await window.__TAURI__.core.addPluginListener('amber-mobile', 'media', (e) => {
        const a = e.action;
        if (a === 'seek') send('seek', { ms: Math.max(0, Math.round(e.positionMs || 0)) });
        else if (['play', 'pause', 'next', 'prev'].includes(a)) send(a);
      });
    } catch (e) {
      console.warn('Medientasten:', e);
    }
    // Einmalig: Ausnahme von der Akkuoptimierung erbitten, sonst stoppt Samsung die Musik im Hintergrund.
    try {
      if (!localStorage.getItem('amber-battery-asked')) {
        localStorage.setItem('amber-battery-asked', '1');
        await invoke('plugin:amber-mobile|ignore_battery_optimizations');
      }
    } catch {}
  },
};

// ---------- Handy: Navigationsleiste selbst zusammenstellen ----------

/** Alle Bereiche, die in die Leiste unten dürfen: [Name, Symbol, verfügbar?] */
const NAV_ITEMS = {
  tracks: ['Titel', 'music'],
  albums: ['Alben', 'disc'],
  artists: ['Interpreten', 'user'],
  genres: ['Genres', 'tag'],
  playlists: ['Playlisten', 'playlist'],
  radio: ['Radio', 'radio'],
  favorites: ['Favoriten', 'heart'],
  recent: ['Verlauf', 'clock'],
  online: ['Online', 'search', () => online.sources.length > 0],
  pc: ['PC', 'monitor', () => IS_ANDROID],
  effects: ['Effekte', 'sliders'],
  settings: ['Einstellungen', 'settings'],
};
const NAV_DEFAULT = ['tracks', 'albums', 'playlists', 'radio'];
const NAV_SLOTS = 4;

const navAvailable = (id) => NAV_ITEMS[id] && (!NAV_ITEMS[id][2] || NAV_ITEMS[id][2]());

/** Die vier Bereiche in der Leiste (gespeichert pro Gerät). */
function navChoice() {
  let list = NAV_DEFAULT;
  try {
    const saved = JSON.parse(localStorage.getItem('nav') || 'null');
    if (Array.isArray(saved) && saved.length === NAV_SLOTS) list = saved;
  } catch {}
  return list.map((id, i) => (navAvailable(id) ? id : NAV_DEFAULT[i]));
}

function setNavChoice(list) {
  try {
    localStorage.setItem('nav', JSON.stringify(list));
  } catch {}
  renderBottomNav();
}

function renderBottomNav() {
  const nav = $('#bottomNav');
  nav.innerHTML = `${navChoice()
    .map((id) => `<button data-nav="${id}"><span data-icon="${NAV_ITEMS[id][1]}"></span><small>${esc(tx(NAV_ITEMS[id][0]))}</small></button>`)
    .join('')}<button id="moreBtn"><span data-icon="list"></span><small>${esc(tx('Mehr'))}</small></button>`;
  hydrateIcons(nav);
  updateNav();
}

/** „Mehr“: alles, was nicht in der Leiste steht. */
function showMoreMenu() {
  const r = $('#moreBtn').getBoundingClientRect();
  const inBar = navChoice();
  const rest = Object.keys(NAV_ITEMS).filter((id) => navAvailable(id) && !inBar.includes(id) && id !== 'effects' && id !== 'settings');
  const items = [
    // PC zuerst, sobald gekoppelt – dann findet man ihn schnell
    ...rest.sort((a, b) => (b === 'pc') - (a === 'pc')).map((id) => ({ label: NAV_ITEMS[id][0], icon: NAV_ITEMS[id][1], action: () => navigate({ name: id }) })),
    'sep',
    { label: 'Visualisierung', icon: 'activity', action: () => toggleViz(true) },
    ...['effects', 'settings'].filter((id) => !inBar.includes(id)).map((id) => ({ label: NAV_ITEMS[id][0], icon: NAV_ITEMS[id][1], action: () => navigate({ name: id }) })),
    { label: 'Leiste anpassen …', icon: 'edit', action: () => navigate({ name: 'settings', focus: 'nav' }) },
  ];
  showMenu(r.right - 230, r.top - 12 - items.length * 38, items);
}

// ---------- Zurück-Taste (Android) ----------

/**
 * Zurück schließt zuerst, was oben liegt (Dialog, Menü, Visualisierung, „Läuft gerade“), dann geht
 * es eine Seite zurück. Gibt `false` zurück, wenn es nichts mehr zurückzugehen gibt.
 */
function handleBack() {
  if ($('#modal').classList.contains('open')) {
    $('#modal [data-r="cancel"]')?.click();
    return true;
  }
  if (document.body.classList.contains('scanning')) {
    invoke('plugin:barcode-scanner|cancel').catch(() => {});
    return true;
  }
  if (openMenu) {
    closeMenu();
    return true;
  }
  if ($('#vizModes').classList.contains('open')) {
    vizMenu(false);
    return true;
  }
  if (viz.open) {
    // Vollbild über „Läuft gerade“: zurück zur Visualisierung hinter den Tasten
    if (viz.layer === 'full' && isMobile() && document.body.classList.contains('deck-open')) vizLayer('behind');
    else toggleViz(false);
    return true;
  }
  if (document.body.classList.contains('deck-open')) {
    closeDeck();
    return true;
  }
  if (state.back.length) {
    goBack();
    return true;
  }
  return false;
}

function bindBackButton() {
  const core = window.__TAURI__.core;
  if (!IS_ANDROID || !core?.addPluginListener) return;
  core
    .addPluginListener('app', 'back-button', () => {
      // Auf der Startseite: in den Hintergrund (Musik läuft weiter) statt die App zu beenden
      if (!handleBack()) invoke('plugin:amber-mobile|move_to_back').catch(() => {});
    })
    .catch(() => {});
}

/** „Läuft gerade“ schließen; eine Visualisierung dahinter geht mit aus. */
function closeDeck() {
  document.body.classList.remove('deck-open');
  if (viz.open && viz.layer === 'behind') toggleViz(false);
}

/** Handy: Mini-Player öffnet „Läuft gerade“, „Mehr“ öffnet die übrigen Bereiche. */
function bindMobile() {
  $('#deck').addEventListener('click', (e) => {
    if (!isMobile() || document.body.classList.contains('deck-open')) return;
    if (e.target.closest('button, input')) return;
    document.body.classList.add('deck-open');
    $('#deck').scrollTop = 0;
  });
  $('#deckClose').addEventListener('click', (e) => {
    e.stopPropagation();
    closeDeck();
  });
  $('#bottomNav').addEventListener('click', (e) => {
    const b = e.target.closest('button');
    if (!b) return;
    e.stopPropagation();
    if (b.id === 'moreBtn') showMoreMenu();
    else navigate({ name: b.dataset.nav });
  });
  renderBottomNav();
}

function bindViz() {
  $('#vizBtn').addEventListener('click', () => vizCycle());
  // Drehen: Zeichenfläche neu anordnen
  addEventListener('resize', () => {
    if (viz.open) vizPlaceCanvas();
  });
  // Hinter „Läuft gerade“: erstes Antippen holt die ausgeblendeten Bedienelemente zurück,
  // Wischen nach links/rechts wechselt die Visualisierung.
  const deck = $('#deck');
  let swallow = false;
  deck.addEventListener('pointerdown', (e) => {
    if (!document.body.classList.contains('viz-behind')) return;
    if (document.body.classList.contains('viz-dim')) {
      e.preventDefault();
      e.stopPropagation();
      swallow = true;
    }
    if (!swallow && (e.target === deck || e.target.closest('.disc-wrap'))) {
      // zweites Antippen der freien Fläche: gleich ausblenden
      clearTimeout(viz.dimTimer);
      document.body.classList.add('viz-dim');
      return;
    }
    vizDimSoon();
  }, true);
  deck.addEventListener('click', (e) => {
    if (!swallow) return;
    swallow = false;
    e.preventDefault();
    e.stopPropagation();
  }, true);
  let touch = null;
  deck.addEventListener('touchstart', (e) => {
    const t = e.touches[0];
    touch = document.body.classList.contains('viz-behind') && !e.target.closest('input, .progress, .deck-row') ? { x: t.clientX, y: t.clientY } : null;
  }, { passive: true });
  deck.addEventListener('touchend', (e) => {
    if (!touch) return;
    const t = e.changedTouches[0];
    const dx = t.clientX - touch.x;
    const dy = t.clientY - touch.y;
    touch = null;
    if (Math.abs(dx) > 60 && Math.abs(dy) < Math.abs(dx) * 0.6) vizStep(dx < 0 ? 1 : -1);
  }, { passive: true });
  $('#vizClose').addEventListener('click', () => toggleViz(false));
  $('#vizFull').addEventListener('click', toggleFullscreen);
  $('#vizPrev').addEventListener('click', () => send('prev'));
  $('#vizNext').addEventListener('click', () => send('next'));
  $('#vizPlay').addEventListener('click', () => send('toggle'));
  $('#vizModes').addEventListener('change', (e) => {
    const sel = e.target.closest('[data-auto]');
    if (!sel) return;
    const cfg = vizAutoCfg();
    cfg[sel.dataset.auto] = sel.dataset.auto === 'every' ? Number(sel.value) : sel.value;
    setVizAuto(cfg);
    $('[data-auto="every"]', $('#vizModes')).disabled = cfg.mode === 'off';
  });
  $('#vizModes').addEventListener('input', (e) => {
    if (!e.target.matches('.viz-search input')) return;
    viz.query = e.target.value;
    vizFilter();
  });
  $('#vizModes').addEventListener('keydown', (e) => {
    // Enter in der Suche: ersten Treffer nehmen
    if (e.key !== 'Enter' || !e.target.matches('.viz-search input')) return;
    const first = $$('#vizModes .viz-item').find((x) => !x.hidden && !x.closest('.viz-group').hidden);
    if (first) {
      vizUse(first.dataset.viz);
      vizMenu(false);
    }
  });
  $('#vizModes').addEventListener('click', (e) => {
    if (e.target.closest('.viz-pick')) return vizMenu();
    const star = e.target.closest('[data-star]');
    if (star) {
      e.stopPropagation();
      return vizToggleStar(star.dataset.star);
    }
    if (e.target.closest('.viz-search, .viz-auto')) return;
    const id = e.target.closest('[data-viz]')?.dataset.viz;
    if (id) {
      vizUse(id);
      vizMenu(false);
    }
  });
  // Klick daneben schließt das Menü
  document.addEventListener('pointerdown', (e) => {
    if (!e.target.closest('#vizModes')) vizMenu(false);
  }, true);
  // Maus: Bewegen blendet ein. Finger: Tippen auf die Fläche blendet ein – nochmal tippen sofort aus.
  $('#visual').addEventListener('pointermove', (e) => {
    if (e.pointerType === 'mouse') vizWake();
  });
  $('#visual').addEventListener('pointerup', (e) => {
    if (e.pointerType === 'mouse' || !e.target.closest('#vizCanvas, .viz-name')) return;
    if ($('#vizModes').classList.contains('open')) return vizMenu(false);
    if ($('#visual').classList.contains('idle')) vizWake();
    else {
      clearTimeout(viz.idleTimer);
      $('#visual').classList.add('idle');
    }
  });
  $('#visual').addEventListener('dblclick', (e) => {
    if (e.target.id === 'vizCanvas') toggleFullscreen();
  });
}

// ---------- Tastatur ----------

function bindKeys() {
  document.addEventListener('keydown', (e) => {
    const typing = e.target.matches('input:not([type=range]), textarea, select');
    if (e.key === 'Escape') {
      closeMenu();
      if ($('#vizModes').classList.contains('open')) vizMenu(false);
      else if (viz.open) toggleViz(false);
      else if (typing) e.target.blur();
      return;
    }
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'f') {
      e.preventDefault();
      $('#search').focus();
      $('#search').select();
      return;
    }
    if (typing) return;
    const ctrl = e.ctrlKey || e.metaKey;
    if (!ctrl && (e.key === 'v' || e.key === 'V')) {
      toggleViz();
      return;
    }
    if (viz.open && !ctrl) {
      vizWake();
      if (e.key === 'f' || e.key === 'F') return toggleFullscreen();
      if (/^[1-9]$/.test(e.key) && viz.list[Number(e.key) - 1]) return vizUse(viz.list[Number(e.key) - 1].id);
    }
    switch (e.key) {
      case ' ':
        e.preventDefault();
        send('toggle');
        break;
      case 'ArrowRight':
        e.preventDefault();
        ctrl ? send('next') : send('seek_by', { ms: 10000 });
        break;
      case 'ArrowLeft':
        e.preventDefault();
        ctrl ? send('prev') : send('seek_by', { ms: -10000 });
        break;
      case 'ArrowUp':
        if (ctrl) { e.preventDefault(); send('volume_up'); }
        break;
      case 'ArrowDown':
        if (ctrl) { e.preventDefault(); send('volume_down'); }
        break;
      case 'm': case 'M':
        if (!ctrl) send('toggle_mute');
        break;
      case 's': case 'S':
        if (!ctrl) send('set_shuffle', { on: null });
        break;
      case 'r': case 'R':
        if (!ctrl) send('set_repeat', { mode: null });
        break;
      case 'a': case 'A':
        if (ctrl && state.list) { e.preventDefault(); state.list.selectAll(); }
        break;
      case 'Delete':
        if (state.list?.opts.playlistId != null && state.list.selected.size) removeFromPlaylist(state.list);
        break;
      case 'Enter':
        if (state.list?.selected.size) {
          const i = Math.min(...state.list.selected);
          playList(state.list.tracks, i);
        }
        break;
    }
  });
  document.addEventListener('mouseup', (e) => {
    if (e.button === 3) goBack();
    if (e.button === 4) goForward();
  });
}

// ---------- Start ----------

async function init() {
  applyTheme();
  applyAccent();
  matchMedia('(prefers-color-scheme: dark)').addEventListener('change', applyTheme);
  guardRangesOnTouch();
  bindBackButton();
  document.addEventListener('click', (e) => {
    if (e.target.closest('[data-pkginstall]')) pickPackage();
  });
  $('#scanCancel').addEventListener('click', () => invoke('plugin:barcode-scanner|cancel').catch(() => {}));
  // Tablet quer (Ansicht mit Reitern oben): PC als eigener Reiter
  $('#pcTab').hidden = !IS_ANDROID;
  // Übersetzungen aus Plugin-Paketen früh laden (Effektnamen, Visualisierungen …)
  invoke('plugin_packages').then((r) => addPluginTexts(r.packages)).catch(() => {});
  hydrateIcons();

  for (const b of $$('[data-view]')) {
    b.addEventListener('click', () => navigate({ name: b.dataset.view }));
  }
  $('#back').addEventListener('click', goBack);
  $('#forward').addEventListener('click', goForward);

  const search = $('#search');
  const doSearch = debounce(() => {
    const q = search.value.trim();
    if (q) {
      if (state.view.name === 'search') {
        state.view = { name: 'search', q };
        render();
      } else {
        navigate({ name: 'search', q });
      }
    } else if (state.view.name === 'search') {
      goBack();
    }
  }, 220);
  search.addEventListener('input', doSearch);

  bindDropTray();
  bindViz();
  bindMobile();
  androidMedia.init();
  setTimeout(() => checkUpdates(), 5000);
  await loadSources();
  bindBar();
  bindKeys();

  await listen('status', (e) => applyStatus(e.payload));
  await listen('library-changed', () => {
    loadPlaylists();
    if (state.view.name !== 'effects') render();
  });
  await listen('scan-progress', (e) => {
    const p = e.payload;
    const box = $('#scanInfo');
    if (p.running) {
      box.classList.add('on');
      $('.txt', box).textContent = p.total ? tx('Bibliothek wird eingelesen … {done} / {total}', { done: fmtNum(p.done), total: fmtNum(p.total) }) : tx('Suche nach Musik …');
      $('.bar div', box).style.width = p.total ? `${(p.done / p.total) * 100}%` : '5%';
    } else {
      box.classList.remove('on');
      if (p.error) toast(p.error, 'err');
      else if (p.result?.added_or_updated || p.result?.removed) {
        toast(tx('Bibliothek aktualisiert: {n}', { n: plural(p.result.total, 'Titel', 'Titel') }), 'ok');
      }
    }
  });

  await listen('cover-progress', (e) => {
    const p = e.payload;
    const box = $('#scanInfo');
    if (p.running) {
      box.classList.add('on');
      $('.txt', box).textContent = tx('Cover werden gesucht … {done} / {total}', { done: fmtNum(p.done), total: fmtNum(p.total) });
      $('.bar div', box).style.width = `${(p.done / Math.max(1, p.total)) * 100}%`;
    } else {
      box.classList.remove('on');
      if (p.error) toast(p.error, 'err');
      else if (p.found) toast(tx('{n} aus dem Internet geladen', { n: plural(p.found, 'Cover', 'Cover') }), 'ok');
    }
  });

  applyStatus(await invoke('get_status'));
  await loadPlaylists();
  render();
  requestAnimationFrame(tickProgress);
}

init();
