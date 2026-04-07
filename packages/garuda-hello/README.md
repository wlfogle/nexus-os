# Garuda Hello

A Garuda Linux implementation inspired by Windows Hello: fast, convenient, and secure local authentication using biometrics (face, fingerprint) and PIN fallback. The goal is to integrate with the Linux PAM stack and common display managers on Garuda (Arch-based), providing a smooth unlock and login experience.

## Vision and scope
- Face unlock via webcam with on-device templates
- Optional fingerprint unlock leveraging fprintd/libfprint
- PIN fallback and rate-limits
- PAM integration for login, sudo, lock, and screensaver
- Display manager integration (e.g., SDDM/KDE)
- Privacy first: no cloud, user-controlled enrollment and deletion

This repository starts with a Rust-based workspace:
- common: shared types and utilities
- daemon: privileged service mediating access to biometric devices and policy
- pam-helper: minimal helper binary for PAM to request verification from daemon
- cli: user-facing tool for enrollment, listing, deletion, and settings

## Status
Early scaffold. No biometric functionality yet. Initial objectives:
1) Define IPC and data models
2) Implement daemon and CLI skeleton
3) Provide PAM helper interface hooks
4) Add simple enrollment and verification stubs

## Security considerations
- Store templates encrypted at rest (per-user)
- Never exfiltrate biometric data
- Support configurable anti-spoofing and liveness checks
- Enforce backoff/lockout on repeated failures

## Development
- Language: Rust (edition 2021)
- Targets: Linux (Garuda)
- Build: Cargo workspace with multiple crates

## License
TBD
