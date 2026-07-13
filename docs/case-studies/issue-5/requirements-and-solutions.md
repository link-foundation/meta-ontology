# Requirements, Solutions, and Implementation Plan

## Requirement derivation

Issue 5 names OpenMetadata in its title and uses the repository's standard case
study instruction body. The explicit requirements are R1-R6. R7-R10 are implied
by “use best practices” and by making the output reviewable and actionable.

## Complete requirement list

| ID | Requirement | Evidence | Delivered here |
| --- | --- | --- | --- |
| R1 | Collect issue-related data under `docs/case-studies/issue-5` | Issue body | Source inventory and compact snapshots |
| R2 | Perform a deep case-study analysis | Issue body | Architectural comparison, adaptations, risks |
| R3 | Search online for additional facts and data | Issue body | Versioned official docs, upstream source, standards |
| R4 | List every requirement from the issue | Issue body | This table |
| R5 | Propose possible solutions and a plan for each requirement | Issue body | Requirement dispositions and phased plan below |
| R6 | Check existing components/libraries that solve or assist each solution | Issue body | Standards/component evaluation in source inventory and below |
| R7 | Identify OpenMetadata practices suitable for meta-ontology | Issue title | Ten practice areas with fit decisions |
| R8 | Avoid adopting unsuitable complexity | “best practices” requires contextual judgment | Explicit deferrals and decision gates |
| R9 | Make recommendations verifiable | Deep analysis requirement | Acceptance tests, provenance, pinned sources |
| R10 | Complete planning and execution in one PR | Issue body | All research artifacts are in PR 6 |

## Solutions by requirement

### R1: collect data

**Options:** vendor the upstream repository; save complete API dumps; or preserve
a curated, time-stamped evidence set with primary links.

**Decision:** curated evidence. Vendoring a multi-gigabyte, Apache-licensed
repository would be noisy and stale quickly. Full GitHub API payloads contain
mostly transport metadata. The snapshots preserve the mutable facts needed to
reproduce this analysis, while the inventory points to exact upstream areas.

**Verification:** every local artifact named in the case-study README exists;
mutable observations include an observation date; empty comment/review streams
are recorded.

### R2 and R7: deep analysis of applicable practices

**Options:** feature checklist, technology-stack comparison, or domain-boundary
analysis.

**Decision:** domain-boundary analysis. It explains how schema, identity,
relationships, provenance, ingestion, validation, governance, API, search, and
operations affect this repository and explicitly states what not to copy.

**Verification:** each practice has current-state context, an adaptation, and
either an implementation slice or a deferral criterion.

### R3: online research

**Options:** secondary articles, current official docs, or source-only review.

**Decision:** combine official versioned documentation with the upstream source
tree and recognized standards. This gives design explanation plus inspectable
implementation boundaries. Secondary claims are unnecessary.

**Verification:** sources are primary, linked, access-dated, and separated from
recommendations. Time-sensitive facts are not treated as permanent.

### R4: enumerate requirements

**Decision:** assign stable IDs to explicit and implied requirements. This avoids
mistaking a long architectural summary for issue completion.

**Verification:** every phrase in the issue body maps to R1-R6; the title's
“best practices” goal maps to R7-R9; single-PR completion maps to R10.

### R5: solutions and plans

**Decision:** use thin vertical slices, each ending in data fixtures, library
behavior, CLI behavior, diagnostics, and tests. Avoid a “platform rewrite” plan.

**Verification:** the phased plan below has dependencies, acceptance criteria,
and stop/go gates.

### R6: components and libraries

**Decision:** prefer the Link Foundation toolchain first, then standards and
focused Rust crates, over an OpenMetadata runtime dependency. Per maintainer
direction ([PR 6 comment](https://github.com/link-foundation/meta-ontology/pull/6#issuecomment-4951163416)),
the model, graph, and interchange layers should be built on `meta-language`
now that its enabling issue
([`meta-language#179`](https://github.com/link-foundation/meta-language/issues/179))
is closed. OpenMetadata can interoperate later via JSON/RDF/OpenLineage or a
purpose-built connector.

| Capability | Build/adopt decision | Candidate |
| --- | --- | --- |
| Canonical model + typed graph | Adopt as foundation | `meta-language` (links-network model, Rust/JS parity) |
| Multi-format interchange | Adopt for Phase 4 | `meta-language` format support (`meta-language#179`, now ready) |
| Typed serialization | Adopt | `serde` |
| JSON Schema generation | Prototype and compare generated diff in CI | `schemars` |
| JSON Schema validation | Adopt at interchange boundary | `jsonschema` |
| Stable opaque IDs | Start with explicit IDs; use crate if UUIDs chosen | `uuid` |
| Graph algorithms | Keep current code until complexity warrants dependency | `petgraph` |
| RDF/OWL exchange | Design mapping first; evaluate maintained Rust libraries with fixtures | W3C RDF/OWL |
| Provenance vocabulary | Map internal provenance fields | PROV-O |
| Constraint exchange | Export selected rules later | SHACL |
| HTTP API | Use existing roadmap selection | `axum` |
| OpenAPI | Generate from endpoint/model types | `utoipa` |
| Search | Linear baseline, then benchmark embedded option | `tantivy` |
| Pipeline lineage | Interoperate only when pipeline entities exist | OpenLineage |

Crates must be re-evaluated for maintenance, license, MSRV, WASM compatibility,
and dependency footprint when their slice begins. Listing one is not approval to
add it. Where `meta-language` already covers a capability (canonical model,
links-network graph, or multi-format interchange), prefer it and treat the
generic crate as a fallback only if a concrete gap is proven.

### R8: control complexity

**Decision:** use decision gates:

- no database until persistence, concurrency, or corpus size requires it;
- no search service until a benchmark fails an agreed latency target;
- no scheduler until at least two recurring connectors exist;
- no distributed components until independent scaling or availability is a
  documented requirement;
- no authorization engine until mutations cross a service boundary.

### R9: verifiability

**Decision:** each implementation slice starts with fixtures and failing tests.
All imports expose provenance and a dry-run report. Diagnostics have stable
codes. Benchmarks, not analogy, trigger infrastructure.

### R10: single-PR execution

**Decision:** PR 6 contains the complete research deliverable and future plans.
The product slices remain separately reviewable future work because implementing
all of OpenMetadata's capabilities would violate R8 and the issue asks for
solution proposals/plans, not a platform rewrite.

## Prioritized implementation roadmap

### Implementation vehicle

Every product phase below is expected to be built on the Link Foundation
`meta-language` library rather than on a from-scratch stack, per the maintainer's
direction on PR 6. `meta-language` provides the links-network model with
guaranteed Rust/JS parity, so the same foundation serves the CLI, the M3
microservice, and the M4 WASM/React app. Its enabling prerequisite,
[`meta-language#179`](https://github.com/link-foundation/meta-language/issues/179)
(multi-format translation/transformation/storage), was closed as completed on
2026-07-13, so Phase 4 interchange is now unblocked. The generic crates named
above remain fallbacks for capabilities `meta-language` does not yet cover.

### Phase 0: accept this architecture record

Deliverables:

- this case study and source inventory;
- explicit decisions and deferrals;
- tracked follow-up issues for approved phases.

Acceptance: documentation links pass; local CI passes; reviewers can trace each
requirement to a delivered artifact.

### Phase 1: versioned identity and validation

Dependencies: none.

1. Add failing fixtures for duplicate IDs, dangling edges, invalid relation
   domain/range, ambiguous aliases, and unsupported schema versions.
2. Introduce `ConceptId`, dataset `schema_version`, lifecycle state, and source
   location without changing CLI display behavior.
3. Add layered validators and stable diagnostic codes.
4. Add `validate --format human|json` and golden reports.
5. Provide a migration for current lino fixtures.

Acceptance:

- all current data migrates without semantic loss;
- invalid fixtures fail at the intended validation layer;
- a rename preserves ID-based references;
- CLI output remains backward compatible unless explicitly versioned.

### Phase 2: provenance and deterministic changes

Dependencies: Phase 1.

1. Model source URI/document, agent, time, review state, license note, and
   content fingerprint.
2. Test create/update/unchanged/deprecate decisions before implementation.
3. Add dry-run diff and newline-delimited JSON change export.
4. Ensure identical imports are idempotent and deterministic.

Acceptance: importing the same fixture twice produces zero updates on the
second run; every changed assertion points to source evidence.

### Phase 3: one connector contract

Dependencies: Phases 1-2.

1. Define typed source configuration, capability declaration, checkpoint, raw
   record, normalized candidate, and structured error contracts.
2. Build one fixture-backed connector end to end.
3. Test partial failures, secret redaction, retry classification, dry run, and
   resume behavior.
4. Document how a second connector would be added without core-model changes.

Acceptance: two unchanged runs are idempotent; invalid records are quarantined
with evidence; no source-specific field enters the core model without a general
domain reason.

### Phase 4: interchange

Dependencies: Phase 1; provenance fields from Phase 2 recommended.

1. Generate or maintain JSON Schema for the stable public model.
2. Add JSON import/export round-trip fixtures.
3. Map a deliberately small RDF/OWL/PROV-O subset and publish loss reports.
4. Evaluate SHACL export for constraints; do not claim lossless round trips
   until tests prove them.

Acceptance: schema validation runs in CI; JSON round trips preserve IDs and
relationships; semantic exports declare unsupported/lossy fields.

### Phase 5: service API (existing roadmap M3)

Dependencies: stable Phases 1-4 contracts.

1. Specify OpenAPI and error/pagination/concurrency conventions first.
2. Implement read-only resources in `axum` over the library.
3. Add mutation/import endpoints only with authentication, authorization,
   limits, audit changes, and threat tests.
4. Add health/readiness, structured logs, metrics, backup, and migration docs.

Acceptance: generated client contract tests pass; malformed/expensive requests
are bounded; library, CLI, and API return semantically consistent entities.

### Phase 6: search and UI (existing roadmap M4)

Dependencies: representative corpus and measured UX needs.

1. Specify query semantics and benchmark linear search.
2. Add a `SearchIndex` abstraction only if needed.
3. Evaluate Tantivy if the baseline misses the target.
4. Build UI against the public API/model, including provenance and validation
   state rather than hiding them.

Acceptance: a recorded benchmark justifies any new index; search relevance has
golden queries; the UI has browser tests and accessibility checks.

### Phase 7: governance enforcement and operations

Dependencies: mutating service and real multi-user requirements.

1. Turn owner, lifecycle, classification, and policy references into evaluated
   actor/resource/operation rules.
2. Add audit retention and policy decision tests.
3. Add connector scheduling only when recurring jobs require it.
4. Document upgrades, restore drills, incident response, and supported versions.

Acceptance: deny/allow tests cover every mutation class; restore drills recover
data and history; operational objectives are explicit.

## Cross-cutting test strategy

- Unit tests: IDs, normalization, fingerprints, rules, diagnostic codes.
- Property tests: parser/serializer round trips and arbitrary graph safety.
- Golden fixtures: version migrations, import reports, API schemas.
- Integration tests: source-to-normalized-to-validated-to-change pipeline.
- Compatibility tests: old lino data and public CLI behavior.
- Security tests: payload limits, traversal bounds, secret redaction, policy.
- Benchmarks: loader, validation, traversal, and search before optimization.
- WASM tests: dependencies and public types compile for the M4 target.

## Final recommendation

Approve Phases 1 and 2 as the first product follow-up, building them on the
`meta-language` library. They create the stable identity, validation, and
evidence foundation required by every later practice. Do not begin with
connectors, HTTP, search, or deployment infrastructure: doing so would freeze
weak contracts behind expensive boundaries.
