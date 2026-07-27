---
bump: minor
---

### Added

- Added an OpenMetadata-inspired catalog contract backed by `meta-language`,
  including stable identities, typed relationships, provenance, governance,
  layered diagnostics, deterministic import plans, JSON Schema interchange,
  fixture ingestion with checkpoints, and linear search.
- Added CLI commands for validation, JSON/schema export, import dry runs, and
  catalog search, with unit, integration, and example coverage.
- Added the issue 5 case study, primary-source inventory, requirement map, and
  explicit decision gates for deferred service-scale components.
- Raised the minimum supported Rust version to 1.88 because
  `meta-language` 0.54's dependency graph uses Rust 2024 let chains.
