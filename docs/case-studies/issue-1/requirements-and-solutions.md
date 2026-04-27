# Requirements → Proposed Solutions

Per‑requirement analysis: prior art reviewed, proposed solution, and acceptance
criteria. Cross‑references the canonical [`REQUIREMENTS.md`](../../../REQUIREMENTS.md).

## R1 — Collect data about the most popular ontologies

- **Prior art:** Schema.org, OWL/RDF, WordNet, ConceptNet, BabelNet, Cyc, SUMO,
  DBpedia, Wikidata, FrameNet (see [`research-popular-ontologies.md`](./research-popular-ontologies.md)).
- **Solution:** treat each ontology as an external "namespace" in our data folder
  (`data/external/<source>.lino`), record only the top‑level "primitive" concepts
  and store cross‑ontology equivalences in `data/concepts/mappings.lino`.
- **Acceptance:** ≥10 ontologies surveyed; ≥5 primitive concepts mapped per
  ontology; mapping format documented and validated by the loader.

## R2 — NSM semantic primes in top‑20 languages

- **Prior art:** [`research-nsm.md`](./research-nsm.md). 65 primes; ~13/20 of the
  top languages have authoritative exponent tables.
- **Solution:** seed `data/primes/nsm.lino` with the 65 English primes and a
  language table for each prime. For languages without an authoritative table,
  store `(coverage: provisional)` to make the gap explicit.
- **Acceptance:** all 65 primes present; English exponents complete; ≥3 other
  languages with full exponents in MVP; CI surfaces missing exponents.

## R3 — Include cyclic / "prime" concepts from world ontologies

- **Prior art:** see candidate list in [`research-popular-ontologies.md`](./research-popular-ontologies.md).
- **Solution:** seed `data/concepts/core.lino` with the lino primes (`link`,
  `reference`, `name`, `thing`, `concept`, …) and define them via each other.
  Add equivalence links to Schema.org / OWL / Wikidata / SUMO equivalents.
- **Acceptance:** ≥10 cyclic primes defined; each has ≥1 cross‑ontology mapping.

## R4 — Network (graph), not acyclic tree

- **Prior art:** Links Notation supports it natively (named links + reuse → graph).
- **Solution:** parse `.lino` files with `links-notation`, then build an
  in‑memory `petgraph::DiGraph` (no acyclicity constraint). The library exposes
  cycles as first‑class data.
- **Acceptance:** at least one cycle exists in the seed data
  (`thing → concept → thing`); the loader does not reject cycles; tests assert
  the cycle is reachable.

## R5 — Self‑describing: every word in the repo is a concept

- **Prior art:** none directly comparable. Adjacent: typed natural‑language
  glossaries, controlled vocabularies, and "linguistically reviewed" docs in
  large standards bodies.
- **Solution:** define a fixed set of repository file types (Markdown, Rust
  source, lino) and a tokenizer that extracts "human‑language words" from each.
  Compare against the ontology's known names + an explicit allow‑list
  (`data/allowlist.lino`) for things that should never be ontology concepts
  (proper nouns, identifiers, version strings).
- **Acceptance:** running `meta-ontology check-words .` returns 0 unknown words
  on a green main branch; CI reproduces the result.

## R6 — CI/CD check that blocks merging when a word is undefined

- **Prior art:** existing `scripts/check-changelog-fragment.rs` pattern in this
  repo, plus the typo‑checker `typos` and `codespell` workflows from other repos.
- **Solution:** add `scripts/check-ontology-coverage.rs` (rust‑script). It
  shells into the library via `cargo run --bin meta-ontology -- check-words`
  on the working tree. Workflow step is added with `exit 1` on any unknown
  word (mirrors the changelog check, per the project's CI conventions in
  [docs/case-studies/issue-11/README.md](../issue-11/README.md)).
- **Acceptance:** CI fails when a known unknown word is introduced; CI passes
  on `main`.

## R7 — Store ontology in `links-notation`

- **Prior art:** `links-notation 0.13.0` on crates.io.
- **Solution:** all ontology data lives under `data/**/*.lino`. The loader
  uses `links_notation::parse_lino`.
- **Acceptance:** `cargo test` parses all `data/**/*.lino` without errors.

## R8 — Use `lino-arguments` for the CLI

- **Prior art:** the template already uses it (`src/main.rs`).
- **Solution:** keep `lino_arguments::Parser`. Define subcommands `list`,
  `show`, `check-words`, `serve` (later).
- **Acceptance:** `meta-ontology --help` lists all subcommands; integration
  tests exercise each.

## R9 — Use `lino-objects-codec` for object encoding/decoding

- **Prior art:** `lino-objects-codec 0.2.0` (git, not yet on crates.io).
- **Solution:** add it as an **optional** dependency behind a `codec` Cargo
  feature so `cargo publish` keeps working. Use it in the (later) HTTP
  microservice to serve `text/lino` responses.
- **Acceptance:** `cargo build --features codec` succeeds; default build
  doesn't pull the dependency.

## R10 — Library

- **Solution:** `src/lib.rs` exposes `Ontology`, `Concept`, `load_default()`,
  `load_from_dir(path)`, `find(name)`, `neighbors(name)`. Stable pub API
  documented with `///` doc comments.
- **Acceptance:** library builds standalone, has unit tests, doc tests pass.

## R11 — CLI

- **Solution:** `src/main.rs` provides `meta-ontology <cmd>`. Subcommands as
  in R8.
- **Acceptance:** integration tests assert each subcommand's output.

## R12 — Microservice

- **Prior art:** `axum` + `tokio` recommended in 2026. A read‑only API matches
  the data shape best (the ontology is mostly static).
- **Solution:** add a `meta-ontology-server` crate (or feature‑gated bin) with
  `axum` routes: `GET /concepts`, `GET /concepts/:id`, `GET /neighbors/:id`,
  `GET /search?q=`. JSON by default; `text/lino` via `Accept` header.
- **Acceptance:** `cargo run --bin meta-ontology-server` serves the routes;
  `curl localhost:3000/concepts/thing` returns the concept. **Deferred to M3.**

## R13 — Web app (Rust → WASM + React.js, GitHub Pages)

- **Prior art:** `wasm-pack` + `wasm-bindgen` for Rust→WASM; `vite` +
  `@vitejs/plugin-react` + `vite-plugin-wasm` for the bundler;
  `react-cytoscapejs` or `reactflow` for graph visualisation.
- **Solution:** add a `web/` directory with a Vite/React project that imports a
  `wasm-pack`‑built crate from `crates/meta-ontology-wasm`. Deploy via
  GitHub Pages alongside the existing `cargo doc` deployment.
- **Acceptance:** `cd web && pnpm dev` shows a graph of concepts; production
  build deploys to `https://<org>.github.io/meta-ontology/app/`. **Deferred
  to M4.**

## R14 — Allow multiple alternative correct definitions per concept

- **Solution:** each concept has a list of `definition` links. The loader keeps
  *all* of them; the CLI's `show` command lists them in order. No "primary"
  flag — they are equally valid; ordering is purely insertion order.
- **Acceptance:** at least one seed concept has ≥2 definitions; `show` prints
  them all.

## R15 — Public domain (Unlicense)

- **Solution:** `LICENSE` is already the Unlicense (inherited from template).
  Update `Cargo.toml` to say `license = "Unlicense"` and the README to link to
  `LICENSE`.
- **Acceptance:** `Cargo.toml` license == `Unlicense`; CI doesn't change this.

## R16 — `REQUIREMENTS.md`, `ROADMAP.md`, `docs/plan/`

- **Solution:** all three created in this PR (see top‑level files).
- **Acceptance:** files exist; cross‑linked from README; ROADMAP milestones
  match the deferred slices.

## R17 — Deep case study under `docs/case-studies/issue-1/`

- **Solution:** this folder. README + 3 research files + this analysis.
- **Acceptance:** every requirement has a row in the table above; every
  external claim has a reference.
