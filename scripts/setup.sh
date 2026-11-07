#!/usr/bin/env bash
#
# One-command spin-up for your own space.
#
#   ./scripts/setup.sh
#
# It gets the app running with the public-domain jyotisha-corpus (github.com/maximally0/jyotisha-corpus,
# CC BY 4.0) imported into your knowledge folder, so the AI writer and Ask box have the classical
# texts to draw on. Minimal inputs: run it, open the printed URL.
#
# It picks the simplest path available:
#   * Docker present        -> `docker compose up -d --build` (the image bakes the corpus in at build:
#                              from ./corpus if you have already put it there, else downloaded).
#   * Otherwise, Cargo      -> build the release binary, import the corpus into your knowledge folder,
#                              and run the server against it.
#
# Options (all optional):
#   --local            Force the Cargo path even if Docker is available.
#   --no-texts         Skip the corpus download (smaller, works offline; no classical texts).
#   --listen ADDR:PORT Address to serve on (default 127.0.0.1:8080 for --local; compose uses 8080).
#   --knowledge DIR    Knowledge folder for the --local path (default ~/.vedic_astrology/knowledge).
#
# The optional AI writer stays off unless you set VA_AI_BASE_URL and VA_AI_API_KEY (put them in a
# .env file beside docker-compose.yml, or export them before running). Nothing here needs them.
#
set -euo pipefail
cd "$(dirname "$0")/.."

FORCE_LOCAL=0
WITH_TEXTS=1
LISTEN="127.0.0.1:8080"
KNOWLEDGE="${VA_KNOWLEDGE:-$HOME/.vedic_astrology/knowledge}"

while [ $# -gt 0 ]; do
  case "$1" in
    --local)      FORCE_LOCAL=1; shift ;;
    --no-texts)   WITH_TEXTS=0; shift ;;
    --listen)     LISTEN="${2:?--listen needs ADDR:PORT}"; shift 2 ;;
    --knowledge)  KNOWLEDGE="${2:?--knowledge needs a folder}"; shift 2 ;;
    -h|--help)    sed -n '2,24p' "$0"; exit 0 ;;
    *) echo "unknown option: $1 (try --help)" >&2; exit 2 ;;
  esac
done

bold() { printf '\033[1m%s\033[0m\n' "$1"; }

# ---- Docker path: everything (including the corpus) is built into the image ----
if [ "$FORCE_LOCAL" -eq 0 ] && command -v docker >/dev/null 2>&1 && docker compose version >/dev/null 2>&1; then
  bold "==> Docker found; building and starting with docker compose"
  if [ "$WITH_TEXTS" -eq 0 ]; then
    docker compose build --build-arg WITH_TEXTS=0
    docker compose up -d
  else
    docker compose up -d --build
  fi
  echo
  bold "Vedic Astrology is up:  http://localhost:8080"
  echo "   logs:  docker compose logs -f     stop:  docker compose down"
  exit 0
fi

# ---- Local path: build, import the corpus, run ----
if ! command -v cargo >/dev/null 2>&1; then
  echo "Neither Docker (with compose) nor Cargo is available." >&2
  echo "Install Docker Desktop, or install Rust from https://rustup.rs, then re-run." >&2
  exit 1
fi

bold "==> Building the release binary (first build takes a few minutes)"
cargo build --release --bin vedic_astrology_web

BIN=target/release/vedic_astrology_web

if [ "$WITH_TEXTS" -eq 1 ]; then
  bold "==> Importing the public-domain jyotisha-corpus into $KNOWLEDGE"
  echo "   (CC BY 4.0, attributed to the maximally0/jyotisha-corpus contributors; needs network access)"
  "$BIN" --import-texts "$KNOWLEDGE" || echo "   text download failed; the app still works without it"
else
  bold "==> Skipping the corpus download (--no-texts)"
fi

echo
bold "==> Starting the server on http://$LISTEN   (Ctrl-C to stop)"
exec "$BIN" --listen "$LISTEN" --knowledge "$KNOWLEDGE"
