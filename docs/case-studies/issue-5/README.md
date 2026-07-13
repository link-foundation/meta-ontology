# Issue 5 Case Study: Applying OpenMetadata Practices

## Summary

Issue: [link-foundation/meta-ontology#5](https://github.com/link-foundation/meta-ontology/issues/5)

OpenMetadata is a mature metadata platform, while this repository is a small,
Links Notation-based ontology. The useful lesson is therefore not to copy its
Java/Python/TypeScript service stack. It is to adopt its boundaries: a canonical
schema, stable entity identity, explicit relationships, versioned changes,
connector isolation, API-first access, governance metadata, and automated
quality checks.

The recommended sequence is deliberately incremental. Strengthen the current
in-process Rust model and lino data first; add interchange and ingestion
contracts next; add persistence, search, HTTP, and a UI only when their roadmap
milestones and measured usage justify the operational cost.

Research was performed on 2026-07-12. OpenMetadata is fast-moving, so facts
that can change are pinned to the observation date and links to primary sources.

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

No issue comments, PR conversation comments, inline review comments, or PR
reviews existed when collected.

## Findings

### Practices to adopt now

1. **Schema-first domain model.** Validate every persisted concept, definition,
   mapping, language exponent, and relationship against a versioned contract.
2. **Stable identity separate from display names.** Renames must not break
   mappings, definitions, lineage, or future external references.
3. **Typed graph edges.** Replace implicit string conventions with a small,
   documented relation vocabulary and validate edge endpoints.
4. **Provenance and change history.** Record source, author/agent, timestamp,
   schema version, and change reason. Emit a deterministic change record when
   loading or updating data.
5. **Layered validation.** Separate parse validity, schema validity, graph
   integrity, semantic quality, and repository word coverage so failures are
   actionable.
6. **Connector boundary.** Future importers should produce the same normalized
   intermediate model; source-specific code must not leak into the core graph.
7. **Contract-driven interfaces.** The library remains authoritative. CLI,
   future HTTP, WASM, and importers consume the same service contracts.
8. **Governance primitives.** Ownership, lifecycle state, classifications, and
   policy references should be ordinary metadata, not UI-only annotations.

### Implementation vehicle

Per the maintainer's direction on
[PR 6](https://github.com/link-foundation/meta-ontology/pull/6#issuecomment-4951163416),
the product phases should be built on the Link Foundation
[`meta-language`](https://github.com/link-foundation/meta-language) library
(links-network model with Rust/JS parity) rather than a from-scratch stack. Its
enabling prerequisite,
[`meta-language#179`](https://github.com/link-foundation/meta-language/issues/179)
(multi-format translation/transformation/storage), was closed as completed on
2026-07-13, so the interchange work is now unblocked. See
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
| Canonical schema and generated/typed models | High | Adopt incrementally |
| Stable IDs, names, ownership, tags, versions | High | Add to domain model |
| Typed relationships and lineage | High | Add a relation vocabulary |
| Source-to-sink connector topology | High | Adapt as a small Rust trait pipeline |
| REST resources and JSON Patch | Medium | Design contracts now; implement at M3 |
| Change events and webhooks | Medium | Start with deterministic local events |
| Data quality tests and observability | Medium | Adapt to ontology integrity rules |
| Search index | Low today | Keep an interface; defer infrastructure |
| Airflow-style workflow scheduling | Low today | Defer |
| Full production deployment stack | Low today | Do not copy |

## Completion Criteria

This case study satisfies issue 5 when it:

- preserves issue- and PR-related observations under this directory;
- uses current primary online sources and distinguishes observations from
  recommendations;
- enumerates all issue requirements;
- evaluates OpenMetadata practices against this repository rather than merely
  summarizing OpenMetadata;
- identifies reusable standards, crates, and libraries;
- supplies a prioritized, testable implementation plan for every requirement.

The analysis and plan meet those criteria. Product changes are intentionally
proposed as future slices rather than bundled into this research PR.
