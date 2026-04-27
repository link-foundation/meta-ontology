# 04 — Ontology data files

Goal: pin down how `.lino` files under `data/` are organised, so contributors
and the loader agree.

## Folder layout

```
data/
├── primes/
│   ├── nsm.lino                   # the 65 NSM primes (English labels)
│   └── exponents/
│       ├── en.lino                # English exponents (canonical)
│       ├── es.lino                # Spanish
│       ├── ru.lino                # Russian
│       └── ...
├── concepts/
│   ├── core.lino                  # lino self-referential primes
│   └── mappings.lino              # cross-ontology equivalences
├── external/
│   ├── schema-org.lino            # top-level Schema.org primitives
│   ├── owl.lino                   # OWL/RDF built-ins
│   ├── sumo.lino                  # SUMO upper concepts
│   └── wikidata.lino              # Wikidata top items
└── allowlist.lino                 # words that never need a concept
```

## File grammar

Each file is plain Links Notation. The lino grammar has only two node types
— `Ref(name)` and `Link { id?, values }` — and **no native string or comment
support**. Concept files therefore use the *first* reference inside a link as
the concept name, and use nested links of the form `(key value...)` to attach
properties. Multi‑word labels are written as parenthesised sub‑links.

### Concept declaration

```
(thing
  (label Thing)
  (origin lino)
  (definition thing is concept)
  (definition thing means something or someone))
```

- `thing` (the first ref in the outer link) is the canonical identifier and
  **must be lowercase ASCII**, optionally with `-` or `_` separators.
- `(label Thing)` — single human‑readable label in the file's primary
  language. Multi‑word labels are written `(label (the universal type))`.
- `(origin lino)` — one of `lino`, `nsm`, `schema`, `owl`, `wikidata`, `sumo`,
  `external`. Used for provenance.
- Any number of `(definition …)` entries; each is a sub‑link describing the
  concept in terms of other concept names. Definitions may be nested arbitrarily.

### Mapping declaration

```
(mapping thing ~ schema_Thing)
(mapping thing ~ owl_Thing)
(mapping thing ~ wikidata_Q35120)
```

The relation symbol `~` means *equivalent*; `<` means *narrower than*; `>`
means *broader than* (we use ASCII because lino identifiers must be ASCII).
External names use `_` as the namespace separator (`schema_Thing`,
`owl_Thing`, `wikidata_Q35120`, `sumo_Entity`).

### Exponent declaration

```
(exponent thing ru veshch (coverage full))
(exponent thing zh dongxi (coverage full))
```

`(coverage full)` or `(coverage provisional)`. Provisional entries are
allowed but surfaced by `meta-ontology langs` as gaps. The exponent itself is
written as a lino identifier; non‑ASCII forms (Cyrillic, CJK) are stored
romanised, with the original form attached as
`(native (in_script <string-of-refs>))` if needed.

### Allow‑list entry

```
(allowlist github (reason proper noun GitHub))
```

Allow‑list entries **must** include a `(reason …)` — this keeps the list
honest.

## Validation rules (loader)

1. Concept names are unique across all files.
2. Each `definition:` references existing concepts (the loader validates this
   in a second pass; cycles are *fine*).
3. Each `mapping` is symmetric in storage but `≈` is the only commutative
   relation.
4. Exponent files declare their own language with `(language <iso639>)` at the
   top.
5. `allowlist.lino` entries that are also concepts are rejected (no
   double‑listing).

## Versioning data files

- A breaking change to the file grammar bumps the **library** major version.
- Adding new concepts or exponents is a minor change.
- Removing a concept needs a deprecation aliasing entry: `(deprecated foo →
  bar)`.

## Authoring workflow

1. Open the relevant file.
2. Add the concept / mapping / exponent following the grammar above.
3. Run `cargo run -- check-words .` and `cargo test`.
4. Commit; add a changelog fragment if it's a user‑facing change.
