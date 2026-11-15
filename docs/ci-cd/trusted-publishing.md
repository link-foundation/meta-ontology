# crates.io trusted publishing

The `auto-release` and `manual-release` jobs in `.github/workflows/release.yml`
request `id-token: write` and use the pinned `rust-lang/crates-io-auth-action`.
Authentication runs after the release build, immediately before publishing.
The publish step supplies `CARGO_REGISTRY_TOKEN` in this order:

1. The auth action's short-lived OIDC token.
2. The `CARGO_REGISTRY_TOKEN` secret.
3. The legacy `CARGO_TOKEN` secret.

The auth action uses `continue-on-error: true` so a missing registration or
unavailable OIDC exchange can use the existing token fallback. A failed publish
still fails the job. Keep both legacy secrets available as fallback; this PR
does not delete them. Credentials are scoped to the publish step, rather than
all build and test steps. The action revokes its temporary token after the job.

## Release behavior

- Pull requests run validation and package builds, without publishing.
- Pushes to `main` run Auto Release only after lint, tests, and the package build
  succeed. Changelog fragments determine the version bump. The patch fragment
  included with this migration is the release trigger; do not manually bump
  `Cargo.toml` in the PR (the version-modification check forbids that).
- Manual dispatch with `release_mode=instant` runs Instant Release after the same
  checks, using the selected bump type. Select `main` for production releases.
- Manual dispatch with `release_mode=changelog-pr` creates a changelog PR rather
  than publishing a package.
- Manual dispatch with `release_mode=bootstrap` publishes the committed version
  without bumping, tagging, or pushing a release commit, then attaches the
  `release.yml` trusted publisher through the crates.io API. It requires a crate
  owner token with initial publishing and trusted-publishing permissions in
  `CARGO_REGISTRY_TOKEN` or `CARGO_TOKEN`. An already published version is skipped
  so registration can be retried. Run it on the reviewed commit with:
  `gh workflow run release.yml --ref issue-8-7c6c697378a0 -f release_mode=bootstrap -f bump_type=patch`.
  The required bump input is unused by bootstrap mode. The same lint, test, and
  build gates apply. API errors fail the job with an explicit diagnostic.
- Documentation deployment depends on a successful package build and remains
  independent of registry publication.

## Register the publisher and verify the migration

[crates.io's setup requirements](https://crates.io/docs/trusted-publishing)
include an existing published crate and a crate owner account. An initial
publication requires an API token. Once the crate exists, register:

| Setting | Value |
| --- | --- |
| Crate | `meta-ontology` |
| GitHub owner | `link-foundation` |
| Repository | `meta-ontology` |
| Workflow filename | `release.yml` |
| Environment | Leave empty (neither release job specifies one) |

If using the migration tool that created PR #7, rerun its `setup --migrate`
command in its authenticated environment and confirm that it registers the
caller workflow above. That tool is not part of this repository. Alternatively,
a crate owner can configure the same publisher in the crate's crates.io
settings. Do not register a helper workflow instead of `release.yml`.

After registration, use the normal release path on `main` to publish a new
version. Verify that the auth action **succeeded**, the publish step reported
`publish_result=success`, and crates.io lists that version with trusted-publisher
metadata. Record the run URL and published version in PR #7. A green job using
fallback, a dry run, or an already-published version is not proof of an OIDC
release. Never print tokens to establish which credential was used.

As of the 2026-11-15 investigation, the public API returned HTTP 404 with
`crate meta-ontology does not exist`. The issue-solver environment had neither
local registry credentials nor GitHub Actions OIDC credentials, and no migration
tool was installed. The organization `CARGO_TOKEN` secret is available to this
repository's Actions jobs; bootstrap mode can attempt initial publication and
publisher registration without exposing its value. Its validity and permissions
must be established by a real run. Preserve the fallback secrets.

Automated version bumps now refresh and stage the workspace lockfile before
creating the release commit and tag. The package build also runs
`cargo publish --dry-run --locked` and requires a clean worktree, catching a
stale lockfile before the publishing job.

## Local validation

Run `python3 experiments/test-publish-auth.py` to compile the actual publisher
script and exercise it against a local Cargo stub. It covers native/OIDC token
precedence, legacy and empty-token fallbacks, missing authentication, duplicate
versions, and registry failures without uploading a crate. CI runs this check
in the lint job. These mocked tests verify the helper's behavior; they do not
verify the crates.io trust configuration or perform the OIDC exchange.
Run `python3 experiments/test-register-trusted-publisher.py` for mocked publisher
registration, pagination, retry, permission failure, and response checks.
