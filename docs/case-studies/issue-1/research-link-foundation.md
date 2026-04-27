# link-foundation Crates — Research Notes

Reference notes for the three crates the issue mandates we use.

## 1. links-notation (lino)

- **Repo:** <https://github.com/link-foundation/links-notation>
- **Crate:** `links-notation = "0.13.0"` on crates.io (2025‑12‑01)
- **What it is:** reference parser/serializer for **Links Notation** — a minimal
  text format whose data model is a list of links over named references. Because
  any reference can name a link that other links also reference, the resulting
  structure is a **graph** (cycles allowed), not just a tree.

### Syntax cheat‑sheet

| Construct | Example | Meaning |
|-----------|---------|---------|
| Reference | `papa` | A bare identifier |
| Link (parens) | `(loves mama)` | A 2‑tuple link `loves`→`mama` |
| Top‑level link | `papa loves mama` | Same as `(papa loves mama)` |
| Named link | `(lovesMama: loves mama)` | Defines `lovesMama` as a reusable id |
| Reuse | `son lovesMama` | Reuses the named link → produces a graph |
| N‑arity | `(papa has car)` | Triplet — any arity is fine |
| Grouping | `(papa and mama) are happy` | Sub‑link grouped as a single ref |
| Indented form | `3:\n  papa\n  loves\n  mama` | Same as `(3: papa loves mama)` |

### Rust API (sketch)

```rust
use links_notation::parse_lino;

let links = parse_lino("papa (lovesMama: loves mama)\nson lovesMama")?;
for link in &links {
    println!("{:?}", link);
}
```

`parse_lino` returns `Result<Vec<Link>, _>` where `Link` is a tree of
`Reference`/`Link` nodes. There is also a serializer to render `Vec<Link>` back
to text. Tuple conversions via `From` make idiomatic Rust use possible.

### Why it fits the meta‑ontology

The issue explicitly says the meta‑ontology must allow cyclic self‑definitions
between primes (`link` ⇄ `thing` ⇄ `concept`). Links Notation supports this by
construction, with no special "cycle escape" syntax. Each concept becomes a
named link; references between concepts become further links.

## 2. lino-arguments

- **Repo:** <https://github.com/link-foundation/lino-arguments>
- **Crate:** `lino-arguments = "0.3.0"` on crates.io (2026‑04‑10)
- **What it is:** unified config layer that wraps **clap** and adds env‑var
  loading from `.lenv` (Links Notation env file) and `.env`. Priority chain:
  CLI args → process env → `.lenv` → `.env` → defaults.

### Re‑exports / API

- Re‑exports clap: `Parser`, `Args`, `Subcommand`, `ValueEnum`, `arg!`,
  `command!`.
- `init()` / `init_with(lenv, env)` — load files into env before `Args::parse()`.
- `LinoParser` trait → `Args::lino_parse()` (one‑liner replacement for
  `Args::parse()`).
- Functional builder: `make_config(|c| c.lenv(...).option(...).flag(...))`.
- Helpers: `getenv`, `getenv_int`, `getenv_bool` (case‑insensitive); case
  converters (`to_snake_case`, `to_camel_case`, …).

### `.lenv` syntax

```
PORT: 8080
LOG_LEVEL: info
```

### Drop‑in replacement for clap

```rust
use lino_arguments::Parser;

#[derive(Parser, Debug)]
#[command(name = "meta-ontology", about = "Meta-ontology CLI")]
struct Args { /* ... */ }

fn main() {
    let args = Args::parse();
    // ...
}
```

## 3. lino-objects-codec

- **Repo:** <https://github.com/link-foundation/lino-objects-codec>
- **Crate:** `0.2.0` in repo's `Cargo.toml`; **not yet published** to crates.io
  (use a git dependency for now).
- **What it is:** universal serializer/deserializer that encodes arbitrary
  object graphs to/from Links Notation. Parallel implementations in Rust, JS,
  Python, C#.

### Wire format examples

```
(int 42)
(bool true)
(str aGVsbG8=)            ; strings are base64 to escape specials
(float NaN)
(array (int 1) (int 2))
(object ((str a2V5) (int 42)) ...)
(obj_0: array (int 1) (int 2))   ; named back-reference for shared/cyclic data
```

### Rust API

```rust
use lino_objects_codec::{encode, decode, LinoValue};

let v = LinoValue::object([("answer", LinoValue::Int(42))]);
let s = encode(&v);
let v2 = decode(&s)?;
assert_eq!(v, v2);
```

`LinoValue` is the universal type:

```rust
enum LinoValue {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Array(Vec<LinoValue>),
    Object(Vec<(String, LinoValue)>),
}
```

### Cargo dependency (until crates.io publish)

```toml
[dependencies]
lino-objects-codec = { git = "https://github.com/link-foundation/lino-objects-codec", branch = "main" }
```

Because the crate is not on crates.io yet, **the MVP shipped in PR #2 keeps
the codec optional** — features that would require it (binary serialisation,
WASM/JSON ↔ lino round‑trip) are gated behind a `codec` cargo feature, default
off. This keeps `cargo publish` working and avoids blocking on the upstream
publish.

## Stack summary

The three crates layer cleanly:

```
lino-arguments      <-- CLI / config (depends on clap)
        |
lino-objects-codec  <-- typed value layer with cycle support
        |
links-notation      <-- syntax / parser foundation
```

A Rust project can adopt them independently or together — exactly what we do
in the meta‑ontology.
