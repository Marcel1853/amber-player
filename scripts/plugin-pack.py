#!/usr/bin/env python3
"""Baut aus einem Plugin-Ordner ein installierbares Plugin-Paket (.amberplugin) für Amber Player.

Aufruf:   python3 scripts/plugin-pack.py <Plugin-Ordner> [Zielordner]
Beispiel: python3 scripts/plugin-pack.py examples/plugins/nachtfahrt
          → nachtfahrt.amberplugin im aktuellen Ordner

Ein .amberplugin ist eine ZIP-Datei. Geprüft wird vorher, was Amber Player beim Installieren
auch prüft: plugin.json mit gültiger id und name, alle Teile aus „contents“ (bzw. „type“/„main“)
sind vorhanden und haben die richtige Endung, eine Lizenzdatei liegt bei und es sind nur
erlaubte Dateien im Paket (plugin.json, die Teile, LICENSE*, README*).
"""
import json
import pathlib
import re
import sys
import zipfile

DEFAULT_MAIN = {"effect": "effect.json", "visualizer": "visualizer.js"}
EXT = {"effect": ".json", "visualizer": ".js"}
SAFE = re.compile(r"[A-Za-z0-9_.-]{1,64}")


def fail(msg):
    print(f"Fehler: {msg}", file=sys.stderr)
    sys.exit(1)


def safe_path(path):
    parts = path.split("/")
    return 1 <= len(parts) <= 2 and all(SAFE.fullmatch(p) and not p.startswith(".") for p in parts)


def main():
    if len(sys.argv) < 2:
        fail("Aufruf: python3 scripts/plugin-pack.py <Plugin-Ordner> [Zielordner]")
    src = pathlib.Path(sys.argv[1])
    out_dir = pathlib.Path(sys.argv[2]) if len(sys.argv) > 2 else pathlib.Path(".")
    manifest_path = src / "plugin.json"
    if not manifest_path.is_file():
        fail(f"{manifest_path} fehlt")
    try:
        m = json.loads(manifest_path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as e:
        fail(f"plugin.json ist kein gültiges JSON: {e}")

    pid = m.get("id", "")
    if not re.fullmatch(r"[A-Za-z0-9_-]{1,32}", pid):
        fail("„id“ in plugin.json: 1–32 Zeichen aus Buchstaben, Ziffern, - und _")
    if not str(m.get("name", "")).strip():
        fail("„name“ in plugin.json fehlt")

    contents = m.get("contents") or []
    if not contents:
        kind = m.get("type")
        if kind not in DEFAULT_MAIN:
            fail("plugin.json braucht „contents“ (Liste der Teile) oder „type“ (effect/visualizer)")
        contents = [{"type": kind, "file": m.get("main") or DEFAULT_MAIN[kind]}]
    if len(contents) > 32:
        fail("Höchstens 32 Teile pro Paket")

    parts = []
    for c in contents:
        kind, file = c.get("type"), c.get("file", "")
        if kind not in EXT:
            fail(f"Unbekannte Art „{kind}“ in „contents“ (erlaubt: effect, visualizer)")
        if not safe_path(file):
            fail(f"Ungültiger Dateipfad in „contents“: {file}")
        if not file.lower().endswith(EXT[kind]):
            fail(f"{file} muss bei dieser Art auf {EXT[kind]} enden")
        path = src / file
        if not path.is_file():
            fail(f"Teil {file} fehlt")
        if path.stat().st_size > 512 * 1024:
            fail(f"{file} ist größer als 512 KB")
        if kind == "effect":
            try:
                json.loads(path.read_text(encoding="utf-8"))
            except json.JSONDecodeError as e:
                fail(f"{file} ist kein gültiges JSON: {e}")
        parts.append(file)

    top = [p for p in src.iterdir() if p.is_file() and not p.name.startswith(".")]
    licenses = [p.name for p in top if p.name.upper().startswith(("LICENSE", "LICENCE", "COPYING"))]
    readmes = [p.name for p in top if p.name.upper().startswith("README")]
    if not licenses:
        fail("Es fehlt eine Lizenzdatei (LICENSE) – jedes Paket braucht seine eigene Lizenz")
    files = ["plugin.json", *parts, *licenses, *readmes]
    total = sum((src / f).stat().st_size for f in files)
    if total > 2 * 1024 * 1024:
        fail("Das Paket wäre größer als 2 MB")

    # Warnen, wenn im Ordner Dateien liegen, die nicht mitkommen (z. B. vergessen in „contents“).
    known = set(files)
    for p in src.rglob("*"):
        rel = p.relative_to(src).as_posix()
        if p.is_file() and not any(part.startswith(".") for part in p.relative_to(src).parts) and rel not in known:
            print(f"Hinweis: {rel} kommt nicht ins Paket (nicht in „contents“)", file=sys.stderr)

    out_dir.mkdir(parents=True, exist_ok=True)
    target = out_dir / f"{pid}.amberplugin"
    with zipfile.ZipFile(target, "w", zipfile.ZIP_DEFLATED) as z:
        for f in files:
            z.write(src / f, f)
    counts = {k: sum(1 for c in contents if c["type"] == k) for k in EXT}
    print(
        f"{target}  ({m.get('name')} {m.get('version', '')}: {counts['effect']} Klangeffekt(e), "
        f"{counts['visualizer']} Visualisierung(en), Lizenz: {m.get('license') or 'siehe LICENSE'})"
    )


if __name__ == "__main__":
    main()
