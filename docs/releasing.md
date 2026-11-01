# Releasing the workspace

The three crates remain published separately, but one workflow invocation
publishes them in order: `ddd-toolkit-core`, `ddd-toolkit-macro`, `ddd-toolkit`.
`facade-tests` is never published. No new release tool is required.

## Prepare a release PR

1. Update the versions of changed crates in their `Cargo.toml` files.
   Versions remain independent (currently facade 0.3.0, core/macro 0.4.0);
   this migration does not release or bump any version automatically.
2. When core or macro changes, update `[workspace.dependencies]` in the root
   manifest. Bump dependent crates as needed so users can receive the update.
   Dependency requirements should not admit an older version missing APIs used
   by the new release. Update version-specific examples/docs as appropriate.
3. Update the relevant changelogs (`CHANGELOG.md`, `crates/core/CHANGELOG.md`,
   `crates/macro/CHANGELOG.md`) and run `cargo check --workspace` to refresh the
   shared `Cargo.lock`. Commit it with the manifests.
4. Run the checks in `CONTRIBUTING.md`, then merge the PR.

## One-time GitHub setup

- Create the `crates-io` deployment environment. Restrict it to `main` and
  optionally require approval.
- Configure its `CARGO_REGISTRY_TOKEN` secret with a crates.io token authorized
  to publish all three crates. Existing crate ownership must permit publishing.

## Release with one operation

From Actions, select **Release**, choose `main`, enable **publish**, and run.
The workflow runs the checks before publishing. Without **publish**, it only
verifies packages using `cargo publish --dry-run` and does not need a token.

The script skips exact versions already published on crates.io, so unchanged
crates do not need a new version. A partially completed release can be resumed
by re-running the same workflow. Published crates cannot be rolled back as a
transaction; if a published version is incorrect, prepare a new version.
Yanked versions and registry errors fail the operation rather than being
silently skipped. An existing version is immutable: local changes will NOT
replace it. Always bump versions for changes intended for publication.

Equivalent local commands (use a clean, committed tree):

```sh
python3 scripts/release.py            # dry-run by default
python3 scripts/release.py --publish  # irreversible; token required
```

Cargo verifies published packages using registry dependencies, not workspace
paths. A dry-run of dependent crates may fail when it requires a new core/macro
version that has not been published yet; dry-run does not populate the registry.
Workspace tests still verify the local combination. During an actual release,
Cargo publishes and waits for each dependency to become available before the
next package is published. Registry state is not changed by the migration.

This workflow does not create Git tags or GitHub Releases automatically.
