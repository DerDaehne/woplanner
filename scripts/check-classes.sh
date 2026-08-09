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

status=0

missing=$(comm -23 "$tmp/used" "$tmp/defined")
if [ -n "$missing" ]; then
  echo "Undefinierte Klassen im Markup:"
  echo "$missing"
  status=1
fi

# Gegenrichtung: eine .wo-*-Klasse, die niemand benutzt, ist toter Code.
# Ausnahme: .wo-field-error gehört zu #680 (sichtbare Feldfehler) und wartet
# dort auf ihren ersten Nutzer.
grep -oE '^\.wo-[a-zA-Z0-9_\\.-]+' static/css/style.css \
  | sed 's/^\.//; s/\\//g' | sort -u > "$tmp/own"
grep '^wo-' "$tmp/used" | sort -u > "$tmp/own_used"
unused=$(comm -23 "$tmp/own" "$tmp/own_used" | grep -v '^wo-field-error$' || true)
if [ -n "$unused" ]; then
  echo "Definierte, aber nie benutzte .wo-*-Klassen:"
  echo "$unused"
  status=1
fi

[ "$status" -eq 0 ] && echo "OK: Klassen im Markup und in style.css decken sich."
exit "$status"
