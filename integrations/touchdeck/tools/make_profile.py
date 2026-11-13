#!/usr/bin/env python3
"""Erzeugt die fertigen TouchDeck-Seiten für das Plugin „Amber Player“ auf Deutsch und Englisch:
„Amber Player (Deutsch).tdprofile“ und „Amber Player (English).tdprofile“ (gleicher Aufbau, nur die Texte unterscheiden sich).

Aufruf:  python3 tools/make_profile.py
Import:  TouchDeck › Seitenliste › „⋯“ › Profil importieren (oder „Fertige Seiten“ in der Plugin-Karte).
Format:  touchdeck-profile, Version 1 (wie die Generatoren der anderen Plugins in touchdeck-plugins).

Aufbau (4 × 5):
  Läuft gerade (3 breit)           | Fortschritt (2 breit, Balken)
  ⏮ Zurück | ⏯ Play/Pause | ⏭ Weiter | 🔀 Zufall | 🔁 Wiederholen
  🔉 Leiser | 🔇 Stumm | 🔊 Lauter | Lautstärke (Balken) | ❤️ Favorit
  ⏪ −10 s | ⏩ +10 s | 🎛️ Equalizer | ⭐ Favoriten zufällig | 🌙 Sleep-Timer
"""
import json
import os
import zipfile

PLUGIN = "plugin:amber-player"
ROWS, COLS = 4, 5
PAGE = "p_amber"
PROFILES = {"de": "Amber Player (Deutsch).tdprofile", "en": "Amber Player (English).tdprofile"}

COLOR = "#23212e"
COLOR_LIVE = "#1c1a26"
COLOR_ON = "#6b4fd8"
COLOR_PINK = "#c23d84"
COLOR_PLAY = "#3a2f6b"

_ids = {"n": 0}
_lang = {"de": True}


def L(de, en):
    """Sichtbarer Text in der Sprache des gerade erzeugten Profils."""
    return de if _lang["de"] else en


def action(type_, **params):
    return {"id": "", "type": type_, "enabled": True, "params": params}


def kw(aid, **params):
    """Aktion des Plugins: plugin:amber-player:<id>."""
    return action(f"{PLUGIN}:{aid}", **params)


def live(lid, bar=False):
    return {"source": f"{PLUGIN}:{lid}", "command": "", "variable": "", "bar": bar}


def button(row, col, title, actions=None, *, icon="", icon_on=None, color=COLOR, color_on=None, span=1,
           live_src=None, hold=None, font=13, pos="bottom"):
    _ids["n"] += 1
    b = {
        "id": f"b_amber_{_ids['n']}",
        "row": row, "col": col, "rowSpan": 1, "colSpan": span,
        "title": title,
        "icon": {"type": "emoji", "value": icon},
        "color": color,
        "titlePos": pos,
        "fontSize": font,
        "mode": "press" if (actions or hold) else "display",
        "actions": actions or [],
        "holdActions": hold or [],
        "releaseActions": [],
        "doubleActions": [],
        "holdRepeat": 0,
        "events": [],
    }
    if color_on:
        b["colorOn"] = color_on
    if icon_on:
        b["iconOn"] = {"type": "emoji", "value": icon_on}
    if live_src:
        b["live"] = live_src
    return b


def main_page():
    buttons = [
        # Zeile 0: Anzeige
        button(0, 0, "", [kw("toggle")], icon="🎵", span=3, live_src=live("now"), color=COLOR_LIVE,
               color_on=COLOR_PLAY, font=15),
        button(0, 3, "", [], icon="⏱️", span=2, live_src=live("progress", bar=True), color=COLOR_LIVE,
               color_on=COLOR_PLAY, font=12),
        # Zeile 1: Wiedergabe
        button(1, 0, L("Zurück", "Previous"), [kw("prev")], icon="⏮️"),
        button(1, 1, L("Play / Pause", "Play / pause"), [kw("toggle")], icon="▶️", icon_on="⏸️",
               live_src=live("state"), color_on=COLOR_ON),
        button(1, 2, L("Weiter", "Next"), [kw("next")], icon="⏭️"),
        button(1, 3, L("Zufall", "Shuffle"), [kw("shuffle", mode="toggle")], icon="🔀", live_src=live("shuffle"),
               color_on=COLOR_ON),
        button(1, 4, L("Wiederholen", "Repeat"), [kw("repeat", mode="cycle")], icon="🔁", live_src=live("repeat"),
               color_on=COLOR_ON),
        # Zeile 2: Lautstärke
        button(2, 0, L("Leiser", "Quieter"), [kw("volume", mode="down", value=50)], icon="🔉"),
        button(2, 1, L("Stumm", "Mute"), [kw("mute")], icon="🔇"),
        button(2, 2, L("Lauter", "Louder"), [kw("volume", mode="up", value=50)], icon="🔊"),
        button(2, 3, "", [], icon="🎚️", live_src=live("volume", bar=True), color=COLOR_LIVE, color_on=COLOR_PLAY),
        button(2, 4, L("Favorit", "Favourite"), [kw("favorite")], icon="🤍", icon_on="❤️",
               live_src=live("favorite"), color_on=COLOR_PINK),
        # Zeile 3: Extras
        button(3, 0, L("−10 s", "−10 s"), [kw("seek", mode="back", seconds=0)], icon="⏪"),
        button(3, 1, L("+10 s", "+10 s"), [kw("seek", mode="forward", seconds=0)], icon="⏩"),
        # Tippen = EQ an/aus, halten = Preset „Bass“
        button(3, 2, "Equalizer", [kw("effect", effect="equalizer", mode="toggle", preset="")], icon="🎛️",
               live_src=live("eq"), color_on=COLOR_ON,
               hold=[kw("effect", effect="equalizer", mode="preset", preset="Bass")]),
        button(3, 3, L("Favoriten mixen", "Favourites mix"),
               [kw("music", artist="", album="", genre="", search="", favorites=True, shuffle=True)], icon="⭐"),
        # Tippen = Sleep-Timer 30 Minuten, halten = aus
        button(3, 4, L("Sleep 30", "Sleep 30"), [kw("sleep", minutes=30)], icon="🌙", live_src=live("sleep"),
               color_on=COLOR_ON, hold=[kw("sleep", minutes=0)]),
    ]
    return {"id": PAGE, "name": "Amber Player", "rows": ROWS, "cols": COLS, "layout": "grid", "folder": "Amber Player",
            "background": "#0f0e15", "ratioW": 16, "ratioH": 10, "colSizes": [], "rowSizes": [],
            "buttons": buttons, "events": []}


def build(lang):
    _lang["de"] = lang == "de"
    _ids["n"] = 0
    return {
        "format": "touchdeck-profile", "formatVersion": 1, "app": "0.12.0",
        "name": "Amber Player", "pages": [main_page()], "flows": [], "events": [],
        "values": {}, "images": {},
    }


def write(path, profile):
    with zipfile.ZipFile(path, "w", zipfile.ZIP_DEFLATED) as z:
        z.writestr("profile.json", json.dumps(profile, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    for lang, file in PROFILES.items():
        path = os.path.join(root, file)
        write(path, build(lang))
        print(f"{path} geschrieben")
