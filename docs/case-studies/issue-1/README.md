# Case Study: Issue #1 — First prototype (MVP)

## Summary

This case study analyses [issue #1](https://github.com/link-foundation/meta-ontology/issues/1) — the founding issue of the `meta-ontology` repository — and turns it into actionable
requirements, a roadmap, and a first‑working‑session prototype.

The issue asks for a **meta‑ontology**: a small, self‑describing, network‑shaped
ontology written in [Links Notation (lino)](https://github.com/link-foundation/links-notation),
distributed as a Rust **library**, **CLI**, **microservice** and **WebAssembly + React.js
GitHub Pages app**, dedicated to the public domain.

A meta‑ontology differs from a normal ontology in two ways:

1. It is **self‑describing**: every word used inside the repository must itself be a
   defined concept in the ontology, enforced in CI/CD.
2. It is **a graph, not a tree**: prime concepts are allowed to define each other in
   cycles (`thing` ↔ `concept` ↔ `link`), exactly the kind of structure
   Links Notation makes natural.

## Issue text (canonical, verbatim)

> We need to collect data about most popular ontologies from the internet.
>
> We also need to make sure our meta ontology supports all semantic primes from
> https://en.wikipedia.org/wiki/Natural_semantic_metalanguage in top 20 most popular
> human languages.
>
> We need to ensure that all prime concepts from all worlds ontologies (the concepts
> that use them selves in a loop or just defined only as themselves) are present in
> our meta ontology.
>
> Our ontology can be network, not plain acyclic tree. So we can define everything
> through concepts like `link`, `thing`, `concept`, `object`, `term` and so on. Only
> those that are really possible to describe each other. So that should be meta
> ontology network, not flat plain tree like in other places, that is why we use
> links notation that allows that.
>
> Also our meta ontology should have enough concepts to describe fully itself,
> meaning each word that is used in this repository should be defined in it, and we
> should have CI/CD check, that guarantees that every human language word is
> described in the ontology, and if not it should block Pull Request from merging,
> to prevent it as a bug.
>
> We should use these:
> - https://github.com/link-foundation/links-notation (to store meta ontology data,
>   definitions and so on)
> - https://github.com/link-foundation/lino-arguments
> - https://github.com/link-foundation/lino-objects-codec (for objects
>   encoding/decoding)
>
> We need to have our product as library, CLI tool, microservice and Web application
> (GitHub Pages) that is using Rust in WebAssembly and React.js. So we can have web
> interface to see each concepts and how they related to each other, and how they
> defined via each other. It is ok to have many alternative (but correct definitions).
>
> This project is dedicated to public domain.
>
> We also need to have REQUIREMENTS.md, ROADMAP.md and plan folder with all detailed
> instructions on how to implement it in docs folder. If everything will not fit to
> the first working session, yet please try to make as much as possible from the
> first go.
>
> We need to collect data related about the issue to this repository, make sure we
> compile that data to `./docs/case-studies/issue-{id}` folder, and use it to do
> deep case study analysis (also make sure to search online for additional facts and
> data), list of each and all requirements from the issue, and propose possible
> solutions and solution plans for each requirement (we should also check known
> existing components/libraries, that solve similar problem or can help in
> solutions).

## Extracted requirements (numbered)

See [REQUIREMENTS.md](../../../REQUIREMENTS.md) for the canonical list. Short index:

| ID  | Requirement | Type |
|-----|-------------|------|
| R1  | Collect data about the most popular world ontologies | data |
| R2  | Cover all NSM semantic primes (~65) in top‑20 languages | data |
| R3  | Include cyclic / self‑referencing prime concepts (`link`, `thing`, …) | data |
| R4  | Be a network (graph), not an acyclic tree | structure |
| R5  | Be self‑describing: every word in the repo is a concept | governance |
| R6  | CI/CD check that blocks merging when a word is undefined | tooling |
| R7  | Store the ontology in **links‑notation** (`.lino` files) | tech stack |
| R8  | Use **lino‑arguments** for the CLI | tech stack |
| R9  | Use **lino‑objects‑codec** for object encoding/decoding | tech stack |
| R10 | Ship as a Rust **library** | distribution |
| R11 | Ship as a **CLI tool** | distribution |
| R12 | Ship as a **microservice** | distribution |
| R13 | Ship as a **Web app** (Rust → WebAssembly + React.js, GitHub Pages) | distribution |
| R14 | Allow multiple alternative correct definitions per concept | semantics |
| R15 | Public domain (Unlicense) | licensing |
| R16 | Provide `REQUIREMENTS.md`, `ROADMAP.md`, and `docs/plan/` | documentation |
| R17 | Provide a deep case‑study analysis under `docs/case-studies/issue-1/` | documentation |

Each requirement has a proposed solution, prior art, and acceptance criteria —
see [`requirements-and-solutions.md`](./requirements-and-solutions.md).

## Online research findings

### Natural Semantic Metalanguage (NSM)

- The current canonical list contains **65 semantic primes** in **16 categories**
  (Goddard & Wierzbicka chart v19, 2017). The full list is reproduced verbatim in
  [`research-nsm.md`](./research-nsm.md) and is the source of `data/primes/nsm.lino`.
- Translations of primes are called **exponents**. Empirical exponent tables exist
  for **30+ languages across 16+ language families**; covering ~13 of the top‑20
  most‑spoken languages (English, Mandarin, Spanish, Russian, Japanese, German,
  Korean, French, Vietnamese, Italian, Polish, Persian, Arabic). Hindi, Bengali,
  Turkish, Ukrainian, Punjabi, and Indonesian still lack a published, peer‑reviewed
  full table — they are tracked as gaps (see ROADMAP).
- Primary public catalogue: <https://nsm-approach.net>.
- Reference chart: Griffith University NSM chart v19 (April 2017).

### Popular world ontologies

| Ontology | Top‑level "primitive" layer | NSM analogue? |
|----------|----------------------------|---------------|
| **Schema.org** | Single root `Thing`, ~12 immediate subclasses (CreativeWork, Event, Person, Place, Product, …) | No — markup vocabulary |
| **OWL / RDF(S)** | `owl:Thing`, `owl:Nothing`, `rdf:Property`, `rdfs:Class`, logical constructors | No — formal logic primitives |
| **WordNet** | ~25 "unique beginners" (lexicographer files) | Loosely — not claimed universal |
| **ConceptNet** | ~36 typed relations over an open multilingual concept graph | Closest analogue at the *relation* level |
| **BabelNet** | No primitives; integrates WordNet + Wikipedia/Wikidata | No |
| **Cyc / OpenCyc / ResearchCyc** | Thousands of upper‑level constants | Partial — much larger scale |
| **SUMO / MILO** | ~1 000 top‑level concepts; IEEE P1600.1; mapped to WordNet | Partial — closer in spirit |
| **DBpedia** | ~700 classes auto‑derived from Wikipedia infoboxes | No |
| **Wikidata** | Open `instance of` / `subclass of` graph | No fixed primitives |
| **FrameNet** | ~1 200 frames + ~10 000 frame elements (Fillmore's frame semantics) | Partial — frames as recurring meanings |

**Take‑away.** No mainstream ontology commits to a closed, universal, lexical
"prime" vocabulary. NSM does — and Links Notation fits it well, because it can
express the cyclic self‑references that primes need (`THING ⇄ SOMETHING`,
`KIND ⇄ TYPE`, …).

The proposed hybrid for the meta‑ontology:

1. **Primes layer** — NSM 65 + the lino self‑referential primes (`link`, `thing`,
   `concept`, …).
2. **Upper layer** — borrowed from SUMO/Wikidata for formal reasoning.
3. **Cross‑ontology mapping** — store equivalences (`schema:Thing ≈ thing`,
   `owl:Thing ≈ thing`) as additional links, not as inheritance, so the structure
   stays a network.

### Tooling — link-foundation crates

| Crate | Latest | Role in this project |
|-------|--------|----------------------|
| [`links-notation`](https://github.com/link-foundation/links-notation) | `0.13.0` (crates.io) | Parse / serialize `.lino` files |
| [`lino-arguments`](https://github.com/link-foundation/lino-arguments) | `0.3.0` (crates.io) | CLI arg + `.lenv` config |
| [`lino-objects-codec`](https://github.com/link-foundation/lino-objects-codec) | `0.2.0` (git, not yet on crates.io) | Encode/decode runtime objects to lino, preserves cycles |

For full API / syntax notes see [`research-link-foundation.md`](./research-link-foundation.md).

### Web / WASM stack

- **Rust → WASM** via [`wasm-pack`](https://rustwasm.github.io/wasm-pack/) with
  `wasm-bindgen` for the JS bridge. Recommended bundler in 2026: `vite` +
  [`@vitejs/plugin-react`](https://www.npmjs.com/package/@vitejs/plugin-react) +
  [`vite-plugin-wasm`](https://www.npmjs.com/package/vite-plugin-wasm).
- **React** for the UI. Graph visualisation via
  [`react-cytoscapejs`](https://www.npmjs.com/package/react-cytoscapejs) or
  [`reactflow`](https://reactflow.dev) — both render large directed graphs and
  handle cycles.
- **GitHub Pages** for hosting the static bundle (Rust docs already deploy there;
  the SPA can live under `/app/`).

### Microservice stack

- [`axum`](https://docs.rs/axum) (Tokio‑based, minimal) — recommended over
  Actix‑Web for a small public read‑only API.
- Endpoints sketch: `GET /concepts`, `GET /concepts/:id`, `GET /concepts/:id/neighbors`,
  `GET /search?q=`. JSON responses; lino round‑trip optional via `Accept: text/lino`.

## Proposed solutions per requirement

See [`requirements-and-solutions.md`](./requirements-and-solutions.md). Each entry
includes: the requirement, prior art reviewed, the chosen approach, and acceptance
criteria.

## What this PR (first working session) ships

The issue explicitly says: *"If everything will not fit to the first working
session, yet please try to make as much as possible from the first go."*

The first prototype scope is intentionally **small but end‑to‑end**:

| Slice | Status | Notes |
|-------|--------|-------|
| Repository renamed from template (`Cargo.toml`, imports) | ✅ | Package name `meta-ontology` |
| Core data: NSM 65 primes in lino | ✅ | `data/primes/nsm.lino` |
| Core data: lino self‑referential primes | ✅ | `data/concepts/core.lino` |
| Library: parse + index ontology, lookup by name | ✅ | `src/ontology.rs`, `src/loader.rs` |
| CLI: `list`, `show <name>`, `check-words <path>` | ✅ | `src/main.rs` via `lino-arguments` |
| CI: undefined‑word check (R6) | ✅ | `scripts/check-ontology-coverage.rs` + workflow step |
| Microservice (R12) | ⏳ deferred | scaffold only; full impl in roadmap M3 |
| WASM + React app (R13) | ⏳ deferred | scaffold only; full impl in roadmap M4 |
| Translations for 20 languages | ⏳ partial | English + minimal stubs; expansion in M2 |
| Cross‑ontology mapping | ⏳ deferred | format documented; full data in M2 |

The deferred slices are documented in `ROADMAP.md` and `docs/plan/`.

## Files in this case study

- [`README.md`](./README.md) — this overview
- [`requirements-and-solutions.md`](./requirements-and-solutions.md) — per‑requirement
  proposed solutions, prior art, and acceptance criteria
- [`research-nsm.md`](./research-nsm.md) — full NSM prime list and language coverage
- [`research-link-foundation.md`](./research-link-foundation.md) — `links-notation`,
  `lino-arguments`, `lino-objects-codec` API/syntax notes
- [`research-popular-ontologies.md`](./research-popular-ontologies.md) — comparison
  of Schema.org, OWL/RDF, WordNet, ConceptNet, BabelNet, Cyc, SUMO, DBpedia,
  Wikidata, FrameNet

## Key takeaways

1. **NSM is the right kernel.** No other ontology gives you a closed,
   lexically‑grounded, universal primitive vocabulary. Build outward from the 65
   primes.
2. **Links Notation lets the kernel be a graph.** Cyclic definitions
   (`thing` ⇄ `concept` ⇄ `link`) are first‑class — exactly what a meta‑ontology
   needs and what RDF/Schema.org awkwardly avoid.
3. **The self‑describing CI check is the differentiator.** It is what makes this
   project a *meta*‑ontology rather than just another ontology. Ship it from day
   one, even if the initial vocabulary is small.
4. **Ship the library first.** CLI, microservice, and WASM/React app all consume
   the same library; the order of work in `ROADMAP.md` reflects this.

## References

- [Natural Semantic Metalanguage — Wikipedia](https://en.wikipedia.org/wiki/Natural_semantic_metalanguage)
- [NSM Approach — Resources](https://nsm-approach.net)
- [Griffith NSM chart v19 (PDF)](https://intranet.secure.griffith.edu.au/__data/assets/pdf_file/0019/346033/NSM_Chart_ENGLISH_v19_April_12_2017_Greyscale.pdf)
- [Schema.org full hierarchy](https://schema.org/docs/full.html)
- [SUMO / MILO ontology](https://www.ontologyportal.org/)
- [WordNet 3.1](https://wordnet.princeton.edu/)
- [ConceptNet 5](https://conceptnet.io)
- [BabelNet](https://babelnet.org)
- [Wikidata top‑level entity Q35120](https://www.wikidata.org/wiki/Q35120)
- [Cyc / OpenCyc](https://www.cyc.com/)
- [FrameNet](https://framenet.icsi.berkeley.edu/)
- [Links Notation](https://github.com/link-foundation/links-notation)
- [lino-arguments](https://github.com/link-foundation/lino-arguments)
- [lino-objects-codec](https://github.com/link-foundation/lino-objects-codec)
