# Requirements

This file is the canonical, numbered list of requirements for **meta‑ontology**,
extracted from [issue #1](https://github.com/link-foundation/meta-ontology/issues/1).
Per‑requirement proposed solutions and acceptance criteria are in
[docs/case-studies/issue-1/requirements-and-solutions.md](docs/case-studies/issue-1/requirements-and-solutions.md).

The notation `MUST` / `SHOULD` / `MAY` follows [RFC 2119](https://www.rfc-editor.org/rfc/rfc2119).

## Data

### R1 — Survey of popular world ontologies
The repository **MUST** contain a written survey of the most popular existing
ontologies (Schema.org, OWL/RDF, WordNet, ConceptNet, BabelNet, Cyc, SUMO,
DBpedia, Wikidata, FrameNet at minimum), recording each ontology's top‑level
"primitive" concepts.

### R2 — NSM semantic primes coverage
The meta‑ontology **MUST** include all 65 [NSM semantic primes](https://en.wikipedia.org/wiki/Natural_semantic_metalanguage)
(Goddard & Wierzbicka chart v19, 2017), and **SHOULD** include exponents
(translations) of each prime in the top‑20 most‑spoken human languages.
Languages without an authoritative NSM exponent table **MUST** be marked
`coverage: provisional` rather than left blank.

### R3 — Cyclic / "prime" concepts from world ontologies
The meta‑ontology **MUST** include the concepts that are cyclic or
self‑referential in popular ontologies — at minimum: `thing`, `class`,
`property`, `relation`, `concept`, `set`, `entity`, `link`, `reference`,
`name`. Each of these concepts **MUST** have at least one cross‑ontology
mapping (e.g. `thing ≈ schema:Thing ≈ owl:Thing ≈ wikidata:Q35120`).

## Structure

### R4 — Network, not acyclic tree
The meta‑ontology **MUST** be a graph (cycles allowed). The loader **MUST NOT**
reject cyclic data. At least one cycle **MUST** exist in the seed data
(`thing → concept → thing`).

### R5 — Self‑describing
Every word used in the repository **MUST** be either (a) defined as a concept
in the ontology, or (b) listed in `data/allowlist.lino` as an explicit
exception (proper nouns, identifiers, version strings, code keywords). The
allow‑list **MUST** be human‑readable lino with reasons.

### R14 — Multiple alternative correct definitions
Each concept **MAY** have any number of `definition` entries. All definitions
are equally valid; the loader **MUST** preserve them in insertion order. There
is no "primary" definition flag.

## Tooling and CI

### R6 — Word‑coverage CI check
The CI pipeline **MUST** include a step that fails the build (and blocks PR
merging) when a human‑language word in the repository's tracked text files is
not present in the ontology and not on the allow‑list. This step **MUST**
print a list of unknown words with file:line locations.

### R7 — Storage in Links Notation
All ontology data **MUST** be stored as `.lino` files using
[`links-notation`](https://github.com/link-foundation/links-notation). No
parallel JSON/YAML/RDF source‑of‑truth.

### R8 — CLI uses `lino-arguments`
The command‑line interface **MUST** use
[`lino-arguments`](https://github.com/link-foundation/lino-arguments) for
argument parsing.

### R9 — Object encoding via `lino-objects-codec`
Object serialisation between layers (CLI ↔ microservice ↔ web) **SHOULD** use
[`lino-objects-codec`](https://github.com/link-foundation/lino-objects-codec).
Until that crate is published to crates.io, it **MAY** be an optional
(feature‑gated) dependency.

## Distribution

### R10 — Rust library
The repository **MUST** ship a Rust library crate exposing types `Ontology`
and `Concept` and APIs `load_default()`, `load_from_dir(path)`, `find(name)`,
`neighbors(name)`.

### R11 — CLI tool
The repository **MUST** ship a `meta-ontology` CLI binary with subcommands
`list`, `show <name>`, `check-words <path>`, and (eventually) `serve`.

### R12 — Microservice
The repository **MUST** ship a microservice exposing the ontology over HTTP.
Recommended endpoints: `GET /concepts`, `GET /concepts/:id`,
`GET /neighbors/:id`, `GET /search?q=`. JSON by default; `text/lino` via
`Accept` header.

### R13 — Web application (WASM + React.js, GitHub Pages)
The repository **MUST** ship a Web application that uses the same Rust core
compiled to WebAssembly, with a React.js front‑end, deployed to GitHub Pages.
The UI **MUST** show concepts and their relationships, including cycles.

## Licensing

### R15 — Public domain
The project **MUST** be released into the public domain via the Unlicense
(already in `LICENSE`). `Cargo.toml` **MUST** say `license = "Unlicense"`.

## Documentation

### R16 — Top‑level docs
The repository **MUST** contain `REQUIREMENTS.md` (this file), `ROADMAP.md`,
and a `docs/plan/` folder with detailed per‑slice implementation instructions.

### R17 — Case study
A deep case‑study analysis of this issue **MUST** live under
`docs/case-studies/issue-1/`, including online research notes (NSM, popular
ontologies, link‑foundation crates) and proposed solutions per requirement.

## Traceability

Each requirement is referenced from at least one of:

- the case study (`docs/case-studies/issue-1/requirements-and-solutions.md`),
- the roadmap (`ROADMAP.md`),
- the implementation plan (`docs/plan/`),
- the code (commit messages and tests).
