#!/usr/bin/env bash
# NexusTerminal launcher — uses release binary if built, dev mode otherwise
export PATH="/usr/local/bin:/usr/bin:/bin:/home/loufogle/.cargo/bin:$PATH"
export HOME="/home/loufogle"

RELEASE="/home/loufogle/nexus-os/packages/nexus-terminal/src-tauri/target/release/nexus-terminal"
DIR="/home/loufogle/nexus-os/packages/nexus-terminal"

if [ -f "$RELEASE" ]; then
  # cd to package root so the binary finds .env, icons, and other assets
  cd "$DIR" && exec "$RELEASE"
else
  cd "$DIR" && exec npm run tauri dev
fi
