# 07 — CI word‑coverage check (R6)

Goal: fail any pull request that introduces a human‑language word that is
neither a concept in the ontology nor on the explicit allow‑list.

This is the differentiating feature that makes this project a
*meta*‑ontology rather than a normal ontology.

## How it works

```
                   ┌──────────────────────┐
                   │ tracked text files   │
                   │  (Markdown, Rust     │
                   │   doc-comments,      │
                   │   lino labels)       │
                   └──────────┬───────────┘
                              │
                  words::scan_path
                              │
                              ▼
                ┌──────────────────────────┐
                │ candidate words          │
                │ (lowercased, lemmatised) │
                └──────────┬───────────────┘
                           │
                ┌──────────┴────────────────┐
                │                           │
        ontology concept?           on allowlist?
                │                           │
                ▼                           ▼
            keep                          keep
                                            │
                ▼                           ▼
            (others) ──────────► report as unknown
```

Anything that survives the two filters is reported with file:line and the
process exits non‑zero.

## Where it runs

1. **Locally**: `cargo run -- check-words .` (developer feedback).
2. **Pre‑commit**: optional hook in `.pre-commit-config.yaml`.
3. **CI**: a step in `.github/workflows/release.yml` (added in PR #2).

## CI integration

Following the conventions documented in
[docs/case-studies/issue-11/README.md](../case-studies/issue-11/README.md), the
check uses `::error::` and `exit 1` (not warning + 0) so PRs are actually
blocked.

```yaml
# === ONTOLOGY COVERAGE CHECK ===
ontology-coverage:
  name: Ontology Word Coverage
  runs-on: ubuntu-latest
  needs: [detect-changes]
  if: |
    always() && !cancelled() && (
      github.event_name == 'pull_request' ||
      github.event_name == 'push'
    )
  steps:
    - uses: actions/checkout@v6
    - uses: dtolnay/rust-toolchain@stable
    - run: cargo install rust-script
    - name: Check ontology word coverage
      run: rust-script scripts/check-ontology-coverage.rs
```

## What is a "word"?

The tokeniser does not consider these as human‑language words and skips them
silently:

- Code spans / fenced code blocks in Markdown.
- Markdown link targets and image filenames.
- HTML tag names.
- URLs (`https://...`).
- Identifiers in Rust source (only doc comments are scanned).
- `data/**/*.lino` reference identifiers (only `(label "...")` strings are
  scanned).
- Numbers, version strings (`v1.2.3`).
- Anything matching `[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+` (file
  paths / version triples).
- Single letters.

What *is* scanned:

- Markdown prose (anything outside code spans/blocks/links).
- Rust doc comments (`///`, `//!`).
- lino label strings.

## Allow‑list

`data/allowlist.lino` lives in version control. Each entry is:

```
(allowlist word: "github" reason: "proper noun: GitHub")
```

Adding to the allow‑list is a normal code review — and the `reason` field is
mandatory. Over time the allow‑list shrinks as more concepts enter the
ontology; M1 has a "zero allow‑list inside `data/`" goal.

## Performance

- Scan happens in‑process; no shelling out.
- ~5 000 LoC scans in well under a second on a developer laptop.

## Known limits (acknowledged, scheduled for M1)

- Lemmatisation is trivial English plural‑strip. Words like "ontologies" /
  "ontology" map to the same key; "indices" / "index" don't.
- Only English is scanned in M0. Other top‑20 languages enter the scanner in
  M2 once exponents are loaded.
- Markdown parser is regex‑based for now; a full CommonMark parser
  (`pulldown-cmark`) is on the M1 list.

## Tests

- `tests/unit/words.rs` — fixture inputs, expected token outputs.
- `tests/integration/cli.rs` — runs `check-words` on a small fixture tree
  with one deliberately unknown word, asserts exit code and message.
