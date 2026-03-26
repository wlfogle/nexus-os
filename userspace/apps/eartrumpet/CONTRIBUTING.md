# Contributing

## Prereqs
- Rust stable toolchain
- GTK4 dev packages on your distro

## Checks
- cargo fmt --all -- --check
- cargo clippy --all-targets -- -D warnings

## Run
```
cargo run
```

## Notes
- Pactl is used for PipeWire compatibility via pipewire-pulse.
- Avoid blocking the main thread; UI updates through glib timers and callbacks.