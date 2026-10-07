# kappfinder-rs

Finds installed applications that have no menu entry and generates
freedesktop.org `.desktop` files for them.

A maintained, desktop-agnostic reimplementation of KDE's
[KAppFinder](https://apps.kde.org/kappfinder/), which KDE marked unmaintained
and stopped releasing (Gentoo removed it as a dead package in 2015). Nothing
has replaced the specific thing it did, so this does.

## What it actually does

KAppFinder was **not** a launcher. It shipped a catalog of templates for known
programs, scanned `$PATH` to see which of them were installed, checked which
already had a menu entry, and offered to create entries for the rest.

`kappfinder-rs` works the same way, and for the same reason: a blind sweep of
`$PATH` is useless. On a normal desktop that is ~5000 binaries, almost all of
them `ls`, `awk`, and library helpers. The catalog is what makes the output
worth reading.

The blind sweep is still available as `orphans`, but it is opt-in, read-only,
and never creates anything on its own.

## Build

```fish
cargo build --release
install -Dm755 target/release/kappfinder-rs ~/.local/bin/kappfinder-rs
```

No system dependencies beyond a Rust toolchain. `desktop-file-utils` is
optional — it is only used to refresh the desktop database after writing.

## Usage

### Scan (default, read-only)

```fish
kappfinder-rs
kappfinder-rs scan
```

```
kappfinder-rs — application menu entry finder

  catalog templates     117
  installed on $PATH    37
  already in the menu   4
  missing an entry      33
  desktop files seen    577 across 6 directories

Installed but missing a menu entry:

    1  alsamixer    terminal  ALSA Mixer
    2  btop         terminal  btop++
    3  ncdu         terminal  ncdu
    …
```

### Install

Interactive by default — it lists candidates and waits for a selection.

```fish
kappfinder-rs install                 # prompt: all / none / 1,3,5-7
kappfinder-rs install --dry-run       # show what would be written
kappfinder-rs install --yes           # accept everything
kappfinder-rs install --force         # overwrite existing files
```

Entries go to `$XDG_DATA_HOME/applications` (usually
`~/.local/share/applications`). Override with `--dir`.

Existing files are never overwritten without `--force`. Writes go through a
temporary file and a rename, so an interrupted run cannot leave a half-written
entry in your menu.

### Create specific entries

Works for catalogued and uncatalogued binaries alike. Uncatalogued ones get
sensible generated defaults.

```fish
kappfinder-rs create btop ncdu
kappfinder-rs create /opt/vendor/bin/thing
```

### List binaries with no entry (heuristic)

```fish
kappfinder-rs orphans --limit 50
kappfinder-rs orphans --no-filter      # include coreutils, libraries, etc.
```

Read-only. The filter is best-effort and will have false positives and
negatives; treat the output as a starting point, not a worklist.

### Undo

Every generated file carries `X-KAppFinder-Generated=true`. `remove` deletes
only files carrying that marker, and only inside the target directory or your
own applications directory — hand-written and system entries are never
touched.

```fish
kappfinder-rs remove --dry-run
kappfinder-rs remove
```

Editing a generated file is safe. Deleting the marker line opts that file out
of `remove` permanently.

### Inspect the catalog

```fish
kappfinder-rs catalog
kappfinder-rs catalog --missing        # templates whose binary isn't installed
```

## Global options

| Flag | Meaning |
| --- | --- |
| `--dir DIR` | Write entries here instead of `$XDG_DATA_HOME/applications` |
| `--catalog FILE` | Use this user catalog instead of the default path |
| `--ignore-nodisplay` | Treat `NoDisplay`/`Hidden` entries as missing |

`--ignore-nodisplay` is useful when a package ships a deliberately hidden entry
and you want a visible one anyway.

## Extending the catalog

Create `~/.config/kappfinder-rs/catalog.toml`. It uses the same syntax as the
built-in catalog (`data/catalog.toml`). An entry with the same `bin` as a
built-in replaces it outright.

```toml
[[app]]
bin = "myapp"
name = "My Application"
generic_name = "Widget Editor"
comment = "Edits widgets"
icon = "applications-development"       # icon *theme name*, not a path
categories = ["Development"]            # first element must be a main category
terminal = false
keywords = ["widget", "editor"]
exec_args = "%F"                        # appended after the binary path
mime_types = ["text/plain"]
```

Verify it loaded with `kappfinder-rs catalog`.

## How "already has an entry" is decided

Every `.desktop` file under the XDG application directories — including
Flatpak and Snap exports — is parsed, and the program each one launches is
recorded. Matching is by:

1. `TryExec`, which is unambiguous; then
2. the program extracted from `Exec`, seeing through `env`, `sh -c`, `nohup`,
   `pkexec` and similar wrappers; then
3. the desktop file stem, as a fallback for `Exec` values too convoluted to
   parse.

## Correctness

```fish
cargo test                                  # 37 unit tests
cargo clippy --all-targets -- -D warnings
```

Tests cover `Exec` tokenising and wrapper unwrapping, Desktop Entry
escape/unescape round-tripping, selection-expression parsing, filename
sanitising, and catalog invariants (exactly one main category per entry,
`Audio`/`Video` always paired with `AudioVideo`, no duplicate categories).

Generated output is verified against `desktop-file-validate` from
`desktop-file-utils` with zero errors, warnings, or hints.

Note on `Version=1.0`: that key names the *spec revision* the file claims to
follow, not the application version. `desktop-file-validate` only recognises
`1.0` and `1.1`, so `1.0` is emitted for maximum compatibility.

## Limitations

- Linux/Unix only — uses Unix permission bits to detect executables.
- The `orphans` filter is heuristic by nature. It is deliberately kept out of
  the catalog-driven path, which is exact.
- Icons use theme names. A generated entry for an application with no themed
  icon falls back to a generic one.

## Licence

GPL-3.0-or-later, matching the original KAppFinder's GPL-2 lineage.
