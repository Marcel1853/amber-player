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
# Unterordner (z. B. en/ für Englisch) mit eigener Seitenleiste werden mit übertragen.
cp -r docs/wiki/. "$work/wiki/"
python3 - "$work/wiki" <<'EOF'
import os, re, sys
d = sys.argv[1]
files = [os.path.join(r, f) for r, _, fs in os.walk(d) if ".git" not in r for f in fs if f.endswith(".md")]
pages = {os.path.basename(f)[:-3] for f in files}
# Im Wiki zählt nur der Seitenname, nicht der Ordner: „../Home.md“ oder „en/FAQ.md“ → „Home“ bzw. „FAQ“.
link = re.compile(r"\]\((?:\.\./|[\w-]+/)*([^)#\s:/]+)\.md(#[^)]*)?\)")
for p in files:
    s = open(p, encoding="utf-8").read()
    s = link.sub(lambda m: f"]({m.group(1)}{m.group(2) or ''})" if m.group(1) in pages else m.group(0), s)
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
# git@github.com:Nutzer/Repo.git bzw. https://github.com/Nutzer/Repo.git → https://github.com/Nutzer/Repo/wiki
web=$(printf '%s' "${repo_url%.git}" | sed -e 's#^git@github.com:#https://github.com/#')
echo "== Wiki aktualisiert: $web/wiki"
