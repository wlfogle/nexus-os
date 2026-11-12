# MISTAKES.md — nexus-os
Review relevant entries before planning or editing code.
Add an entry after a confirmed mistake or user correction.
Keep entries short, specific, and project-related.
Merge repeated mistakes instead of creating duplicates.
Do not record transient tool failures or unverified guesses.

## Entry format
### [YYYY-MM-DD] [Short title]
**Mistake:** [What the agent did wrong]
**Root cause:** [Why the decision failed]
**Prevention:** [The rule to apply next time]
**Verification:** [How to confirm the mistake was avoided]

---

### [2026-11-12] Designing a replacement before studying existing packages
**Mistake:** Proposed a new reconciliation app before finding the existing Librarian, then treated stale scripts as archive-only and briefly introduced a date cutoff.
**Root cause:** Inventoried filenames before reading the umbrella source of truth and package catalog; conflated outdated implementation with an obsolete idea.
**Prevention:** Study home material alongside current repositories and their history first. Extend existing tools, recover useful behavior separately from reference preservation, and never classify by age alone.
**Verification:** Librarian comparisons cite current repository files, keep ambiguous findings unresolved, and require human approval for all moves. Historical originals remain reference material.

### [2026-11-12] Structured Ollama responses consumed by thinking
**Mistake:** A real Librarian idea assessment received no final JSON with the existing 700-token request budget.
**Root cause:** Installed `qwen3.5:9b` advertises thinking enabled by default; reasoning and the final response are separate fields.
**Prevention:** Inspect `/api/show` thinking controls. Explicitly request `think:false` for bounded structured classification and allow enough final-response tokens; do not parse or expose reasoning as the answer.
**Verification:** Request serialization includes `think:false`; exercise the real local assessment path, not only JSON parser fixtures.

### [2026-11-08] Treating completed Jellyfin scans as successful discovery
**Mistake:** Tried targeted updates and a separate WWE library before extracting the exact scanner exception; neither bypassed the database failure.
**Root cause:** Movies cleanup attempted to tombstone stale Tickle Me (1965) watch-history rows onto conflicting UserData keys. SQLite Error 19 aborted discovery even while the scheduled scan reported completion. The two WWE MP4s were readable and had valid H.264/AAC streams.
**Prevention:** Inspect the library-operation exception first. Never clear UserData wholesale. Stop only Jellyfin, make a private SQLite backup, and scope any repair to confirmed missing items with conflicting keys. Preserve all history fields; archival keys retain the old records but are not automatic watch-state relinking.
**Verification:** CT-300 backup and original-key manifest: `/var/backups/jellyfin-wwe-20261108T195653Z/`. Four conflicting stale rows received unique `:archived:<ItemId>` keys; all 2,716 UserData rows remained immediately after repair and SQLite quick checks passed. Restarted only Jellyfin and rescanned. Both titles now belong to Movies; the empty workaround library was removed without deleting media. Crown Jewel item `c9f18ff96b85c2dd9f5dc036a5b08b86` is 10,056.01 seconds; Countdown item `c0d45ca6d30db381d781130d53c6e3d0` is 5,841.800391 seconds. Both report direct-play/direct-stream support and served authenticated HTTP 206 video/mp4 ranges; health returned 200. No device playback was started. Restore only with Jellyfin stopped and reconcile subsequent viewing activity before using the backup.

### [2026-10-17] Mistaking Snap path visibility for policy access
**Mistake:** Treated a successful `test -r` and a host-visible policy file as evidence that Brave Snap could consume managed settings.
**Root cause:** DAC visibility did not establish AppArmor read permission; actual directory enumeration and file reads were denied, and Brave's policy log reported no file found.
**Prevention:** Verify directory listing, file reads and the browser's loaded-policy page. If confinement blocks supported host policies, use the official extension store/settings UI instead of weakening AppArmor or editing extension databases.
**Verification:** Confirm extension activation and the saved self-hosted URL after reloading its settings; do not claim Snap policy enforcement where none exists.
### [2026-10-17] Treating a copied volume as a completed cutover
**Mistake:** The client-attached CT-300 root move ended with broken pipe, while a later completion report conflicted with the live HDD root.
**Root cause:** A destination volume's existence and an ended process do not establish successful Proxmox configuration cutover.
**Prevention:** Supervise long maintenance on the host with durable logs, hold the watchdog throughout, and reconcile native task status, configured rootfs and live mount. Preserve both copies and account for post-copy writes before recovery.
**Verification:** Require successful task status, matching SSD configuration/live mount and application health before claiming success or deleting the source.
### [2026-10-17] Misreading DietPi one-shot time synchronization
**Mistake:** Flagged Bahamut's inactive time client and `NTP=no` as a synchronization failure before checking DietPi's scheduler.
**Root cause:** DietPi intentionally runs daily one-shot NTP and stops the daemon after success to save resources.
**Prevention:** Inspect recent sync logs, the installed DietPi helper and synchronization marker before changing time-service policy. For an immediate sync, use `dietpi-timesync 1` rather than adding an unnecessary persistent daemon.
**Verification:** A fresh NTP exchange and marker timestamp confirm success; the daemon stopping afterward is expected and critical edge services stay active.
### [2026-10-15] Scanning SQLite on a saturated source disk during cutover
**Mistake:** Paused Pulse and ran a full SQLite check on its HDD source; the operation exceeded the wrapper deadline and left a blocked read-only helper. The first rollback health check ran before startup was ready.
**Root cause:** Source random-read validation competed with severe HDD I/O pressure, and terminating the wrapper did not cancel the attached helper.
**Prevention:** Preserve the original store, cancel only owned maintenance helpers, stream copy progress, verify checksums privately, run bounded SQLite checks on the SSD copy, and wait for application readiness during rollback. Preserve the already-repaired authentication when retrying storage work.
**Verification:** Pulse health and authenticated access succeed, database file descriptors resolve to SSD, and router/HA/container process identities remain unchanged.
### [2026-10-15] Assuming a package-manager name identifies its executable
**Mistake:** Invoked `nala install` without detecting that a pip mocking tool and local Python modules shadowed the distro frontend.
**Root cause:** PATH and Python import precedence selected unrelated or incompatible packages; dpkg also recorded Python files that were missing.
**Prevention:** Check executable resolution and distro package integrity first. Use the isolated launcher in `docs/setup/nala-system`, preserve the mocking tool as `nala-mock`, and respect the user's preference for interactive confirmations.
**Verification:** Both user and sudo `nala --version` report 0.11.1 and `nala install --help` succeeds; check missing files with `dpkg --verify`.
### [2026-08-03] Reintroducing Docker on the laptop
**Mistake:** Proposed or installed Docker/containerd on the Pop!_OS laptop to run a service.
**Root cause:** Defaulted to the most common deployment pattern instead of reading the host policy. Most upstream project READMEs lead with a `docker run` one-liner, so the habit is strong.
**Prevention:** The laptop is Docker-free by policy. Container workloads go on **CT-300** (`192.168.12.30`). Locally, use libvirt/QEMU or a native install. Podman is present on the laptop and is the correct choice if a container is genuinely required there.
**Verification:** `command -v docker` returns nothing; `dpkg -l | grep -E 'docker|containerd'` is empty.

### [2026-08-07] Cloning repositories into the home directory
**Mistake:** Cloned a new repo into `~` or a subfolder of `~`.
**Root cause:** Assumed the conventional `~/projects`-style layout instead of checking where existing repos live.
**Prevention:** **All repos live in `/media/loufogle/Data/Repos/<name>`.** Home holds no repos. Local-only state that could not be pushed is preserved as git bundles in `Repos/_bundles/<repo>/`.
**Verification:** `ls /media/loufogle/Data/Repos/<name>` resolves and `ls ~ | grep -i <name>` is empty.

### [2026-08-03] Using `apt` instead of `nala`
**Mistake:** Issued `apt install` / `apt update` commands.
**Root cause:** `apt` is the default in virtually all Debian/Ubuntu documentation, so it is the reflexive choice.
**Prevention:** This host uses **`nala`**. Translate package commands before running them.
**Verification:** The command issued begins with `nala`, not `apt`.

### [2026-07-28] Writing Limine config in `key=value` form
**Mistake:** Edited a Limine config entry using `key=value` and `:Name` entry headers.
**Root cause:** Carried over GRUB/older-Limine syntax. Limine v12 changed the format and the failure is silent — the entry is ignored rather than rejected, so the boot just does the wrong thing.
**Prevention:** Limine v12 uses `key: value` (colon-space) and `/Name` entry headers.
**Verification:** The ISO boots to the expected entry under `make run-laptop` rather than falling through to a default.

### [2026-05-xx] Leaving stubs, TODOs or unimplemented paths
**Mistake:** Committed a function containing `TODO`, `FIXME`, `unimplemented!()`, or an empty branch, intending to finish it later.
**Root cause:** Treated the task as "scaffold now, fill in later" when the standing requirement is complete working code per change.
**Prevention:** No `TODO`, `FIXME`, `XXX`, `HACK`, `stub`, or `unimplemented` markers. Every function must be complete and compile clean before committing. The sole documented exception is `SYS_GPU_MMAP`, which is acceptable only with an inline comment scoping it to Phase 5.1+.
**Verification:** `grep -rnE 'TODO|FIXME|XXX|HACK|unimplemented!' kernel/src/` returns only the documented Phase 5.1+ exception.

### [2026-07-xx] Assuming an id-keyed lookup can use 0 as a valid key
**Mistake:** Added a table lookup keyed on pid/id without rejecting `0`, which triple-faulted once a second CPU core began scheduling.
**Root cause:** `0` doubles as both "empty slot" and a potentially valid key. On a single core the ambiguity never surfaced; it only manifested under real SMP. The bug was latent for several phases.
**Prevention:** Every id-keyed lookup must explicitly reject id `0` (K5.6). When adding per-core state, reason about what happens when a second core is live, not just whether it boots.
**Verification:** Boots clean under `make run-laptop` with `-smp 4`, not just `-smp 1`.
