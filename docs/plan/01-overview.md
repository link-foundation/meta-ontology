# 01 — Architecture overview

```
                     ┌─────────────────────────────────────┐
                     │   data/**/*.lino                    │
                     │  - data/primes/nsm.lino             │
                     │  - data/primes/exponents/<lang>.lino│
                     │  - data/concepts/core.lino          │
                     │  - data/concepts/mappings.lino      │
                     │  - data/allowlist.lino              │
                     └──────────────┬──────────────────────┘
                                    │
                       links_notation::parse_lino
                                    │
                                    ▼
                  ┌──────────────────────────────┐
                  │   src/ (library crate)       │
                  │  - Ontology, Concept types   │
                  │  - load_default()            │
                  │  - load_from_dir(path)       │
                  │  - find(name) / neighbors    │
                  │  - words::scan_path()        │
                  └─────┬──────────────┬─────────┘
                        │              │
            ┌───────────▼──┐     ┌─────▼───────────┐
            │ CLI bin      │     │ WASM crate      │
            │ src/main.rs  │     │ crates/         │
            │              │     │  meta-ontology- │
            │              │     │  wasm/          │
            └───┬──────────┘     └─────┬───────────┘
                │                      │
                │                      ▼
                │              ┌──────────────────┐
                │              │  web/  (React)   │
                │              │  GitHub Pages    │
                │              └──────────────────┘
                │
                ▼
        ┌─────────────────┐
        │ HTTP server     │
        │ (axum, M3)      │
        └─────────────────┘
```

## Crates and binaries

- `meta-ontology` (this crate) — library + CLI binary.
- `meta-ontology-server` (M3) — feature‑gated HTTP server, or a sub‑crate.
- `meta-ontology-wasm` (M4) — `wasm-bindgen` re‑exports for the web app.

## Why a single library crate?

Three reasons:
1. The CLI, the microservice, and the WASM module all need exactly the same
   loading + querying API. Keeping it in one library prevents drift.
2. The data set is small enough (≪ 1 MiB at full size) to keep entirely in
   memory.
3. Deduplication: `petgraph` integration, tokenisation, and lino I/O happen in
   one place; consumers add only what they need.

## Public Rust API (M0 baseline)

```rust
pub struct Ontology { /* opaque */ }
pub struct Concept  { /* opaque */ }

impl Ontology {
    pub fn load_default() -> Result<Self>;
    pub fn load_from_dir(path: impl AsRef<Path>) -> Result<Self>;
    pub fn find(&self, name: &str) -> Option<&Concept>;
    pub fn names(&self) -> impl Iterator<Item = &str>;
    pub fn neighbors(&self, name: &str) -> impl Iterator<Item = &Concept>;
    pub fn definitions(&self, name: &str) -> impl Iterator<Item = &Definition>;
}
```

Stability rule: nothing in the public API may change incompatibly without a
major‑version bump after `1.0.0`. Until then, breaking changes are allowed
between minor versions and **must** include a changelog fragment.

## Data flow

1. **Load**: walk the `data/` folder; parse every `*.lino` file with
   `links-notation`; collect concepts and their relations.
2. **Index**: build a name → concept hashmap; build a `petgraph::DiGraph` of
   relations (allowing cycles).
3. **Query**: lookups are O(1) on the hashmap; neighbour traversal is
   `petgraph` BFS/DFS as needed.
4. **Serialise out** (M3): convert `Concept` → `LinoValue` →
   `lino_objects_codec::encode` for `text/lino`; or `serde_json` for JSON.

## Error handling

- Public functions return `anyhow::Result` (or a thin custom error enum once
  the library is stable). Loader errors include file path + line for
  developer ergonomics.
- The CLI prints errors with `eprintln!` and exits non‑zero. CI consumes the
  exit code only.
