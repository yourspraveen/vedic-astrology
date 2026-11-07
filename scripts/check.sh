#!/usr/bin/env bash
#
# Quality gate. Run before opening a pull request; CI runs the same checks.
#
#   ./scripts/check.sh
#
# It enforces: a clean build, no clippy warnings (this also catches Rust naming lints such as
# non_snake_case), passing tests, and structural caps on file size so modules stay reviewable.
#
set -euo pipefail
cd "$(dirname "$0")/.."

# ---- size caps (keep modules reviewable; split before they become god-files) ----
WARN_LINES=1000      # a nudge to consider splitting
FAIL_LINES=1600      # hard cap

bold() { printf '\033[1m%s\033[0m\n' "$1"; }

bold "==> clippy (deny all warnings, incl. naming lints)"
cargo clippy --lib --bin vedic_astrology_web --all-targets -- -D warnings

bold "==> tests"
cargo test --lib

bold "==> structure: file-size caps (warn >${WARN_LINES}, fail >${FAIL_LINES} lines)"
status=0
while IFS= read -r f; do
  n=$(wc -l < "$f" | tr -d ' ')
  if   [ "$n" -gt "$FAIL_LINES" ]; then printf '   FAIL  %-28s %s lines (> %s)\n' "$f" "$n" "$FAIL_LINES"; status=1
  elif [ "$n" -gt "$WARN_LINES" ]; then printf '   warn  %-28s %s lines (> %s, consider splitting)\n' "$f" "$n" "$WARN_LINES"
  fi
done < <(find src -name '*.rs' | sort)
if [ "$status" -ne 0 ]; then
  bold "structure gate failed: split the file(s) above into smaller modules."
  exit 1
fi

# Formatting is intentionally NOT enforced: the codebase uses a deliberately dense style
# (long embedded HTML/CSS strings, packed match arms). Match the surrounding style in new code.

bold "==> all checks passed"
