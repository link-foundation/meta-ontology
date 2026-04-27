## Problem

The Rust CI/CD template's `deploy-docs` job depends on release publication jobs:

```yaml
deploy-docs:
  needs: [auto-release, manual-release]
```

and only runs when `auto-release` or `manual-release` succeeds. This couples GitHub Pages documentation deployment to package/GitHub release publication.

## Reproducible Example

1. Use the current `release.yml` from `link-foundation/rust-ai-driven-development-pipeline-template`.
2. Run the workflow on `main` with a package build that succeeds.
3. Make the release publication path fail, for example by missing/invalid crates.io credentials, a crates.io publish failure, or a release script failure.
4. Observe that `Deploy Rust Documentation` is skipped because it needs the failed release job.

The same pattern reproduced in `link-foundation/meta-ontology`:

- `Build Package` succeeded.
- `Auto Release` failed.
- `Deploy Rust Documentation` was skipped.

See downstream issue: https://github.com/link-foundation/meta-ontology/issues/3

## Workaround

After a release failure, manually rerun a documentation deployment job or manually build and publish `target/doc` to GitHub Pages.

## Suggested Fix

Make documentation deployment depend on the successful validation/build job instead of release publication jobs. For example:

```yaml
deploy-docs:
  needs: [build]
  if: |
    !cancelled() &&
    needs.build.result == 'success' && (
      (github.event_name == 'push' && github.ref == 'refs/heads/main') ||
      (github.event_name == 'workflow_dispatch' && github.event.inputs.release_mode == 'instant')
    )
```

This keeps the website release tied to a successful build while allowing it to proceed when package publication or GitHub release creation fails.
