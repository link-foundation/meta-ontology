# Issue 5 Case Study: Applying OpenMetadata Practices

## Summary

Issue: [link-foundation/meta-ontology#5](https://github.com/link-foundation/meta-ontology/issues/5)

OpenMetadata is a mature metadata platform, while this repository is a small,
Links Notation-based ontology. The useful lesson is therefore not to copy its
Java/Python/TypeScript service stack. It is to adopt its boundaries: a canonical
schema, stable entity identity, explicit relationships, versioned changes,
connector isolation, API-first access, governance metadata, and automated
quality checks.

The implementation follows that incremental sequence in this PR. It strengthens
the in-process Rust model and lino data, adds interchange and ingestion
contracts, and provides a measured linear-search baseline. Persistence, HTTP,
distributed search, authorization enforcement, and UI work remain behind the
documented decision gates.

Research was performed on 2026-07-12 and refreshed for implementation on
2026-07-27. OpenMetadata is fast-moving, so facts that can change are pinned to
the observation date and links to primary sources.

## Collected Data

- [`source-inventory.md`](source-inventory.md) records the issue, PR, repository,
  release, documentation, and source-code evidence consulted.
- [`openmetadata-analysis.md`](openmetadata-analysis.md) describes transferable
  practices, mismatches, risks, and existing components that can help.
- [`requirements-and-solutions.md`](requirements-and-solutions.md) enumerates
  every explicit and implied requirement and gives a solution and staged plan.
- [`raw-data/issue-5.json`](raw-data/issue-5.json) preserves the issue snapshot.
- [`raw-data/research-observations.json`](raw-data/research-observations.json)
  preserves time-sensitive repository and PR observations in a compact,
  reviewable form.

No issue comments, inline review comments, or PR reviews existed when refreshed.
The PR conversation and the implementation request are recorded in the source
inventory and compact observations.

## Delivered implementation

The case study now has an executable counterpart:

- `catalog.rs` implements schema versioning, immutable IDs, typed relationships,
  provenance, governance, stable diagnostics, JSON interchange, deterministic
  change plans, and linear search.
- The ontology is backed by a verified `meta-language::LinkNetwork`; callers can
  obtain an immutable versioned `NetworkSnapshot`.
- `ingestion.rs` implements a source-neutral connector contract and a complete
  fixture connector with typed configuration, credential redaction,
  capabilities, checkpoints, quarantineable errors, normalization, dry run,
  resume, and idempotent replay.
- `validate`, `export-json`, `json-schema`, `plan-import`, and `search` expose
  the contracts through the existing CLI.
- The seed data declares its schema version, governance metadata, and explicit
  relationships in Links Notation.
- Focused unit and CLI integration tests reproduce invalid contracts and verify
  the successful end-to-end paths.

### Implementation vehicle

Per the maintainer's direction on
[PR 6](https://github.com/link-foundation/meta-ontology/pull/6#issuecomment-4951163416),
the product implementation is built on the Link Foundation
[`meta-language`](https://github.com/link-foundation/meta-language) library
(links-network model with Rust/JS parity) rather than a from-scratch stack. Its
enabling prerequisite,
[`meta-language#179`](https://github.com/link-foundation/meta-language/issues/179)
(multi-format translation/transformation/storage), was closed as completed on
2026-07-13. The delivered catalog uses its links-network, verification, term
lookup, and snapshot APIs. See
[`openmetadata-analysis.md`](openmetadata-analysis.md#implementation-vehicle-the-meta-language-library)
for how each practice maps onto that library.

### Practices to defer

- A separate database and search index until the corpus or query latency needs it.
- A workflow scheduler until multiple repeatable external ingestion jobs exist.
- Microservices, event buses, and Kubernetes until deployment requirements demand
  independent scaling or availability.
- A broad connector catalog before one end-to-end connector contract is proven.
- Enterprise RBAC before the HTTP service exists; design authorization fields now,
  enforce them at the service boundary later.

## Target Architecture

```text
source files / future connectors
              |
              v
      normalized import records
              |
              v
 parse -> schema -> graph -> semantic validation
              |
              v
     versioned ontology service
       /          |          \
     CLI       future API    WASM/UI
                  |
          optional persistence/search
```

The pipeline is the important boundary. Each consumer sees one validated model;
each importer produces one normalized representation.

## Decision Matrix

| OpenMetadata practice | Fit | Decision |
| --- | --- | --- |
| Canonical schema and generated/typed models | High | Implemented |
| Stable IDs, names, ownership, tags, versions | High | Implemented |
| Typed relationships and lineage | High | Implemented baseline |
| Source-to-sink connector topology | High | Implemented fixture pipeline |
| REST resources and JSON Patch | Medium | Design contracts now; implement at M3 |
| Change events and webhooks | Medium | Deterministic local plans implemented; webhooks deferred |
| Data quality tests and observability | Medium | Layered integrity diagnostics implemented |
| Search index | Low today | Linear baseline implemented; infrastructure deferred |
| Airflow-style workflow scheduling | Low today | Defer |
| Full production deployment stack | Low today | Do not copy |

## Completion Criteria

This case study and implementation satisfy issue 5 by:

- preserves issue- and PR-related observations under this directory;
- uses current primary online sources and distinguishes observations from
  recommendations;
- enumerates all issue requirements;
- evaluates OpenMetadata practices against this repository rather than merely
  summarizing OpenMetadata;
- identifies reusable standards, crates, and libraries;
- supplying a prioritized, testable implementation plan for every requirement;
- implementing the in-process phases selected by the decision gates;
- keeping only service- and scale-dependent work as explicit future slices.
