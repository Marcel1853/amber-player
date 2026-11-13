'use strict';
// Läuft als Web Worker: lädt Visualisierungs-Plugins und zeichnet auf eine OffscreenCanvas.
// Plugins sehen nur das Objekt `amber` und ihre Leinwand – kein Netz, keine Dateien, kein Player.

const send = self.postMessage.bind(self);

// Netz und Speicher für Plugins abschalten, bevor fremder Code läuft.
for (const name of ['fetch', 'XMLHttpRequest', 'WebSocket', 'EventSource', 'importScripts', 'indexedDB', 'caches', 'Worker', 'SharedWorker', 'BroadcastChannel', 'WebTransport']) {
  try {
    Object.defineProperty(self, name, { value: undefined, writable: false, configurable: false });
  } catch (e) {
    /* nicht vorhanden */
  }
}

const registry = new Map();
let loading = '';
let canvas = null;
let ctx = null;
let current = null;
let state = {};
let size = { width: 1, height: 1, dpr: 1 };
let info = { title: '', artist: '', album: '', cover: null, accent: [242, 160, 61], accent2: [232, 102, 74] };
const failed = new Set();

const clamp = (v, a, b) => Math.max(a, Math.min(b, v));

/** Schnittstelle für Plugins. */
const amber = Object.freeze({
  /** Visualisierung anmelden: { id, name, author, description, setup?, resize?, draw } */
  visualizer(def) {
    if (!def || typeof def.id !== 'string' || !def.id || typeof def.draw !== 'function') {
      throw new Error('amber.visualizer() braucht mindestens „id“ und „draw“');
    }
    registry.set(def.id, {
      def,
      meta: {
        id: def.id,
        name: String(def.name || def.id).slice(0, 40),
        author: String(def.author || '').slice(0, 60),
        description: String(def.description || '').slice(0, 200),
        source: loading,
      },
    });
  },
  /** Farbe als CSS-Text: amber.rgba([r, g, b], alpha) */
  rgba: (c, a = 1) => `rgba(${c[0] | 0}, ${c[1] | 0}, ${c[2] | 0}, ${a})`,
  /** Zwischen zwei Farben mischen (t = 0 … 1). */
  mix: (c1, c2, t, a = 1) => amber.rgba([0, 1, 2].map((i) => c1[i] + (c2[i] - c1[i]) * clamp(t, 0, 1)), a),
  clamp,
  lerp: (a, b, t) => a + (b - a) * t,
  /** Mittelwert eines Bereichs der Bänder (0 … 1 = tief … hoch). */
  range: (bands, from, to) => {
    const a = Math.floor(from * bands.length);
    const b = Math.max(a + 1, Math.floor(to * bands.length));
    let s = 0;
    for (let i = a; i < b; i++) s += bands[i] || 0;
    return s / (b - a);
  },
});

function scene(extra = {}) {
  return { ctx, width: size.width, height: size.height, dpr: size.dpr, info, state, amber, ...extra };
}

function fail(id, err) {
  if (failed.has(id)) return;
  failed.add(id);
  send({ type: 'error', id, message: String((err && err.message) || err) });
}

function call(fn, extra) {
  if (!current || typeof current.def[fn] !== 'function') return;
  ctx.save();
  ctx.setTransform(size.dpr, 0, 0, size.dpr, 0, 0);
  try {
    current.def[fn](scene(extra));
  } catch (e) {
    fail(current.meta.id, e);
  } finally {
    ctx.restore();
  }
}

function load(plugins) {
  registry.clear();
  failed.clear();
  const errors = [];
  for (const p of plugins) {
    loading = p.source;
    try {
      new Function('amber', `"use strict";\n${p.code}`)(amber);
    } catch (e) {
      errors.push({ source: p.source, message: String((e && e.message) || e) });
    }
  }
  loading = '';
  send({ type: 'list', list: [...registry.values()].map((r) => r.meta), errors });
}

function use(id) {
  current = registry.get(id) || registry.values().next().value || null;
  state = {};
  if (!ctx) return;
  ctx.setTransform(1, 0, 0, 1, 0, 0);
  ctx.clearRect(0, 0, canvas.width, canvas.height);
  call('setup');
  send({ type: 'using', id: current ? current.meta.id : null });
}

self.onmessage = (e) => {
  const m = e.data;
  switch (m.type) {
    case 'init':
      canvas = m.canvas;
      ctx = canvas.getContext('2d');
      break;
    case 'load':
      load(m.plugins);
      break;
    case 'use':
      use(m.id);
      break;
    case 'resize':
      size = { width: m.width, height: m.height, dpr: m.dpr };
      if (canvas) {
        canvas.width = Math.max(1, Math.round(m.width * m.dpr));
        canvas.height = Math.max(1, Math.round(m.height * m.dpr));
      }
      call('resize');
      break;
    case 'info':
      if (info.cover && info.cover !== m.info.cover && info.cover.close) info.cover.close();
      info = m.info;
      break;
    case 'frame': {
      const f = m.frame;
      const b = f.bands;
      f.bass = amber.range(b, 0, 0.15);
      f.mid = amber.range(b, 0.15, 0.6);
      f.treble = amber.range(b, 0.6, 1);
      call('draw', { frame: f });
      break;
    }
  }
};
