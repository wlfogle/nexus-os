# Backend Debug Verification

## Check Tauri Backend Logs:
```bash
# Look for backend terminal manager logs
sudo chroot /mnt bash -c "cd /home/lou/github/nexus-terminal && export HOME=/home/lou && export DISPLAY=:0 && export XDG_RUNTIME_DIR=/run/user/1000 && export OLLAMA_MODELS=/mnt/media/workspace/models && npm run tauri dev 2>&1 | grep -E '(write_to_terminal|terminal_manager)'"
```

## Terminal State Verification:
1. Check if terminal ID exists
2. Verify terminal process is running
3. Ensure terminal manager can receive data

## Common Backend Issues:
- Terminal not properly initialized
- Permission issues with terminal processes  
- Tauri IPC communication failures
- Terminal manager state corruption

## Manual Terminal Test:
```bash
# Test if backend can create and write to terminals
# (This would be done through the app's interface)
```

## Debug Steps:
1. Check active tab has valid terminalId
2. Verify terminal manager is initialized
3. Test write_to_terminal Tauri command directly
4. Monitor backend process logs during command execution
