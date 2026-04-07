#!/bin/bash

# Move legacy scripts
mkdir -p legacy
mv 230-plex.sh 231-jellyfin-proxmox.sh auto-fix-alexa.sh final-fix.sh legacy/ 2>/dev/null

# Move legacy code and experimental folders
mv docs-analyzer/ multi-processor/ open-interpreter-tauri/ Coding/ legacy/ 2>/dev/null

# Move all homeassistant-configs* if present
mv homeassistant-configs* legacy/ 2>/dev/null

# Move old docs and experiments into docs/_archive
mkdir -p docs/_archive
mv docs/old* docs/experiments* docs/_archive/ 2>/dev/null

# Place README stubs for new archive locations
cat > legacy/README.md <<'EOF'
# Legacy Scripts & Configs

This folder contains legacy scripts, configs, or experiments that are no longer maintained or part of the current stack.

**For the supported stack and automation, see the root README and `/ansible`.**
EOF

cat > docs/_archive/README.md <<'EOF'
# Archived Documentation

This folder contains deprecated or superseded documentation.

**For the current stack and plans, see `/docs/_organized/summary.md`.**
EOF

echo "Legacy files and folders moved. README stubs created in /legacy and /docs/_archive."