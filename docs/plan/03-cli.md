# 03 — CLI

Goal: a small CLI that consumes the library and gives reviewers a way to
inspect the ontology by hand.

Implementation: `src/main.rs`, parsed via `lino_arguments`.

## Subcommands (M0)

```
meta-ontology list                  # print all concept names, alphabetically
meta-ontology show <name>           # print a concept's label, definitions, mappings
meta-ontology check-words <path>    # exit non-zero if any word is unknown
```

### Examples

```text
$ meta-ontology list | head
all
because
before
big
body
can

$ meta-ontology show thing
name:    thing
label:   Thing
origin:  lino
definitions:
  - (thing is a (kind of concept))
  - (thing means something or someone)
mappings:
  - schema:Thing  (≈)
  - owl:Thing     (≈)
  - wikidata:Q35120 (≈)

$ meta-ontology check-words .
4 unknown words found:
  - "elucidate"   in CONTRIBUTING.md:32
  - "reify"       in docs/plan/04-ontology-data.md:14
  - "polysemy"    in docs/case-studies/issue-1/research-nsm.md:8
  - "explication" in docs/case-studies/issue-1/research-nsm.md:104
exit 1
```

## Subcommands (M1+)

| Cmd | Milestone | Purpose |
|-----|-----------|---------|
| `dot` | M1 | Emit Graphviz DOT |
| `graph --json` | M1 | Cytoscape‑compatible JSON for the web app |
| `langs` | M2 | List language coverage |
| `mappings <name>` | M2 | List cross‑ontology equivalences |
| `serve` | M3 | Start the HTTP microservice |
| `find <substring>` | M1 | Substring search |

## Argument parsing

Use `lino_arguments::Parser` (re‑exports clap). Subcommands follow the
`#[derive(Subcommand)]` pattern:

```rust
use lino_arguments::Parser;
use clap::Subcommand;

#[derive(Parser)]
#[command(name = "meta-ontology", about = "Meta-ontology library + CLI")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    List,
    Show { name: String },
    CheckWords { path: PathBuf },
}
```

## Exit codes

- `0` — success
- `1` — known error (unknown words, missing concept, parse error)
- `2` — usage error (e.g. unknown subcommand) — clap default

## Where to put integration tests

`tests/integration/cli.rs` — uses `assert_cmd` and `predicates`.
