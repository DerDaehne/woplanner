#!/usr/bin/env bash
# Prüft, ob jede im Markup verwendete CSS-Klasse auch definiert ist.
# Ausgabe leer = alles definiert. Nicht-leer = die genannten Klassen sind wirkungslos.
#
# Hintergrund: Bei der Tailwind→Bulma-Migration wurden mehrfach Klassennamen
# erfunden, die in keinem Stylesheet existieren (is-align-center, is-flex-between,
# is-expanded auf freien Divs, ...). Eine Textsuche nach "tailwind" findet das nicht.
set -euo pipefail
cd "$(dirname "$0")/.."

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

# Klassen aus dem Markup
grep -rhoE 'class="[^"]*"' templates/ \
  | sed 's/class="//; s/"$//' | tr ' ' '\n' \
  | grep -E '^[a-zA-Z]' | sort -u > "$tmp/used"

# Selektoren aus beiden Stylesheets. Der Punkt darf NICHT in die Zeichenklasse,
# sonst wird ".column.is-half" als ein einziger Name gelesen.
cat static/css/bulma.min.css static/css/style.css \
  | grep -oE '\.[a-zA-Z][a-zA-Z0-9_-]*' | sed 's/^\.//' | sort -u > "$tmp/defined"

# Escaped Namen in style.css (.wo-w-1\.5rem) separat nachtragen
grep -oE '\.wo-[a-zA-Z0-9_\\.-]+' static/css/style.css \
  | sed 's/^\.//; s/\\//g' | sort -u >> "$tmp/defined"
sort -u -o "$tmp/defined" "$tmp/defined"

missing=$(comm -23 "$tmp/used" "$tmp/defined")
if [ -n "$missing" ]; then
  echo "Undefinierte Klassen im Markup:"
  echo "$missing"
  exit 1
fi
echo "OK: alle im Markup verwendeten Klassen sind definiert."
