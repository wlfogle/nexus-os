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
