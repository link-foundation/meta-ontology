# Roadmap

A staged plan for building **meta‑ontology** from MVP to a public‑domain library
+ CLI + microservice + WASM/React web app, as specified by
[issue #1](https://github.com/link-foundation/meta-ontology/issues/1).

The numbered requirement IDs (`R1` … `R17`) refer to [REQUIREMENTS.md](REQUIREMENTS.md).

## Milestone status legend

- ✅ done in PR #2 (first working session)
- 🚧 partial in PR #2, completion follow‑up
- ⏳ planned, not yet started
- ❄ deferred (lower priority)

## Milestone M0 — Bootstrap (this PR)

| ID | Item | Status |
|---|---|---|
| R7 | Adopt `links-notation` for data storage | ✅ |
| R8 | CLI uses `lino-arguments` | ✅ (inherited from template) |
| R10 | Library crate with `Ontology`/`Concept` API | ✅ |
| R11 | CLI binary with `list`, `show`, `check-words` | ✅ |
| R2 | Seed NSM 65 primes (English) | ✅ |
| R3 | Seed lino self‑referential primes (`link`, `thing`, …) | ✅ |
| R4 | Network structure (cycle present) | ✅ |
| R5 | Self‑describing — first cut | 🚧 |
| R6 | CI word‑coverage check | ✅ |
| R14 | Multiple definitions per concept | ✅ |
| R15 | Unlicense in `Cargo.toml` | ✅ |
| R16 | `REQUIREMENTS.md`, `ROADMAP.md`, `docs/plan/` | ✅ |
| R17 | Case study `docs/case-studies/issue-1/` | ✅ |
| R1 | Survey of popular ontologies | ✅ (research notes; data ingestion in M2) |
| R9 | `lino-objects-codec` integration | 🚧 (feature‑gated) |
| R12 | Microservice | ⏳ (M3) |
| R13 | WASM + React web app | ⏳ (M4) |

**Exit criteria for M0:** PR #2 merges; CI green; the CLI prints concept
information; the word‑coverage check fails on a deliberately introduced
unknown word.

## Milestone M1 — Make the kernel solid

Goal: deepen the kernel without expanding scope.

- Expand `data/concepts/core.lino` so every word used inside `data/` is itself
  a concept (zero‑allowlist for the data folder).
- Add a stricter tokenizer that handles plurals, contractions, hyphenation,
  Markdown code spans/blocks, and Rust identifiers correctly.
- Add `meta-ontology check-words` golden tests with synthetic fixtures
  (a word that *should* be missing → exit non‑zero with the right message).
- Add `meta-ontology dot` to emit a Graphviz DOT graph of the ontology so
  reviewers can eyeball cycles and clusters.
- Add `meta-ontology graph --json` to emit Cytoscape‑compatible JSON for the
  future web app.

**Exit:** repository's `data/` folder has zero allow‑listed words; tests
cover happy and failure paths.

## Milestone M2 — Multilingual exponents and cross‑ontology mapping

Goal: deliver real value on R2 and R1.

- For each of the top‑20 languages with an authoritative NSM exponent table,
  ingest the table into `data/primes/exponents/<lang>.lino`.
- For each of the remaining ~7 languages, mark the file `coverage: provisional`
  and seed it with the closest neighbour (e.g. Malay → Indonesian).
- Ingest cross‑ontology mappings into `data/concepts/mappings.lino`:
  - Schema.org root subset (`Thing`, `Action`, `Event`, `Person`, …)
  - OWL / RDFS built‑ins (`owl:Thing`, `rdf:Property`, `rdfs:Class`)
  - SUMO upper concepts (`Entity`, `Physical`, `Abstract`, `Object`, `Process`)
  - Wikidata top items (`entity Q35120`, `class Q16889133`)
- Document the ingestion pipeline in `docs/plan/04-ontology-data.md`.
- Add CLI `meta-ontology langs` that lists language coverage, and
  `meta-ontology mappings <name>` that lists cross‑ontology equivalences.

**Exit:** all 65 primes have ≥3 language exponents; ≥5 ontologies are mapped.

## Milestone M3 — Microservice (R12)

Goal: serve the ontology over HTTP.

- Add an `meta-ontology-server` binary (or `--features server` build) using
  `axum` + `tokio`.
- Endpoints:
  - `GET /healthz`
  - `GET /concepts` — list concept names (paginated)
  - `GET /concepts/:id` — concept detail (definitions, neighbours, mappings)
  - `GET /neighbors/:id` — adjacency
  - `GET /search?q=` — substring + label search
- Content negotiation: `application/json` (default) and `text/lino` via
  `Accept`. Use `lino-objects-codec` for `text/lino`.
- Add Dockerfile and a smoke‑test workflow that runs `cargo run` and curls
  the endpoints.
- Document the service in `docs/plan/05-microservice.md`.

**Exit:** a running container answers all endpoints; integration tests use
`reqwest` against a local server.

## Milestone M4 — Web app (R13)

Goal: see and explore the ontology in a browser.

- Add a `crates/meta-ontology-wasm/` crate that re‑exports the library API via
  `wasm-bindgen` (search / lookup / neighbours).
- Add a `web/` directory with `vite` + `@vitejs/plugin-react` +
  `vite-plugin-wasm`. Use `pnpm` (or `npm`) — pinned via lockfile.
- Visualise the graph with `react-cytoscapejs` or `reactflow`. Show:
  - Each concept as a node, with definition snippets on hover/click.
  - Each definition link as an edge, with the relation label.
  - Cycles emphasised (e.g. coloured edges).
  - Language switcher that swaps prime labels.
- Deploy the production build to GitHub Pages alongside the existing
  `cargo doc` artefact (separate paths; the docs deploy doesn't break).
- Document the web app in `docs/plan/06-web-app.md`.

**Exit:** `https://link-foundation.github.io/meta-ontology/app/` shows the
graph; the existing docs page still works.

## Milestone M5 — Hardening, polish, governance

- ❄ Performance: benchmark the loader on the full data set; consider an
  on‑disk index if cold start is slow.
- ❄ Stability: stabilise the public Rust API; release `1.0.0`.
- ❄ Schema validation: add a lino schema (the meta‑ontology of the
  meta‑ontology) and check that `data/**/*.lino` conforms.
- ❄ Contributor experience: a `cargo xtask new-concept <name>` scaffold; a
  `templates/` folder with starter `.lino` files.
- ❄ Multi‑language docs: render the README and CONTRIBUTING in each language
  for which we have NSM exponents.
- ❄ Continuous web deploy: deploy a preview app per pull request.

## Out of scope

The following are *not* in the roadmap unless re‑opened:

- Reasoning / inference (this is a meta‑ontology, not an OWL DL reasoner).
- Editing the ontology through the web UI (read‑only).
- Hosting a SPARQL endpoint (open question; track in a separate issue if
  needed).

## Summary diagram

```
M0 ── library + CLI + seed data + CI check + docs (this PR)
   │
M1 ── kernel hardening, zero‑allow‑list under data/
   │
M2 ── 20 languages + cross‑ontology mappings
   │
M3 ── HTTP microservice (axum)
   │
M4 ── WASM + React web app on GitHub Pages
   │
M5 ── hardening, governance, 1.0.0
```
