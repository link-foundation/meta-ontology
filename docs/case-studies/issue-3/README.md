# Issue 3 Case Study: Documentation Deployment Must Not Depend on Package Release

## Summary

Issue: [link-foundation/meta-ontology#3](https://github.com/link-foundation/meta-ontology/issues/3)

The GitHub Pages documentation deployment was coupled to package release jobs. When the package release path failed, the website deployment at `https://link-foundation.github.io/meta-ontology/` was skipped even though CI validation and the package build had already succeeded.

## Collected Data

- `raw-data/issue-3.json`: issue metadata and full issue body.
- `raw-data/issue-3-comments.json`: issue comments, empty at investigation time.
- `raw-data/pr-4*.json`: PR metadata, review comments, conversation comments, and reviews.
- `raw-data/main-runs.json`: recent main-branch workflow runs.
- `raw-data/main-run-24979900862.json` and `raw-data/main-run-24979900862.log.gz`: initial main push run metadata and logs.
- `raw-data/main-run-24983875003.json` and `raw-data/main-run-24983875003.log.gz`: run for merge commit `c162cc702f8de991634d97de8394763621d3349f`.
- `template-data/*-template-tree.txt`: CI/CD-related file tree snapshots from the JavaScript and Rust pipeline templates.
- `template-data/*-template-release.yml`: release workflow snapshots from both templates.

## Timeline

- 2026-04-27 06:26 UTC: main run `24979900862` started for the initial commit and failed in `Auto Release`.
- 2026-04-27 06:29 UTC: `Build Package` succeeded in run `24979900862`.
- 2026-04-27 06:30 UTC: `Auto Release` failed in `Check if version already released or no fragments`.
- 2026-04-27 06:30 UTC: `Deploy Rust Documentation` was skipped.
- 2026-04-27 08:11 UTC: main run `24983875003` started for merge commit `c162cc702f8de991634d97de8394763621d3349f` and reproduced the same pattern.
- 2026-04-27 08:13 UTC: `Build Package` succeeded in run `24983875003`.
- 2026-04-27 08:15 UTC: `Auto Release` failed, then `Deploy Rust Documentation` was skipped.
- 2026-04-27 08:25 UTC: issue 3 was opened to require website deployment regardless of package release success.

## Requirements From the Issue

1. Website deployment to GitHub Pages must work independently of package release success.
2. CI/CD files should be compared against the JavaScript and Rust pipeline templates.
3. Logs and related issue/PR data should be preserved under `docs/case-studies/issue-3`.
4. The case study should reconstruct the timeline, list requirements, identify root causes, and propose solutions.
5. Same issues found in referenced templates should be reported upstream with a reproducible example, workaround, and fix suggestion.
6. If data is insufficient, add debug output or verbose mode for future investigation.

## Root Causes

### Root Cause 1: Documentation Deployment Depends on Release Publication

The workflow configured `deploy-docs` with:

```yaml
needs: [auto-release, manual-release]
```

and only allowed deployment when one of those release jobs succeeded. That makes documentation deployment depend on package release publication instead of on the package build. In both preserved main runs, `Build Package` succeeded but `Deploy Rust Documentation` was skipped after `Auto Release` failed.

GitHub Actions supports job-level conditions and the `needs` context for dependent job results. The `needs.<job_id>.result` value can be `success`, `failure`, `cancelled`, or `skipped`, and job-level `if` expressions decide whether a job is sent to a runner. References:

- GitHub Actions contexts reference: https://docs.github.com/en/actions/reference/workflows-and-actions/contexts
- GitHub Actions job conditions: https://docs.github.com/en/actions/how-tos/write-workflows/choose-when-workflows-run/control-jobs-with-conditions

### Root Cause 2: Release Script Fails Under `RUSTFLAGS=-Dwarnings`

`Auto Release` failed before crate publishing. The preserved logs show `rust-script scripts/check-release-needed.rs` failing because warnings are treated as errors:

- `scripts/check-release-needed.rs:44`: unused `get_arg`
- `scripts/rust-paths.rs:106`: unused `get_cargo_lock_path`
- `scripts/rust-paths.rs:115`: unused `get_changelog_dir`
- `scripts/rust-paths.rs:124`: unused `get_changelog_path`
- `scripts/rust-paths.rs:133`: unused `needs_cd`
- `scripts/rust-paths.rs:138`: unused `parse_rust_root_from_args`
- `scripts/rust-paths.rs:258`: unused `main`

The shared `rust-paths.rs` helper is included by several `rust-script` binaries, and each binary intentionally uses only a subset of the helper API. That pattern needs an explicit dead-code allowance in the shared helper module, otherwise release-only script execution can fail even after normal `cargo clippy` and `cargo test` succeed.

## Template Comparison

### Rust Template

The Rust template currently has the same documentation deployment dependency pattern:

```yaml
deploy-docs:
  needs: [auto-release, manual-release]
```

This means the same failure mode can occur there. The upstream issue is tracked as `link-foundation/rust-ai-driven-development-pipeline-template#38`.

### JavaScript Template

The JavaScript template does not have a GitHub Pages documentation deployment job, so the same website-specific issue was not found there. It does contain newer CI/CD practices worth preserving for future Rust workflow maintenance, especially main-only cancellation and avoiding unnecessary `always()` usage where normal cancellation/failure propagation is preferable.

## Implemented Solution

1. Changed `deploy-docs` to depend on `build`, not on `auto-release` or `manual-release`.
2. Kept deployment gated to successful package builds on main pushes and manual instant releases.
3. Removed the unused `get_arg` helper from `check-release-needed.rs`.
4. Marked the shared `rust-paths.rs` helper module with `#![allow(dead_code)]` because it is intentionally included by scripts that use different subsets of the API.
5. Added a CI/CD unit test that asserts `deploy-docs` depends on `build` and does not reference release publication job results.

## Reproduction

Before the fix:

1. Trigger the current main workflow with changelog fragments present.
2. Let `Build Package` succeed.
3. Cause `Auto Release` to fail in release-only scripting or package publication.
4. Observe `Deploy Rust Documentation` skipped because it needs the release jobs.

The preserved logs show this exact pattern in runs `24979900862` and `24983875003`.

## Verification Plan

- `cargo fmt --check`
- `cargo test --all-features --verbose`
- `cargo test --doc --verbose`
- `RUSTFLAGS=-Dwarnings HAS_FRAGMENTS=true rust-script scripts/check-release-needed.rs`
- `cargo clippy --all-targets --all-features`
- `rust-script scripts/check-file-size.rs`

## Upstream Follow-Up

Filed upstream Rust template issue: [link-foundation/rust-ai-driven-development-pipeline-template#38](https://github.com/link-foundation/rust-ai-driven-development-pipeline-template/issues/38)

The report includes:

- Reproduction: `deploy-docs` depends on `auto-release` and `manual-release`; a failed release job skips Pages deployment even after build success.
- Workaround: rerun or manually deploy documentation after package release failures.
- Suggested fix: make `deploy-docs` depend on the successful validation/build job rather than release publication jobs.
