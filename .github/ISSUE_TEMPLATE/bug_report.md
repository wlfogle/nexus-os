---
name: Bug Report
about: Report a kernel bug, crash, or unexpected behavior
title: '[BUG] '
labels: ['bug']
assignees: ''
---

## Description
What happened? Be specific.

## Build target
- [ ] laptop (x86_64)
- [ ] tiamat (x86_64 server)
- [ ] bahamut (AArch64)

## How to reproduce
1. Build: `make <target> && make iso-<target> && make run-<target>`
2. ...
3. Observe: ...

## Expected behavior
What should have happened instead.

## Serial / framebuffer output
```
Paste the relevant serial console output here.
Include the full boot log if it's a boot failure.
```

## Environment
- Host OS:
- QEMU version (if applicable):
- Rust toolchain: `rustc --version`
- Commit hash: `git rev-parse --short HEAD`

## Additional context
Screenshots, dmesg, or anything else that helps.
