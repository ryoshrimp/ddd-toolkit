# Releasing the workspace

The three crates remain published separately, but one workflow invocation
publishes them in order: `ddd-toolkit-core`, `ddd-toolkit-macro`, `ddd-toolkit`.
`facade-tests` is never published. No new release tool is required.

## Prepare a release PR

1. Update the versions of changed crates in their `Cargo.toml` files.
   Versions remain independent (currently facade 0.4.1, core 0.4.3, macro 0.5.1);
   publishing is always an explicit operation.
2. When core or macro changes, update `[workspace.dependencies]` in the root
   manifest. Bump dependent crates as needed so users can receive the update.
   Dependency requirements should not admit an older version missing APIs used
   by the new release. Update version-specific examples/docs as appropriate.
3. Update the relevant changelogs (`CHANGELOG.md`, `crates/core/CHANGELOG.md`,
   `crates/macro/CHANGELOG.md`) and run `cargo check --workspace` to refresh the
   shared `Cargo.lock`. Commit it with the manifests.
4. Run the checks in `CONTRIBUTING.md`, then merge the PR.

### Generated-code compatibility gate

Review [the supported core/macro matrix](macro-compatibility.md) before changing
traits or `__private` generated-code helpers. A hidden name is not permission to
break emitted code in a patch. For a 0.x incompatible change, bump the affected
crate's minor version, bump macro if its generated code/compatibility range
changes, and bump facade if its exposed API or required dependencies change.
Update dependency lower bounds, migration notes, the matrix and consumer tests
in the same PR; identical version numbers are not required.

The Python checks include exact released minimum-core and current-core consumers,
direct/facade dependencies, dependency renames, and both serde configurations.
They must pass in CI and Release verification before publication. A newly raised
minimum must be tested from an actual released core; if it is not published yet,
retain an explicitly identified local candidate test and add the registry test
when available, rather than claiming a substituted local path is released-core
coverage. Publish core → macro → facade and preserve the documented supported
range throughout partial release recovery. Out-of-range consumers must migrate
to the documented current combination.

## One-time Trusted Publishing setup

Publishing uses GitHub Actions OIDC, not a stored crates.io API token.

1. Create the GitHub `crates-io` deployment environment. Restrict deployment
   branches to `main`; enable required reviewers if supported by your plan.
2. As an owner on crates.io, open the **Trusted Publishing** settings for
   **each** of `ddd-toolkit-core`, `ddd-toolkit-macro`, and `ddd-toolkit`.
   Add the same GitHub publisher to all three:

   | Field | Value |
   | --- | --- |
   | Repository owner | `ryoshrimp` |
   | Repository name | `ddd-toolkit` |
   | Workflow filename | `release.yml` (not the full path) |
   | Environment | `crates-io` |

   The environment must be specified on crates.io as well as in the workflow.
   With matching publisher settings, the workflow identity can publish all
   three crates using the temporary token. These crates are already published;
   a brand-new crate still needs an initial publication before setup.
3. After migration, revoke any old API token dedicated to this workflow on
   crates.io and remove its `CARGO_REGISTRY_TOKEN` GitHub secret. Check other
   uses before revoking a shared token. No persistent registry secret is needed.

These account-side settings must be configured by a maintainer; changing the
workflow file does not configure crates.io or GitHub environment protections.
There is intentionally no fallback to a long-lived token.

### Security model

Compared with the previous stored-token workflow:

- There is no long-lived publishing secret to leak, rotate, or retain in backups.
- crates.io verifies the repository, workflow filename, and environment using
  GitHub's signed OIDC identity. Each crate explicitly trusts that identity.
- The exchanged publishing token expires after 30 minutes. The official auth
  action also revokes it in its post-job cleanup. Authentication happens after
  verification and toolchain installation, immediately before publication.
- Only the `publish` job has `id-token: write`. The job is restricted to manual
  runs on `main`, uses the protected environment, does not restore build caches,
  and does not persist checkout credentials. Release actions are pinned to full
  commit SHAs; review and update these pins periodically.

This reduces credential exposure; it does not make a compromised workflow safe.
Code/build scripts running in the publishing job can still misuse the temporary
credential while it is valid. Protect `main`, review changes to release workflows
and scripts/dependencies, and enforce environment protections. The branch check
inside YAML alone is not a substitute for those external protections.

References: [crates.io Trusted Publishing](https://crates.io/docs/trusted-publishing),
[official authentication action](https://github.com/rust-lang/crates-io-auth-action).

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
python3 scripts/release.py --publish  # irreversible; local cargo credentials required
```

The local publish command does not perform GitHub OIDC authentication; it uses
locally configured Cargo credentials. Prefer the GitHub workflow for releases.
Never copy the workflow's temporary token to local files or logs. If a release
outlasts the temporary token, rerun the workflow to obtain a fresh one; versions
already published will be skipped.

Cargo verifies published packages using registry dependencies, not workspace
paths. A dry-run of dependent crates may fail when it requires a new core/macro
version that has not been published yet; dry-run does not populate the registry.
Workspace tests still verify the local combination. During an actual release,
Cargo publishes and waits for each dependency to become available before the
next package is published. Registry state is not changed by the migration.

This workflow does not create Git tags or GitHub Releases automatically.
