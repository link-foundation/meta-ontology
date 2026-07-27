# Source Inventory and Research Method

## Method

The investigation used the GitHub API/CLI for repository state and official
OpenMetadata documentation and source repositories for technical claims.
Marketing claims and time-sensitive counts are treated as observations, not as
architecture requirements. Initial access date: 2026-07-12; implementation
facts and PR feedback were refreshed on 2026-07-27.

## Local issue and pull request

| Evidence | Observation |
| --- | --- |
| [Issue 5](https://github.com/link-foundation/meta-ontology/issues/5) | Open; asks for data collection, deep online research, complete requirements, solution options, plans, and component/library evaluation in one PR |
| [PR 6](https://github.com/link-foundation/meta-ontology/pull/6) | Draft placeholder on branch `issue-5-e2602184e2c5` at collection time |
| Issue comments API | No comments as of 2026-07-27 |
| PR review comments API | No inline comments as of 2026-07-27 |
| PR conversation comments API | Ten comments as of 2026-07-27, including the `meta-language` direction and the request for actual implementation |
| PR reviews API | No reviews as of 2026-07-27 |
| [Issue 1 case study](../issue-1/README.md) | Establishes this repository's requirements/research/plan precedent |
| [Issue 3 case study](../issue-3/README.md) | Establishes the collected-evidence and explicit-root-cause precedent |

## OpenMetadata repository snapshot

The [OpenMetadata repository](https://github.com/open-metadata/OpenMetadata)
reported Apache-2.0 licensing, `main` as its default branch, and a multi-language
codebase. The latest GitHub release observed was
[`1.13.1-release`](https://github.com/open-metadata/OpenMetadata/releases/tag/1.13.1-release),
published 2026-06-27. Counts such as stars, forks, connectors, schemas, or open
issues are intentionally not used to choose this repository's architecture.

Relevant upstream areas:

- [`openmetadata-spec`](https://github.com/open-metadata/OpenMetadata/tree/main/openmetadata-spec):
  canonical JSON Schemas and generated model inputs.
- [`openmetadata-service`](https://github.com/open-metadata/OpenMetadata/tree/main/openmetadata-service):
  server, repositories, APIs, authorization, and event handling.
- [`ingestion`](https://github.com/open-metadata/OpenMetadata/tree/main/ingestion):
  connector and workflow framework.
- [`openmetadata-ui`](https://github.com/open-metadata/OpenMetadata/tree/main/openmetadata-ui):
  web application kept separate from the metadata service.
- [`openmetadata-clients`](https://github.com/open-metadata/OpenMetadata/tree/main/openmetadata-clients):
  generated/client-facing access layers.
- [Contribution guide](https://docs.open-metadata.org/v1.12.x/developers/contribute):
  issue-based contribution and local build/test workflow.

## Link Foundation implementation toolchain

The maintainer's direction ([PR 6 comment, 2026-07-13](https://github.com/link-foundation/meta-ontology/pull/6#issuecomment-4951163416))
is that the full implementation should be built on the Link Foundation
`meta-language` library rather than on a bespoke stack. That comment names
[`link-foundation/meta-language#179`](https://github.com/link-foundation/meta-language/issues/179)
as the enabling prerequisite; it was closed as completed on 2026-07-13, so the
prerequisite is now satisfied. This section records that ecosystem so the plan
below can prefer it over generic crates.

| Component | Observation | Relevance to the plan |
| --- | --- | --- |
| [`meta-language`](https://github.com/link-foundation/meta-language) | Universal, self-describing meta language backed by a links network; Rust crate 0.54.0 selected for this implementation; parity JavaScript package; Unlicense | Implemented foundation for the normalized semantic network, full-match verification, term lookup, and versioned snapshots |
| [`meta-language#179`](https://github.com/link-foundation/meta-language/issues/179) | "Support all file formats … required at meta-ontology#6"; closed/completed 2026-07-13 | Unblocks format translation/transformation/storage, which is the interchange requirement (Phase 4) |
| [`links-notation`](https://github.com/link-foundation/links-notation) | lino parser/serializer already used for every `data/*.lino` file | Remains the canonical storage syntax; the semantic model layers on top |
| [`lino-arguments`](https://github.com/link-foundation/lino-arguments) | clap-style CLI argument parser with `.lenv` config | Already the CLI boundary; reused unchanged |
| [`lino-objects-codec`](https://github.com/link-foundation/meta-ontology/blob/main/Cargo.toml) | Feature-gated runtime object codec (`codec`) staged for M3/M4 | Object encoding at the service and web-app boundaries |

The repository README already states this project "follows the issue's directive
to use the `link-foundation` toolchain," so preferring `meta-language` is
consistent with existing direction, not a new dependency policy.

The maintainer's later feedback
([PR 6 comment, 2026-07-26](https://github.com/link-foundation/meta-ontology/pull/6#issuecomment-5085348085))
clarified that documentation alone was insufficient. The catalog, ingestion,
interchange, CLI, data migration, examples, and tests in this PR implement the
in-process recommendations; only components blocked by explicit scale/service
decision gates remain future work.

## Primary technical documentation

| Topic | Source | Evidence used |
| --- | --- | --- |
| Metadata standard | [Metadata Standard](https://docs.open-metadata.org/v1.11.x/api-reference/main-concepts/metadata-standard) | Canonical, language-neutral schemas drive generated models and APIs |
| REST conventions | [APIs](https://docs.open-metadata.org/v1.12.x/api-reference/main-concepts/metadata-standard/apis) | Schema-first resources, versioned `/api/v1` namespace, consistent collection/instance URIs |
| Connector architecture | [Metadata ingestion deep dive](https://docs.open-metadata.org/v1.12.x/developers/contribute/codebase-deep-dives/metadata-ingestion) | Source-to-sink pipeline, topology, result/error handling, fingerprint-based incremental ingestion |
| Ingestion capabilities | [Ingestion overview](https://docs.open-metadata.org/v1.12.x/connectors/ingestion) | Metadata, usage, lineage, profiling, quality, and versioning are distinct workflows |
| Lineage | [Lineage deep dive](https://docs.open-metadata.org/v1.12.x/developers/contribute/codebase-deep-dives/lineage-ingestion) | Relationships are extracted, resolved, and published as graph data |
| Connector schemas | [Define the JSON Schema](https://docs.open-metadata.org/latest/developers/contribute/developing-a-new-connector/define-json-schema) | One connection schema generates consistent Java, Python, and TypeScript types |
| Governance | [Data Governance](https://docs.open-metadata.org/v1.12.x/how-to-guides/data-governance) | Glossaries, classifications, ownership, and policies are first-class concepts |
| Glossary semantics | [Glossary Term](https://docs.open-metadata.org/v1.12.x/how-to-guides/data-governance/glossary/glossary-term) | Definitions, synonyms, related terms, tags, and references form a semantic graph |
| Domains | [Domains and Data Products](https://docs.open-metadata.org/v1.12.x/how-to-guides/data-governance/domains-%26-data-products) | Domain ownership and discoverable, documented products support decentralized stewardship |
| Authorization | [Roles and Policies](https://docs.open-metadata.org/v1.12.x/how-to-guides/admin-guide/roles-policies) | Authorization evaluates actor, resource attributes, and operation |
| Quality model | [Data Quality API](https://docs.open-metadata.org/v1.12.x/api-reference/data-quality) | Reusable definitions, applied cases, suites, and results separate rule from execution |
| Observability | [Quality and Observability](https://docs.open-metadata.org/v1.12.x/how-to-guides/data-quality-observability) | Tests, profiling, alerts, and incident handling form a feedback loop |
| Deployment/security | [Deployment](https://docs.open-metadata.org/v1.12.x/deployment) and [Security](https://docs.open-metadata.org/v1.12.x/deployment/security) | Production operations and identity-provider integration are explicit concerns |

## External standards and candidate components

These are evaluated as interoperability options, not mandatory dependencies.

| Need | Candidate | Intended use |
| --- | --- | --- |
| Canonical model + typed graph | [`meta-language`](https://github.com/link-foundation/meta-language) | Adopted foundation for the verified semantic network and snapshots |
| Multi-format translation/storage | [`meta-language#179`](https://github.com/link-foundation/meta-language/issues/179) | Enables format translation/transformation/storage for interchange (Phase 4); now ready |
| Data contract validation | [JSON Schema 2020-12](https://json-schema.org/draft/2020-12) | Portable interchange schema; keep lino canonical if desired |
| Semantic graph exchange | [RDF 1.2](https://www.w3.org/TR/rdf12-concepts/) / [OWL 2](https://www.w3.org/TR/owl2-overview/) | Import/export and ontology-tool interoperability |
| Graph constraints | [SHACL](https://www.w3.org/TR/shacl12-core/) | Express portable graph validity rules |
| Provenance | [PROV-O](https://www.w3.org/TR/prov-o/) | Map source, agent, derivation, and activity metadata |
| Lineage events | [OpenLineage](https://openlineage.io/docs/spec/) | Future pipeline lineage interchange, not core concept storage |
| Rust serialization | [`serde`](https://docs.rs/serde/latest/serde/) | Adopted for typed interchange models |
| Rust JSON Schema | [`schemars`](https://docs.rs/schemars/latest/schemars/) | Adopted to generate JSON Schema from Rust types |
| Schema validation | [`jsonschema`](https://docs.rs/jsonschema/latest/jsonschema/) | Adopted to validate imported JSON |
| Stable identifiers | [`uuid`](https://docs.rs/uuid/latest/uuid/) | UUID identity when content-derived IDs are unsuitable |
| HTTP service | [`axum`](https://docs.rs/axum/latest/axum/) | Existing roadmap choice for M3 |
| API description | [`utoipa`](https://docs.rs/utoipa/latest/utoipa/) | Generate OpenAPI from Rust endpoint types |
| Embedded search | [`tantivy`](https://docs.rs/tantivy/latest/tantivy/) | Optional local full-text index before external search infrastructure |
| Graph algorithms | [`petgraph`](https://docs.rs/petgraph/latest/petgraph/) | Cycle, reachability, and integrity analysis if custom traversal becomes costly |

## Reproducibility notes

- Documentation versions are included in URLs where the official site exposed a
  stable version. `latest` links should be rechecked before implementation.
- The compact JSON snapshots record mutable facts. Full upstream repository
  contents are not vendored because that would add millions of unrelated files
  and create licensing/update noise.
- Recommendations are reasoned adaptations to the current repository; they are
  not claims that OpenMetadata implements the exact proposed Rust design.
