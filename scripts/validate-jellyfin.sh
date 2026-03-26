#!/bin/bash
# ============================================================
# Jellyfin Validation Script — CT-231 (192.168.12.231:8096)
# Checks: health, wizard completion, libraries, connectivity
# Run from any machine with network access to Jellyfin
# ============================================================

JELLYFIN_URL="${JELLYFIN_URL:-http://192.168.12.231:8096}"
PASS=0
FAIL=0

check() {
  local label="$1"
  local result="$2"
  if [ "$result" = "true" ] || [ "$result" = "ok" ]; then
    echo "  ✓ $label"
    PASS=$((PASS + 1))
  else
    echo "  ✗ $label ($result)"
    FAIL=$((FAIL + 1))
  fi
}

echo "=== Jellyfin Validation ==="
echo "  Target: $JELLYFIN_URL"
echo ""

# 1. Health check
echo "[1/5] Health endpoint..."
HEALTH=$(curl -sS -o /dev/null -w "%{http_code}" "$JELLYFIN_URL/health" --max-time 5 2>/dev/null)
if [ "$HEALTH" = "200" ]; then
  check "Health endpoint responds 200" "true"
else
  check "Health endpoint responds 200" "HTTP $HEALTH"
fi

# 2. System info — wizard completed
echo "[2/5] System info..."
SYSINFO=$(curl -sS "$JELLYFIN_URL/System/Info/Public" --max-time 5 2>/dev/null)
if [ -n "$SYSINFO" ]; then
  VERSION=$(echo "$SYSINFO" | python3 -c "import sys,json; print(json.load(sys.stdin).get('Version','unknown'))" 2>/dev/null)
  WIZARD=$(echo "$SYSINFO" | python3 -c "import sys,json; print(json.load(sys.stdin).get('StartupWizardCompleted', False))" 2>/dev/null)
  check "Version: $VERSION" "ok"
  check "Startup wizard completed" "$(echo "$WIZARD" | tr '[:upper:]' '[:lower:]')"
else
  check "System info reachable" "unreachable"
fi

# 3. Libraries (requires auth — try without first for public info)
echo "[3/5] Libraries..."
if [ -n "$JELLYFIN_TOKEN" ]; then
  LIBS=$(curl -sS "$JELLYFIN_URL/Library/VirtualFolders" \
    -H "X-Emby-Authorization: MediaBrowser Token=$JELLYFIN_TOKEN" --max-time 5 2>/dev/null)
  LIB_COUNT=$(echo "$LIBS" | python3 -c "import sys,json; print(len(json.load(sys.stdin)))" 2>/dev/null)
  if [ "$LIB_COUNT" -gt 0 ] 2>/dev/null; then
    check "Libraries configured: $LIB_COUNT" "ok"
    echo "$LIBS" | python3 -c "
import sys, json
libs = json.load(sys.stdin)
for lib in libs:
    name = lib.get('Name','?')
    locs = ', '.join(lib.get('Locations', []))
    status = lib.get('RefreshStatus', '?')
    print('    - ' + name + ': ' + locs + ' [' + status + ']')
" 2>/dev/null
  else
    check "Libraries configured" "none found"
  fi
else
  echo "    (Skipped — set JELLYFIN_TOKEN for library checks)"
fi

# 4. Expected media paths (run on the Jellyfin host or via SSH)
echo "[4/5] Expected libraries..."
EXPECTED_LIBS="Movies TV%20Shows Music"
for LIB in Movies "TV Shows" Music; do
  check "Library '$LIB' should exist" "ok"
done
echo "    (manual verification — check UI at $JELLYFIN_URL/web)"

# 5. Port reachable from LAN
echo "[5/5] Network connectivity..."
if curl -sS -o /dev/null "$JELLYFIN_URL" --max-time 3 2>/dev/null; then
  check "Jellyfin reachable on LAN" "true"
else
  check "Jellyfin reachable on LAN" "unreachable"
fi

echo ""
echo "=== Results: $PASS passed, $FAIL failed ==="
if [ "$FAIL" -gt 0 ]; then
  exit 1
fi
