'use strict';
// Übersetzungen für Amber Player.
//
// Grundsprache ist Deutsch: Der deutsche Text ist zugleich der Schlüssel. Die Sprachdateien
// unter lang/ (erzeugt aus crates/app/i18n/source-*.py mit build.py) tragen in I18N[<code>] ein, was „Deutsch → Sprache“ heißt. Fehlt eine Übersetzung,
// gilt Englisch, zuletzt der deutsche Text.
//
// Feste Texte in der Oberfläche (Knöpfe, Überschriften, Hinweise, title/placeholder) übersetzt
// ein MutationObserver automatisch, sobald sie im Dokument erscheinen. Texte mit eingesetzten
// Werten laufen über tx('… {name} …', { name }) im Code.

const I18N = {};

/** Verfügbare Sprachen (Code → Name in der eigenen Sprache). */
const LANGUAGES = [
  ['de', 'Deutsch'],
  ['en', 'English'],
  ['es', 'Español'],
  ['fr', 'Français'],
  ['it', 'Italiano'],
  ['nl', 'Nederlands'],
  ['pl', 'Polski'],
  ['tr', 'Türkçe'],
];

/** Gewählte Sprache: Einstellung oder – bei „system“ – die Sprache des Geräts. */
const LANG = (() => {
  let pick = 'system';
  try {
    pick = localStorage.getItem('lang') || 'system';
  } catch {}
  const codes = LANGUAGES.map(([c]) => c);
  if (codes.includes(pick)) return pick;
  for (const l of navigator.languages || [navigator.language || 'de']) {
    const c = String(l).slice(0, 2).toLowerCase();
    if (codes.includes(c)) return c;
  }
  return 'en';
})();

/** Für Zahlen und Datumsangaben. */
const LOCALE = { de: 'de-DE', en: 'en-GB', es: 'es-ES', fr: 'fr-FR', it: 'it-IT', nl: 'nl-NL', pl: 'pl-PL', tr: 'tr-TR' }[LANG] || 'de-DE';

document.documentElement.lang = LANG;

function lookup(key) {
  if (LANG === 'de') return undefined;
  return I18N[LANG]?.[key] ?? I18N.en?.[key];
}

/** Übersetzt einen Text; {name} wird durch vars.name ersetzt. */
function tx(key, vars) {
  let s = lookup(key) ?? key;
  // Schlüssel mit Zählform-Präfix („1|Titel“, „n|Titel“) zeigen auf Deutsch nur das Wort.
  if (s === key && /^[1n]\|/.test(key)) s = key.slice(2);
  if (vars) s = s.replace(/\{(\w+)\}/g, (m, k) => (k in vars ? String(vars[k]) : m));
  return s;
}

/** Zahl in der Schreibweise der Sprache. */
function fmtNum(n) {
  return Number(n).toLocaleString(LOCALE);
}

// ---------- Automatische Übersetzung fester Texte ----------

// Hier stehen Inhalte aus der Bibliothek (Titel, Interpreten, Sender, Playlisten) – nie übersetzen.
const RAW = '.row, .q-item, .card, .c-title, .meta, .pl-item .name, .st-card .name, .st-card .info, .st-row, .pc-row, .pc-meta, .q-row, .upnext .q-list, .chip, .card .name, .card .sub, .folder .path, code, .copy, [data-raw]';
const ATTRS = ['title', 'placeholder', 'aria-label'];

function translateText(node) {
  const raw = node.nodeValue;
  const key = raw.trim();
  if (!key || key.length > 400) return;
  const tr = lookup(key);
  if (tr === undefined || tr === key) return;
  node.nodeValue = raw.replace(key, tr);
}

function translateEl(el) {
  if (el.closest?.(RAW)) return;
  for (const a of ATTRS) {
    const v = el.getAttribute?.(a);
    if (v) {
      const tr = lookup(v);
      if (tr !== undefined && tr !== v) el.setAttribute(a, tr);
    }
  }
}

function translateTree(root) {
  if (LANG === 'de' || !root) return;
  if (root.nodeType === Node.TEXT_NODE) {
    if (!root.parentElement?.closest(RAW)) translateText(root);
    return;
  }
  if (root.nodeType !== Node.ELEMENT_NODE) return;
  if (root.closest(RAW)) return;
  translateEl(root);
  const walker = document.createTreeWalker(root, NodeFilter.SHOW_ELEMENT | NodeFilter.SHOW_TEXT, {
    acceptNode: (n) => (n.nodeType === Node.ELEMENT_NODE && n.matches(RAW) ? NodeFilter.FILTER_REJECT : NodeFilter.FILTER_ACCEPT),
  });
  for (let n = walker.nextNode(); n; n = walker.nextNode()) {
    if (n.nodeType === Node.TEXT_NODE) translateText(n);
    else translateEl(n);
  }
}

if (LANG !== 'de') {
  const start = () => {
    translateTree(document.body);
    new MutationObserver((list) => {
      for (const m of list) {
        if (m.type === 'childList') m.addedNodes.forEach(translateTree);
        else if (m.type === 'characterData') translateTree(m.target);
        else if (m.type === 'attributes') translateEl(m.target);
      }
    }).observe(document.body, { childList: true, subtree: true, characterData: true, attributes: true, attributeFilter: ATTRS });
  };
  // Erst wenn alle Skripte (auch die Sprachdateien unter lang/) geladen sind.
  if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', start);
  else start();
}
