# Monorepo import provenance

The core and macro crates were imported as source snapshots, without rewriting
this repository's Git history or modifying the original repositories:

| Directory | Original repository | Imported commit |
| --- | --- | --- |
| `crates/core` | https://github.com/ryoshrimp/ddd-toolkit-core | `b180edf6f644dd0925e57e392f0370ef11000efe` |
| `crates/macro` | https://github.com/ryoshrimp/ddd-toolkit-macro | `12c07524574b9241fb5e13f813bf0006e3ad456c` |

Source, tests, READMEs, changelogs, and licenses are preserved. Build outputs,
trybuild `wip` output, nested lockfiles, and standalone repository CI/settings
are not imported. CI and release configuration live at the workspace root;
all crates share the root `Cargo.lock`. Earlier history remains available in
the original repositories at the commits above.

The migration retains crate names, versions, public APIs, and feature behavior.
In particular, derive consumers still need a direct core dependency; changing
macro dependency resolution is a separate API improvement, not part of moving
repositories. Older changelog entries and release-specific README text describe
their original releases and are retained as historical documentation.

After merging, maintainers can update the old repositories' READMEs to point
here and archive them. Neither archiving, remote changes, nor crates.io
publication is performed by this migration.
