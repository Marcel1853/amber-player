#!/usr/bin/env bash
# Überträgt die Wiki-Seiten aus docs/wiki/ ins GitHub-Wiki des Repositorys.
#
# Voraussetzungen:
#  - Das Wiki ist eingeschaltet (Repository öffentlich oder bezahltes GitHub-Konto) und hat
#    mindestens eine Seite (die erste Seite einmal im Browser anlegen).
#  - Zugriff auf das Repository per git (z. B. über `gh auth login`).
#
# Im Repository verweisen die Seiten mit „Seite.md“ aufeinander (damit die Links auch auf GitHub
# im Ordner docs/wiki funktionieren); im Wiki heißen die Seiten ohne Endung. Das Skript passt die
# Links beim Übertragen an.
set -euo pipefail

cd "$(dirname "$0")/.."
repo_url=$(git remote get-url origin)
wiki_url="${repo_url%.git}.wiki.git"
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

echo "== Wiki holen: $wiki_url"
if ! git clone --quiet "$wiki_url" "$work/wiki"; then
  echo "Das Wiki ist nicht erreichbar. Ist es eingeschaltet und gibt es schon eine erste Seite?" >&2
  exit 1
fi

# Alte Seiten entfernen (außer .git), neue hineinkopieren und Links anpassen.
find "$work/wiki" -mindepth 1 -maxdepth 1 ! -name .git -exec rm -rf {} +
cp docs/wiki/*.md "$work/wiki/"
python3 - "$work/wiki" <<'EOF'
import os, re, sys
d = sys.argv[1]
pages = {f[:-3] for f in os.listdir(d) if f.endswith(".md")}
for f in os.listdir(d):
    if not f.endswith(".md"):
        continue
    p = os.path.join(d, f)
    s = open(p, encoding="utf-8").read()
    s = re.sub(r"\]\(([^)#\s:/]+)\.md(#[^)]*)?\)",
               lambda m: f"]({m.group(1)}{m.group(2) or ''})" if m.group(1) in pages else m.group(0), s)
    open(p, "w", encoding="utf-8").write(s)
EOF

cd "$work/wiki"
git add -A
if git diff --cached --quiet; then
  echo "== Keine Änderungen – das Wiki ist aktuell."
  exit 0
fi
git commit --quiet -m "Wiki aus docs/wiki aktualisiert"
git push --quiet
echo "== Wiki aktualisiert: ${repo_url%.git}/wiki"
