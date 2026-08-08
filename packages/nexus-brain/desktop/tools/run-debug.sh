#!/usr/bin/env bash
# Launch the debug build with everything useful turned on.
#
#   run-debug.sh              open the window
#   run-debug.sh --headless   run the pipeline with no window
#
# Output is teed to ~/.local/state/librarian/run.log so a crash is still
# readable after the terminal is closed.

set -Eeuo pipefail

HERE="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
SRC="$HERE/../src-tauri"
LOG_DIR="$HOME/.local/state/librarian"
LOG="$LOG_DIR/run.log"
BIN="$SRC/target/debug/librarian"

mkdir -p "$LOG_DIR"

# Backtraces on panic, and the WebKit inspector reachable by right-clicking the
# window (debug builds only).
export RUST_BACKTRACE=full
export RUST_LOG="${RUST_LOG:-librarian=debug,tauri=info}"
export WEBKIT_DISABLE_COMPOSITING_MODE=1

if [[ ! -x "$BIN" ]]; then
    echo "debug binary missing, building..." | tee -a "$LOG"
    ( cd "$SRC" && cargo build ) 2>&1 | tee -a "$LOG"
fi

# The window build needs the frontend bundle present; the headless path does not.
if [[ ! -f "$HERE/../frontend/dist/index.html" ]]; then
    echo "frontend bundle missing, building..." | tee -a "$LOG"
    ( cd "$HERE/../frontend" && npm install --no-audit --no-fund && npm run build ) \
        2>&1 | tee -a "$LOG"
fi

{
    printf '\n===== %s : librarian %s =====\n' "$(date -Is)" "${*:-window}"
    printf 'binary  : %s\n' "$BIN"
    printf 'log     : %s\n\n' "$LOG"
} | tee -a "$LOG"

# stdbuf keeps the pipeline's line-buffered progress flowing into the terminal
# instead of sitting in a 4 KB pipe buffer.
exec stdbuf -oL -eL "$BIN" "$@" 2>&1 | tee -a "$LOG"
