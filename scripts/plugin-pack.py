#!/usr/bin/env python3
"""Baut aus einem Plugin-Ordner ein installierbares Plugin-Paket (ZIP) für Amber Player.

Aufruf:   python3 scripts/plugin-pack.py <Plugin-Ordner> [Zielordner]
Beispiel: python3 scripts/plugin-pack.py examples/plugins/telefon
          → telefon.zip im aktuellen Ordner

Geprüft wird vorher, was Amber Player beim Installieren auch prüft: plugin.json mit gültiger
id, name und type, die Plugin-Datei (main) ist vorhanden, es liegt eine Lizenzdatei bei und es
sind nur erlaubte Dateien im Ordner (plugin.json, Plugin-Datei, LICENSE*, README*).
"""
import json
import pathlib
import re
import sys
import zipfile

DEFAULT_MAIN = {"effect": "effect.json", "visualizer": "visualizer.js"}
EXT = {"effect": ".json", "visualizer": ".js"}


def fail(msg):
    print(f"Fehler: {msg}", file=sys.stderr)
    sys.exit(1)


def main():
    if len(sys.argv) < 2:
        fail(__doc__.strip().splitlines()[2])
    src = pathlib.Path(sys.argv[1])
    out_dir = pathlib.Path(sys.argv[2]) if len(sys.argv) > 2 else pathlib.Path(".")
    manifest_path = src / "plugin.json"
    if not manifest_path.is_file():
        fail(f"{manifest_path} fehlt")
    try:
        m = json.loads(manifest_path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as e:
        fail(f"plugin.json ist kein gültiges JSON: {e}")

    pid, kind = m.get("id", ""), m.get("type", "")
    if not re.fullmatch(r"[A-Za-z0-9_-]{1,32}", pid):
        fail("„id“ in plugin.json: 1–32 Zeichen aus Buchstaben, Ziffern, - und _")
    if not str(m.get("name", "")).strip():
        fail("„name“ in plugin.json fehlt")
    if kind not in DEFAULT_MAIN:
        fail("„type“ in plugin.json muss „effect“ oder „visualizer“ sein")
    main_file = m.get("main") or DEFAULT_MAIN[kind]
    if not main_file.lower().endswith(EXT[kind]):
        fail(f"„main“ muss bei dieser Plugin-Art auf {EXT[kind]} enden")
    if not (src / main_file).is_file():
        fail(f"Plugin-Datei {main_file} fehlt")
    if kind == "effect":
        try:
            json.loads((src / main_file).read_text(encoding="utf-8"))
        except json.JSONDecodeError as e:
            fail(f"{main_file} ist kein gültiges JSON: {e}")

    def allowed(name):
        up = name.upper()
        return name in ("plugin.json", main_file) or up.startswith(("LICENSE", "LICENCE", "COPYING", "README"))

    files = sorted(p for p in src.iterdir() if p.is_file() and not p.name.startswith("."))
    extra = [p.name for p in files if not allowed(p.name)]
    if extra:
        fail(f"Nicht erlaubte Dateien im Ordner: {', '.join(extra)}")
    if not any(p.name.upper().startswith(("LICENSE", "LICENCE", "COPYING")) for p in files):
        fail("Es fehlt eine Lizenzdatei (LICENSE) – jedes Plugin braucht seine eigene Lizenz")
    total = sum(p.stat().st_size for p in files)
    if total > 1024 * 1024:
        fail("Das Paket wäre größer als 1 MB")

    out_dir.mkdir(parents=True, exist_ok=True)
    target = out_dir / f"{pid}.zip"
    with zipfile.ZipFile(target, "w", zipfile.ZIP_DEFLATED) as z:
        for p in files:
            z.write(p, p.name)
    print(f"{target}  ({m.get('name')} {m.get('version', '')}, Lizenz: {m.get('license') or 'siehe LICENSE'})")


if __name__ == "__main__":
    main()
