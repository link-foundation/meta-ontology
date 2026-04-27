# 02 — Library

Goal: a small, well‑documented Rust library that consumers (CLI, microservice,
WASM module) can use without surprises.

## Layout

```
src/
├── lib.rs            // public re-exports
├── ontology.rs       // Ontology, Concept, Definition types + queries
├── loader.rs         // walk data/, parse .lino, build the graph
├── words.rs          // tokenizer used by the CI check
└── (deprecated)
    └── sum.rs        // demo file from template — to be removed in M1
```

## Types

```rust
/// Distinguished unit of meaning — a node in the ontology graph.
pub struct Concept {
    /// Canonical lowercase identifier, e.g. "thing".
    pub name: String,
    /// Human‑readable label (English by default), e.g. "Thing".
    pub label: String,
    /// Origin tag: "nsm", "lino", "schema", "owl", … — used for provenance.
    pub origin: String,
    /// All definitions, in insertion order. Each is a list of references
    /// pointing to other concepts.
    pub definitions: Vec<Definition>,
    /// Cross‑ontology equivalences (mappings).
    pub mappings: Vec<Mapping>,
}

pub struct Definition {
    pub references: Vec<String>,
    pub language: Option<String>, // ISO 639-1 / -3
}

pub struct Mapping {
    pub external: String,         // e.g. "schema:Thing"
    pub kind: MappingKind,        // Equivalent, BroaderThan, NarrowerThan
}
```

## Loader

- Walks `data/` recursively, picking up `*.lino` files.
- Each file declares concepts via top‑level named links following the
  conventions in [`04-ontology-data.md`](./04-ontology-data.md).
- A second pass resolves references and builds a `petgraph::DiGraph<usize, ()>`.
- Cycles are explicitly allowed; the graph type is `DiGraph` (not a DAG).

## Word scanner

The `words::scan_path` module supports the CI check (R6):

- Walks the repo (skipping `.git/`, `target/`, `node_modules/`).
- For each tracked text file, tokenises the content into "human‑language words":
  - Markdown: skip code blocks/spans, links, images; keep prose.
  - Rust: scan only doc comments (`///`, `//!`).
  - lino: skip identifiers; only scan `(label …)` strings.
- Lowercases, lemmatises (stub for now — strip trivial plural `s`/`es`).
- Returns a vec of `(word, file, line)` triples that are *not* in the
  ontology and *not* in `data/allowlist.lino`.

## Tests

- `tests/unit/ontology.rs` — round‑trip parse + lookup; cycle present.
- `tests/unit/words.rs` — tokenizer fixtures.
- `tests/integration/cli.rs` — end‑to‑end CLI tests.

## Open questions

- **Lemmatisation depth** — for M0 we use a trivial English plural strip; M1
  may swap in a proper English lemmatiser (e.g. `lindera`/`rust-stemmers`)
  guarded by a feature flag.
- **Multilingual word scanning** — initially English‑only. Other languages
  enter the scanner in M2 once exponents are loaded.
