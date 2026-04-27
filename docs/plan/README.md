# Implementation plan

This folder contains the detailed, per‑slice implementation instructions for
the meta‑ontology project. The high‑level milestones live in
[`ROADMAP.md`](../../ROADMAP.md); this folder explains *how* to deliver each
slice.

| File | Slice | Milestone |
|------|-------|-----------|
| [`01-overview.md`](./01-overview.md) | Architecture overview | All |
| [`02-library.md`](./02-library.md) | Rust library | M0 / M1 |
| [`03-cli.md`](./03-cli.md) | CLI tool | M0 / M1 |
| [`04-ontology-data.md`](./04-ontology-data.md) | Ontology data files | M0 / M2 |
| [`05-microservice.md`](./05-microservice.md) | HTTP microservice | M3 |
| [`06-web-app.md`](./06-web-app.md) | WASM + React app | M4 |
| [`07-ci-coverage-check.md`](./07-ci-coverage-check.md) | CI word‑coverage check | M0 / M1 |

## How to use this plan

- Open the file for the slice you're working on.
- Each file is a checklist plus enough rationale to make decisions later.
- When a step is shipped, mark it `[x]` in a PR.
- New steps are appended at the bottom of each file with a date marker.
