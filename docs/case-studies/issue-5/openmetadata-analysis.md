# Deep Analysis: What Meta-Ontology Should Learn from OpenMetadata

## Context and limits

OpenMetadata solves enterprise metadata discovery, ingestion, governance,
lineage, quality, and collaboration. Meta-ontology currently loads a small lino
graph into an in-memory Rust library and exposes it through a CLI. Their shared
domain is metadata graphs; their scale, product scope, and operational needs are
not comparable.

Accordingly, this analysis extracts invariants and interfaces. It rejects
technology copying where the problem is absent.

## 1. Canonical schema as the center of the system

OpenMetadata defines entities and types in language-neutral schemas and derives
models used by server, ingestion, and UI layers. This prevents three subtly
different definitions of the same entity.

Meta-ontology already has Rust structs plus conventions embedded in lino. The
risk is that conventions such as tuple position, mapping kind, language, or
definition source evolve without a machine-checkable contract.

### Adaptation

- Declare a `schema_version` for the dataset and interchange representation.
- Keep one authoritative Rust domain model.
- Derive a JSON Schema interchange contract from it, or check a hand-maintained
  schema against generated fixtures.
- Validate lino-to-model normalization independently of parser success.
- Reject unknown fields only at stable boundaries; provide migrations when a
  stored format changes.

JSON is an interchange format here, not a replacement for Links Notation.

## 2. Identity, names, and references

Metadata systems must distinguish machine identity, fully qualified names,
display names, and aliases. A graph that uses a mutable label as identity makes
rename operations destructive.

### Adaptation

Add an opaque `ConceptId` and make names indexed attributes. Define uniqueness,
normalization, alias, deprecation, and redirect rules. A mapping or definition
references the ID; CLI lookup may accept a canonical name or alias. Initially,
IDs can be explicitly stored strings. UUIDs are useful only if decentralized
creation makes coordinated human-readable IDs unsafe.

### Required invariants

- IDs are unique and immutable.
- A canonical name is unique within its namespace and language rules.
- Every edge endpoint resolves.
- Alias chains terminate and cannot fork ambiguously.
- Removing a referenced concept requires an explicit migration or tombstone.

## 3. Typed relationships and graph integrity

OpenMetadata treats lineage, ownership, classification, glossary association,
and containment as relationships with meaning, rather than arbitrary strings.
Meta-ontology needs the same discipline on a much smaller scale.

### Adaptation

Define a relation registry containing at least:

- relation ID and canonical name;
- source and target concept categories;
- cardinality;
- inverse, symmetry, and transitivity declarations where meaningful;
- lifecycle state and provenance.

Do not infer logical properties from English labels. Validation should report
the source file and record for dangling endpoints, invalid cardinality, and
forbidden relation types. Cycles remain legal—the ontology is intentionally a
graph—but selected acyclic relations such as alias redirects can be checked.

## 4. Provenance, versioning, and change events

OpenMetadata preserves entity versions and uses source fingerprints to avoid
unnecessary ingestion updates. Both ideas transfer well.

### Adaptation

Each imported assertion should be able to carry:

- source URI or source document;
- source record/version and retrieval time;
- creating agent (human, connector, or migration);
- confidence or review state when the assertion is inferred;
- license/usage note for externally sourced descriptions;
- deterministic content fingerprint.

The loader can compare fingerprints and produce `Created`, `Updated`,
`Unchanged`, `Deprecated`, and `Rejected` records. Start with newline-delimited
JSON written by an explicit command. A database event table, webhook, or broker
is unnecessary until another process consumes events.

This also supports reproducible case studies: a claim can point to its source
and the exact import decision.

## 5. Ingestion as a normalized pipeline

OpenMetadata connector code separates source extraction from normalized entity
publication and uses a declarative topology for parent/child traversal.

### Adaptation

Use small Rust interfaces:

```text
Source -> RawRecord -> Normalizer -> CandidateAssertion
       -> Validator -> OntologyChange -> Sink/Report
```

The first implementation should be a fixture connector, not a large catalog.
It must demonstrate discovery, normalization, dry-run, idempotency,
checkpointing, structured errors, provenance, and a second unchanged run.

Connector capability declarations should state which entity/relation kinds are
supported. Configuration must be typed, secrets must not appear in reports, and
partial failures must not silently create a partially valid graph.

## 6. Layered quality and observability

OpenMetadata separates reusable test definitions, applications of those tests,
suites, executions, and results. Meta-ontology can adopt that conceptual model
without building a scheduler or UI.

### Layers

1. **Syntax:** lino parses.
2. **Shape:** records conform to the versioned model.
3. **Referential integrity:** IDs and relation endpoints resolve.
4. **Semantic constraints:** cardinality, relation domain/range, alias rules,
   required definition/language coverage.
5. **Quality policy:** provenance, ownership, review status, mapping coverage,
   duplicate/suspicious definitions.
6. **Repository policy:** files format, tests pass, word coverage, size limits.

Return stable diagnostic codes with file/record locations. Produce a JSON report
for CI in addition to human output. Trend history and alert routing should wait
until repeated scheduled validation exists.

## 7. Governance belongs in the model

OpenMetadata models ownership, glossaries, classifications, domains, policies,
and lifecycle status. For meta-ontology, the immediate value is accountability
and explainability, not enterprise access control.

### Adaptation

- Add optional owner/steward references at dataset, namespace, and concept level.
- Define lifecycle values such as `draft`, `reviewed`, `stable`, `deprecated`.
- Separate descriptive classification from security policy.
- Let policies refer to metadata attributes so the future service can evaluate
  actor + resource + operation.
- Record review decisions as provenance rather than overwriting history.

Authentication and authorization enforcement belongs at the future HTTP/API
boundary. The library should expose policy-relevant attributes without deciding
who the current user is.

## 8. API and client boundaries

OpenMetadata's schema-first REST API demonstrates the value of consistent
resource naming and shared types. Meta-ontology's M3 service should not expose
internal loader structs directly.

### Proposed resources

- `/api/v1/concepts` and `/api/v1/concepts/{id}`;
- `/api/v1/relations` and graph traversal endpoints;
- `/api/v1/definitions`, `/mappings`, and `/languages`;
- `/api/v1/validation-runs` and `/changes`;
- explicit import/export endpoints with dry-run support.

Use cursor pagination, sparse field selection only when needed, structured
errors, ETags for optimistic concurrency, and OpenAPI generation. Defer JSON
Patch until partial updates and conflict semantics are fully specified.

## 9. Search and discovery

OpenMetadata uses dedicated indexing because it serves a large, heterogeneous
catalog. The current ontology can search normalized in-memory fields.

Define a `SearchIndex` interface only after search semantics are specified:
exact ID/name, alias, language-aware text, relation filters, and ranking. A
linear implementation is the correct baseline. Tantivy is a plausible embedded
step if benchmarks show a need; an external cluster is not.

## 10. Security and operations

OpenMetadata documents authentication, authorization, secrets, upgrades,
backups, and production deployment. These are reminders that a service boundary
creates obligations.

Before M3 is production-ready, specify:

- read-only versus mutation endpoints;
- authentication and authorization model;
- request limits and payload limits;
- secret handling for connectors;
- audit/change retention;
- schema/data migration and backup/restore procedures;
- health, readiness, metrics, and structured logging;
- threat tests for malformed graphs and expensive traversal.

None of this requires adopting OpenMetadata's deployment topology today.

## Risks and countermeasures

| Risk | Countermeasure |
| --- | --- |
| Architecture astronautics | Require a measured use case and acceptance test for each infrastructure dependency |
| Two canonical models (lino and JSON) | Define lino as storage syntax and Rust model as semantic authority; JSON is generated interchange |
| Breaking old datasets | Version schemas and test migration fixtures |
| Mutable names break links | Introduce stable IDs before importers or APIs |
| Connector-specific core fields | Normalize behind a narrow candidate-assertion contract |
| Unverifiable external claims | Require provenance and license metadata |
| Quality rules become one opaque command | Layer checks and stable diagnostic codes |
| Enterprise governance overwhelms MVP | Start with owner, lifecycle, provenance, and review status |
| Search stack adds operations burden | Benchmark linear search; use embedded indexing before external services |
| Upstream practices change | Pin source/version/date and re-evaluate at implementation time |

## Implementation vehicle: the meta-language library

OpenMetadata builds its own canonical schema, model generation, and typed graph.
Meta-ontology does not need to reproduce that machinery, because the Link
Foundation ecosystem already provides it. The maintainer's direction on PR 6 is
to implement the practices above on the
[`meta-language`](https://github.com/link-foundation/meta-language) library once
its enabling prerequisite is ready; that prerequisite,
[`meta-language#179`](https://github.com/link-foundation/meta-language/issues/179)
(multi-format translation/transformation/storage), was closed as completed on
2026-07-13.

This changes *how*, not *what*: the invariants and boundaries in sections 1–10
still hold. The in-process implementation in this PR realizes them as follows.

- **Canonical model and typed graph (sections 1–3).** The loader normalizes
  Links Notation into one Rust contract and then constructs a verified
  `meta-language::LinkNetwork`. Stable IDs and explicit relationships remain
  available through the typed catalog model, while the network supplies shared
  graph semantics, term lookup, full-match verification, and snapshots.
- **Interchange (section 1 and the JSON boundary).** Route format
  semantics through the same network. The delivered JSON boundary is generated
  from and validated against the typed catalog contract because it carries
  catalog-specific governance and provenance fields. Additional
  `meta-language` formats require explicit supported-field and loss tests; this
  PR does not claim an unproved lossless conversion.
- **Rust/JS parity for M3 and M4.** `meta-language` ships a Rust crate and a
  parity JavaScript package, so the same model underpins the CLI, the
  microservice, and the WASM/React web app instead of three divergent shapes.

`serde`, `schemars`, and `jsonschema` cover the concrete typed-JSON gap.
`petgraph`, an external search service, and OpenMetadata runtime components were
not added.

## Recommended outcome

The durable OpenMetadata lesson is contract coherence: schemas, ingestion,
service APIs, governance, versioning, quality, and clients describe the same
entities. Meta-ontology now establishes that coherence in process: one model
drives LiNo loading, the `meta-language` network, JSON Schema, imports,
validation, change plans, connector normalization, CLI output, and search. A
clean modular monolith can later move selected interfaces across process
boundaries without redesigning the ontology.
