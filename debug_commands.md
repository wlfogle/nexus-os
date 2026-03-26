# Command Routing Debug Tests

## Shell Commands (should go to terminal):
- `ls -la` ✅ File operation
- `pwd` ✅ Directory query
- `git status` ✅ Development tool
- `npm run build` ✅ Package management
- `ps aux` ✅ System info
- `cat file.txt` ✅ Text processing
- `./script.sh` ✅ Script execution
- `cd ..` ✅ Navigation

## AI Queries (should go to AI):
- `what is git?` ✅ Question word trigger
- `how do I install npm?` ✅ Question word trigger  
- `explain this error` ✅ Conversational phrase
- `help me with this command` ✅ Conversational phrase
- `why is this failing?` ✅ Question mark
- `can you show me the syntax?` ✅ Conversational phrase

## Edge Cases to Test:
- `ls` (single word - should go to shell)
- `help` (ambiguous - currently goes to AI via conversational detection)
- `history` (could be shell or AI context)
- Empty input (should be ignored)
- Commands with pipes: `ls | grep txt`
- Commands with redirection: `echo "test" > file.txt`

## Debugging Command Execution Flow:

1. **Input Detection**: Check console for routing logs
2. **Terminal ID**: Verify `activeTab.terminalId` exists
3. **Tauri Communication**: Check if `write_to_terminal` invoke succeeds
4. **Backend Processing**: Verify terminal backend receives the command
5. **Output Display**: Check if command output appears in terminal

## Common Issues:

1. **Missing Terminal ID**: If terminal not properly initialized
2. **Tauri Communication Failure**: Backend not responding
3. **Regex Edge Cases**: Commands not matching patterns correctly
4. **Async Race Conditions**: Commands processed out of order
