## What this PR does
Brief description. Reference the issue if there is one: `Fixes #N`

## Phase / subsystem
Which phase and kernel module(s) does this touch?

## Changes
- List the key changes (new files, modified modules, new syscalls, etc.)

## Build verification
- [ ] `make laptop` compiles clean (no warnings)
- [ ] `make iso-laptop && make run-laptop` boots and runs correctly
- [ ] Tested on additional targets: [ ] tiamat [ ] bahamut

## Serial output
```
Paste relevant serial console output showing your changes work.
```

## Checklist
- [ ] No `TODO`, `FIXME`, `unimplemented!()`, or dead code
- [ ] Every `unsafe` block has a `// SAFETY:` comment
- [ ] Commit messages follow `type(scope): description` format
- [ ] Documentation updated (if applicable)
