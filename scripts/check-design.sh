#!/usr/bin/env bash
# Prüft die Design-Gesetze aus der Notiz adr-002-design-rack.
#
# Zwei Stufen:
#   1. statisch  — jede im Markup benutzte Klasse ist definiert (check-classes.sh)
#   2. gemessen  — die laufende App im Headless-Chromium bei 390x844
#
# Warum gemessen und nicht gelesen: CSS scheitert lautlos. Eine Regel mit
# undefiniertem Custom Property wird verworfen, ein erfundener Klassenname
# erzeugt keinen Fehler, ein negativer Außenrand erzeugt keine Warnung. In zwei
# aufeinanderfolgenden Reviews wurden deshalb Kriterien falsch abgehakt.
#
# Aufruf: ./scripts/check-design.sh
# Exit 0 = alle scharf geschalteten Gesetze eingehalten.
set -euo pipefail
cd "$(dirname "$0")/.."

# Welche Gesetze geprüft werden. Die Tickets des Epics #703 schalten ihres
# frei, sobald sie umgesetzt sind: L1,L2,L3,L4,L6.
ACTIVE_LAWS="${ACTIVE_LAWS:-L1,L2,L3,L6}"

# --- Stufe 1: Klassennamen ------------------------------------------------
./scripts/check-classes.sh

# --- Voraussetzungen für Stufe 2 -----------------------------------------
if [ ! -e node_modules/playwright-core ]; then
  echo "playwright-core fehlt — Stufe 2 übersprungen (npm i -D playwright-core)."
  exit 0
fi

CHROMIUM_BIN="${CHROMIUM_BIN:-}"
if [ -z "$CHROMIUM_BIN" ]; then
  for c in /nix/store/*-playwright-chromium/chrome-linux64/chrome \
           "$HOME"/.cache/ms-playwright/chromium-*/chrome-linux64/chrome \
           "$(command -v chromium || true)" \
           "$(command -v google-chrome || true)"; do
    [ -x "$c" ] && CHROMIUM_BIN="$c" && break
  done
fi
if [ -z "$CHROMIUM_BIN" ]; then
  echo "Keine Chromium-Binary gefunden — Stufe 2 übersprungen (CHROMIUM_BIN setzen)."
  exit 0
fi
export CHROMIUM_BIN

# --- App starten ----------------------------------------------------------
PORT="${PORT:-3111}"
DB="${DATABASE_URL:-sqlite:./bodybuilding.db}"
export BASE_URL="http://localhost:$PORT"

cargo build --quiet
PORT="$PORT" DATABASE_URL="$DB" SEED_DATABASE=true ./target/debug/woplanner > /tmp/woplanner-check.log 2>&1 &
APP_PID=$!
trap 'kill $APP_PID 2>/dev/null || true' EXIT

for _ in $(seq 1 60); do
  curl -sf -o /dev/null "$BASE_URL/health" && break
  sleep 0.5
done

# --- Routen aus der Seed-Datenbank auflösen -------------------------------
db_file="${DB#sqlite:}"
q() { sqlite3 "$db_file" "$1" 2>/dev/null | head -1; }

user_id=$(q "select id from users limit 1")
workout_id=$(q "select id from workouts limit 1")
exercise_id=$(q "select id from exercises limit 1")
history_id=$(q "select id from completed_workouts limit 1")
active_id=$(q "select id from active_workouts limit 1")

routes="{\"users\":\"/users\",\"dashboard\":\"/dashboard\",\"workouts\":\"/workouts\""
[ -n "$workout_id" ]  && routes="$routes,\"workout-detail\":\"/workouts/$workout_id\""
routes="$routes,\"exercises\":\"/exercises\""
[ -n "$exercise_id" ] && routes="$routes,\"progression\":\"/exercises/$exercise_id/progression\""
routes="$routes,\"history\":\"/history\""
[ -n "$history_id" ]  && routes="$routes,\"history-detail\":\"/history/$history_id\""
[ -n "$active_id" ]   && routes="$routes,\"live-training\":\"/live-training/$active_id\""
routes="$routes}"

export ROUTES="$routes"
export SEED_USER_ID="${user_id:-user-demo-001}"
export ACTIVE_LAWS
export NODE_PATH="$PWD/node_modules"

echo "Geprüfte Gesetze: $ACTIVE_LAWS"
node scripts/design-audit.js
