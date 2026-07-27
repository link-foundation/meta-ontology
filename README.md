# meta-ontology

A self-describing meta-ontology stored in [Links Notation](https://github.com/link-foundation/links-notation),
shipping as a Rust library, command-line tool, microservice, and a
WebAssembly + React.js web application — public domain.

[![License: Unlicense](https://img.shields.io/badge/license-Unlicense-blue.svg)](http://unlicense.org/)

## Status

First prototype (MVP). See [issue #1](https://github.com/link-foundation/meta-ontology/issues/1)
and the milestones in [`ROADMAP.md`](ROADMAP.md). What is shipped today:

- Library + CLI that load the seed ontology from `data/`.
- 65 NSM semantic primes + ~25 self-referential link-notation primes.
- English exponents complete; Spanish and Russian seeded.
- Cross-ontology mappings to Schema.org, OWL/RDF, SUMO, Wikidata.
- Word-coverage check that reports words missing from the ontology.
- A versioned catalog contract with stable IDs, typed relationships,
  provenance, governance metadata, and stable validation diagnostics.
- A verified `meta-language` links network and immutable snapshots.
- Generated JSON Schema, validated JSON round trips, deterministic import
  plans, and ranked in-memory search.
- A fixture-backed ingestion contract with redacted secrets, checkpoints,
  partial-failure quarantine, dry runs, and idempotent replay.

The HTTP microservice and the WASM/React web app are scaffolded only;
their full implementation is staged in `ROADMAP.md` (M3 and M4).

## Design

A meta-ontology is an ontology that describes itself. The seed graph
contains concepts like `link`, `thing`, `concept`, and `definition` that
reference each other — for example, `thing` is defined via `concept`,
and `concept` is defined via `thing`. This is a network with cycles, not
a tree, which is why we store it in Links Notation.

For the deep analysis see
[`docs/case-studies/issue-1/`](docs/case-studies/issue-1/) and the
implementation plan in [`docs/plan/`](docs/plan/).

## Quick start

```bash
# Build
cargo build

# List every concept
cargo run -- list

# Show one concept and its definitions / mappings
cargo run -- show thing

# Walk the example
cargo run --example basic_usage

# Check that every word in a path is in the ontology or allow-list
cargo run -- check-words README.md

# Show declared languages
cargo run -- langs

# Validate all catalog layers
cargo run -- validate

# Export the catalog contract or its generated JSON Schema
cargo run -- export-json
cargo run -- json-schema

# Preview a full-replacement import without applying it
cargo run -- plan-import candidate.json --format ndjson

# Search stable IDs, names, labels, aliases, and definitions
cargo run -- search "body or not a body"

# Run the fixture-backed ingestion example
cargo run --example openmetadata_fixture_ingestion
```

## Crates we depend on

This project follows the issue's directive to use the `link-foundation`
toolchain:

- [`links-notation`](https://github.com/link-foundation/links-notation)
  — parser and serializer for the lino format used by every `data/*.lino`
  file.
- [`lino-arguments`](https://github.com/link-foundation/lino-arguments)
  — clap-style argument parser with `.lenv` config support, used by the
  CLI binary.
- [`meta-language`](https://github.com/link-foundation/meta-language)
  — canonical links-network model, full-match verification, and immutable
  snapshots shared with other Link Foundation consumers.
- [`lino-objects-codec`](https://github.com/link-foundation/lino-objects-codec)
  — feature-gated dependency for runtime object encoding (used by the
  microservice in M3 and the web app in M4).

## Repository layout

```
.
├── data/
│   ├── primes/            # NSM semantic primes + per-language exponents
│   ├── concepts/          # core lino concepts and cross-ontology mappings
│   └── allowlist.lino     # words that intentionally have no concept
├── src/
│   ├── lib.rs             # library entry point
│   ├── catalog.rs         # identity, validation, interchange, plans, search
│   ├── ingestion.rs       # connector, checkpoint, normalization contracts
│   ├── ontology.rs        # Ontology, Concept, Definition, Mapping types
│   ├── loader.rs          # walks data/, parses lino, builds the graph
│   ├── words.rs           # tokenizer + scanner used by check-words
│   └── main.rs            # CLI binary
├── docs/
│   ├── case-studies/      # deep research per issue
│   └── plan/              # per-slice implementation plan
├── examples/              # library and fixture-ingestion examples
├── tests/
│   ├── unit/              # unit tests
│   └── integration/       # CLI integration tests
├── REQUIREMENTS.md        # canonical requirement list
├── ROADMAP.md             # milestones from MVP through 1.0
└── Cargo.toml
```

## Documentation

- [`REQUIREMENTS.md`](REQUIREMENTS.md) — canonical requirement list
- [`ROADMAP.md`](ROADMAP.md) — staged milestones
- [`docs/plan/`](docs/plan/) — per-slice implementation plan
- [`docs/case-studies/issue-1/`](docs/case-studies/issue-1/) — deep analysis
- [`docs/case-studies/issue-5/`](docs/case-studies/issue-5/) — OpenMetadata
  practice analysis and delivered implementation map

## License

Public domain — released under [the Unlicense](LICENSE).

## Contributing

See [`CONTRIBUTING.md`](CONTRIBUTING.md). Pull requests welcome.
