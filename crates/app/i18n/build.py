#!/usr/bin/env python3
"""Erzeugt aus den Übersetzungstabellen (source-*.py) die Sprachdateien ui/lang/<code>.js.

Aufruf:  python3 crates/app/i18n/build.py
Prüft dabei, dass in jeder Übersetzung dieselben Platzhalter ({name}, {n} …) und
HTML-Tags (<b>, </b>) stehen wie im deutschen Text.
"""
import json
import pathlib
import re
import runpy
import sys

HERE = pathlib.Path(__file__).parent
OUT = HERE.parent / "ui" / "lang"
CODES = ["en", "es", "fr", "it", "nl", "pl", "tr"]

rows = []
for src in sorted(HERE.glob("source-*.py")):
    rows += runpy.run_path(str(src))["ROWS"]

problems = []
seen = {}
for row in rows:
    if len(row) != len(CODES) + 1:
        problems.append(f"Falsche Spaltenzahl: {row[0]!r}")
        continue
    key = row[0]
    if key in seen:
        problems.append(f"Doppelt: {key!r}")
    seen[key] = row[1:]
    marks = sorted(re.findall(r"\{\w+\}|</?b>", key))
    for code, text in zip(CODES, row[1:]):
        if sorted(re.findall(r"\{\w+\}|</?b>", text)) != marks:
            problems.append(f"{code}: Platzhalter/Tags passen nicht bei {key!r}")

for i, code in enumerate(CODES):
    data = {key: vals[i] for key, vals in seen.items()}
    body = json.dumps(data, ensure_ascii=False, indent=1)
    (OUT / f"{code}.js").write_text(
        f"// Erzeugt von crates/app/i18n/build.py aus source-*.py – nicht von Hand bearbeiten.\nI18N.{code} = {body};\n",
        encoding="utf-8",
    )

print(f"{len(seen)} Texte in {len(CODES)} Sprachen geschrieben")
if problems:
    print("\n".join(problems))
    sys.exit(1)
