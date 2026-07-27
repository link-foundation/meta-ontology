//! OpenMetadata-inspired catalog contracts.
//!
//! The module keeps schema, identity, provenance, validation, ingestion,
//! governance, search, and interchange coherent without requiring a
//! distributed metadata platform.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use meta_language::{LinkNetwork, NetworkSnapshot, ParseConfiguration};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::ontology::{Concept, Ontology};

/// Current version of the public catalog/interchange contract.
pub const CURRENT_SCHEMA_VERSION: u32 = 1;

/// Stable identity for a concept.
#[derive(
    Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(transparent)]
pub struct ConceptId(String);

impl ConceptId {
    /// Creates an identity from its serialized representation.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Derives the migration identity used by legacy seed concepts.
    #[must_use]
    pub fn from_name(name: &str) -> Self {
        Self(format!("meta-ontology:{name}"))
    }

    /// Serialized identity.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Whether the identity has no value.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl fmt::Display for ConceptId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Review state attached to imported evidence.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReviewState {
    /// Evidence has not been reviewed.
    Draft,
    /// Evidence was checked by a repository contributor.
    #[default]
    Reviewed,
    /// Evidence should no longer be used for new assertions.
    Rejected,
}

/// Evidence and source location for an entity or relationship.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Provenance {
    /// URI or path of the source document.
    pub source_uri: String,
    /// One-based source line when known.
    pub source_line: usize,
    /// Source-native record identity or version.
    pub source_record: String,
    /// RFC 3339 observation time when supplied by an external source.
    pub observed_at: String,
    /// Agent responsible for normalization.
    pub agent: String,
    /// Review state of the evidence.
    pub review_state: ReviewState,
    /// License or usage note inherited from the source.
    pub license: String,
    /// SHA-256 of normalized content.
    pub fingerprint: String,
}

impl Provenance {
    /// Repository-safe provenance for a parsed `LiNo` source.
    #[must_use]
    pub fn for_source(source_uri: &str, source_line: usize) -> Self {
        Self {
            source_uri: source_uri.to_string(),
            source_line,
            source_record: String::new(),
            observed_at: String::new(),
            agent: "meta-ontology-loader".to_string(),
            review_state: ReviewState::Reviewed,
            license: "Unlicense".to_string(),
            fingerprint: String::new(),
        }
    }
}

/// Lifecycle state used by governance and search consumers.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum LifecycleState {
    /// Entity is available to consumers.
    #[default]
    Active,
    /// Entity remains readable but should not be selected for new use.
    Deprecated,
    /// Entity is retained only for history.
    Deleted,
}

/// Governance metadata kept beside the entity contract.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Governance {
    /// Responsible teams or people.
    pub owners: Vec<String>,
    /// Policy classifications such as `public`.
    pub classifications: Vec<String>,
    /// Searchable governance tags.
    pub tags: Vec<String>,
    /// Entity lifecycle state.
    pub lifecycle: LifecycleState,
}

/// Typed stable-ID relationship between concepts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Relationship {
    /// Source concept identity.
    pub from: ConceptId,
    /// Relationship kind.
    pub kind: String,
    /// Target concept identity.
    pub to: ConceptId,
    /// Source evidence for the assertion.
    pub provenance: Provenance,
}

impl Relationship {
    fn kind_is_supported(&self) -> bool {
        matches!(
            self.kind.as_str(),
            "defines" | "equivalent" | "broader" | "narrower" | "related"
        )
    }
}

/// Stable machine-readable validation code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiagnosticCode {
    /// `MO001`: dataset schema version is unsupported.
    #[serde(rename = "MO001")]
    UnsupportedSchemaVersion,
    /// `MO002`: more than one entity has the same stable identity.
    #[serde(rename = "MO002")]
    DuplicateId,
    /// `MO003`: an alias resolves to more than one identity.
    #[serde(rename = "MO003")]
    AmbiguousAlias,
    /// `MO004`: a relationship kind is outside the public vocabulary.
    #[serde(rename = "MO004")]
    UnsupportedRelationKind,
    /// `MO005`: relationship source does not exist.
    #[serde(rename = "MO005")]
    DanglingRelationSource,
    /// `MO006`: relationship target does not exist.
    #[serde(rename = "MO006")]
    DanglingRelationTarget,
    /// `MO007`: required provenance is missing.
    #[serde(rename = "MO007")]
    MissingProvenance,
    /// `MO008`: the meta-language source network is not a full match.
    #[serde(rename = "MO008")]
    NetworkVerification,
    /// `MO009`: more than one entity has the same canonical name.
    #[serde(rename = "MO009")]
    DuplicateName,
}

impl DiagnosticCode {
    /// Stable compact code used in human and JSON diagnostics.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UnsupportedSchemaVersion => "MO001",
            Self::DuplicateId => "MO002",
            Self::AmbiguousAlias => "MO003",
            Self::UnsupportedRelationKind => "MO004",
            Self::DanglingRelationSource => "MO005",
            Self::DanglingRelationTarget => "MO006",
            Self::MissingProvenance => "MO007",
            Self::NetworkVerification => "MO008",
            Self::DuplicateName => "MO009",
        }
    }
}

impl fmt::Display for DiagnosticCode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Severity of a catalog diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// Contract violation that blocks an import.
    Error,
    /// Reviewable condition that does not block an import.
    Warning,
}

/// One validation finding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diagnostic {
    /// Stable code.
    pub code: DiagnosticCode,
    /// Severity.
    pub severity: Severity,
    /// Human-readable detail.
    pub message: String,
    /// Source URI when known.
    pub source_uri: Option<String>,
    /// One-based source line when known.
    pub source_line: Option<usize>,
}

/// Layered validation result.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationReport {
    diagnostics: Vec<Diagnostic>,
}

impl ValidationReport {
    /// Whether the catalog has no error diagnostics.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        !self
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == Severity::Error)
    }

    /// Ordered validation findings.
    #[must_use]
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    fn error(
        &mut self,
        code: DiagnosticCode,
        message: impl Into<String>,
        provenance: Option<&Provenance>,
    ) {
        self.diagnostics.push(Diagnostic {
            code,
            severity: Severity::Error,
            message: message.into(),
            source_uri: provenance.map(|value| value.source_uri.clone()),
            source_line: provenance.map(|value| value.source_line),
        });
    }
}

/// Versioned JSON boundary. `LiNo` remains the repository source of truth.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct InterchangeDocument {
    /// Contract version.
    pub schema_version: u32,
    /// Entity records ordered by canonical name.
    pub concepts: Vec<Concept>,
    /// Typed relationships.
    pub relationships: Vec<Relationship>,
    /// Declared exponent languages.
    pub languages: Vec<String>,
    /// Explicit word-coverage exceptions.
    pub allowlist: Vec<String>,
}

/// Deterministic import decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeKind {
    /// New identity.
    Created,
    /// Existing identity with changed normalized content.
    Updated,
    /// Identity and normalized content are identical.
    Unchanged,
    /// Existing identity is absent from a full replacement document.
    Deprecated,
}

/// Kind of normalized object described by an import change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImportObjectKind {
    /// Concept entity.
    Concept,
    /// Typed relationship assertion.
    Relationship,
}

/// One dry-run import decision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportChange {
    /// Kind of normalized object.
    pub object_kind: ImportObjectKind,
    /// Stable entity ID or deterministic relationship key.
    pub id: String,
    /// Decision.
    pub kind: ChangeKind,
    /// Existing fingerprint when present.
    pub before: Option<String>,
    /// Candidate fingerprint when present.
    pub after: Option<String>,
}

/// Dry-run report for a normalized import.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportPlan {
    changes: Vec<ImportChange>,
}

impl ImportPlan {
    /// Deterministically identity-sorted decisions.
    #[must_use]
    pub fn changes(&self) -> &[ImportChange] {
        &self.changes
    }

    /// Serialize one deterministic change per newline.
    pub fn to_ndjson(&self) -> Result<String, CatalogError> {
        let mut output = String::new();
        for change in &self.changes {
            output.push_str(&serde_json::to_string(change)?);
            output.push('\n');
        }
        Ok(output)
    }
}

/// Search hit with a deterministic relevance score.
#[derive(Debug, Clone, Copy)]
pub struct SearchMatch<'a> {
    /// Matching concept.
    pub concept: &'a Concept,
    /// Higher values are more relevant.
    pub score: u32,
}

/// Errors at the JSON/import contract boundary.
#[derive(Debug, Error)]
pub enum CatalogError {
    /// JSON could not be decoded or encoded.
    #[error("JSON interchange error: {0}")]
    Json(#[from] serde_json::Error),
    /// JSON did not match the generated schema.
    #[error("JSON schema validation failed: {0}")]
    Schema(String),
    /// Candidate import failed validation.
    #[error("catalog validation failed with {count} diagnostic(s)")]
    Validation {
        /// Number of validation findings.
        count: usize,
        /// Full report for callers that need structured diagnostics.
        report: ValidationReport,
    },
}

impl Ontology {
    /// Public catalog schema version.
    #[must_use]
    pub const fn schema_version(&self) -> u32 {
        self.schema_version
    }

    /// Normalized canonical `meta-language` network.
    #[must_use]
    pub const fn network(&self) -> &LinkNetwork {
        &self.network
    }

    /// Immutable versioned network snapshot.
    #[must_use]
    pub fn snapshot(&self) -> NetworkSnapshot {
        self.network
            .snapshot(u64::from(self.schema_version), "meta-ontology catalog")
    }

    /// Iterate explicit typed relationships.
    pub fn relationships(&self) -> impl Iterator<Item = &Relationship> {
        self.relationships.iter()
    }

    /// Validate schema, identity, relationships, provenance, and source parse.
    #[must_use]
    pub fn validate(&self) -> ValidationReport {
        let mut report = ValidationReport::default();
        if self.schema_version != CURRENT_SCHEMA_VERSION {
            report.error(
                DiagnosticCode::UnsupportedSchemaVersion,
                format!(
                    "schema version {} is unsupported; expected {CURRENT_SCHEMA_VERSION}",
                    self.schema_version
                ),
                None,
            );
        }

        let mut ids = BTreeMap::<&ConceptId, &Concept>::new();
        let mut aliases = BTreeMap::<String, &ConceptId>::new();
        for concept in self.concepts.values() {
            if let Some(previous) = ids.insert(&concept.id, concept) {
                report.error(
                    DiagnosticCode::DuplicateId,
                    format!(
                        "concepts {} and {} share identity {}",
                        previous.name, concept.name, concept.id
                    ),
                    Some(&concept.provenance),
                );
            }
            for alias in concept.aliases.iter().chain(std::iter::once(&concept.name)) {
                let normalized = alias.to_lowercase();
                if let Some(previous) = aliases.insert(normalized.clone(), &concept.id) {
                    if previous != &concept.id {
                        report.error(
                            DiagnosticCode::AmbiguousAlias,
                            format!(
                                "alias {normalized:?} resolves to both {previous} and {}",
                                concept.id
                            ),
                            Some(&concept.provenance),
                        );
                    }
                }
            }
            if concept.provenance.source_uri.is_empty()
                || concept.provenance.agent.is_empty()
                || concept.provenance.license.is_empty()
                || concept.provenance.fingerprint.is_empty()
            {
                report.error(
                    DiagnosticCode::MissingProvenance,
                    format!("concept {} has incomplete provenance", concept.id),
                    Some(&concept.provenance),
                );
            }
        }

        for relation in &self.relationships {
            if relation.provenance.source_uri.is_empty()
                || relation.provenance.agent.is_empty()
                || relation.provenance.license.is_empty()
                || relation.provenance.fingerprint.is_empty()
            {
                report.error(
                    DiagnosticCode::MissingProvenance,
                    format!(
                        "relationship {}:{}:{} has incomplete provenance",
                        relation.from, relation.kind, relation.to
                    ),
                    Some(&relation.provenance),
                );
            }
            if !relation.kind_is_supported() {
                report.error(
                    DiagnosticCode::UnsupportedRelationKind,
                    format!("relationship kind {:?} is unsupported", relation.kind),
                    Some(&relation.provenance),
                );
            }
            if !ids.contains_key(&relation.from) {
                report.error(
                    DiagnosticCode::DanglingRelationSource,
                    format!("relationship source {} does not exist", relation.from),
                    Some(&relation.provenance),
                );
            }
            if !ids.contains_key(&relation.to) {
                report.error(
                    DiagnosticCode::DanglingRelationTarget,
                    format!("relationship target {} does not exist", relation.to),
                    Some(&relation.provenance),
                );
            }
        }

        for issue in self.network.verify_full_match(None).issues() {
            report.error(
                DiagnosticCode::NetworkVerification,
                format!(
                    "meta-language verification {:?} at link {}",
                    issue.kind(),
                    issue.link_id()
                ),
                None,
            );
        }
        report
    }

    /// Build the versioned interchange value.
    #[must_use]
    pub fn to_interchange(&self) -> InterchangeDocument {
        let mut concepts = self.concepts.values().cloned().collect::<Vec<_>>();
        for concept in &mut concepts {
            refresh_fingerprint(concept);
        }
        let mut relationships = self.relationships.clone();
        for relationship in &mut relationships {
            refresh_relationship_fingerprint(relationship);
        }
        InterchangeDocument {
            schema_version: self.schema_version,
            concepts,
            relationships,
            languages: self.languages.iter().cloned().collect(),
            allowlist: self.allowlist.iter().cloned().collect(),
        }
    }

    /// Serialize the public model as stable, pretty JSON.
    pub fn to_json_pretty(&self) -> Result<String, CatalogError> {
        Ok(serde_json::to_string_pretty(&self.to_interchange())?)
    }

    /// Serialize the generated public JSON Schema.
    pub fn json_schema_pretty() -> Result<String, CatalogError> {
        Ok(serde_json::to_string_pretty(&schemars::schema_for!(
            InterchangeDocument
        ))?)
    }

    /// Decode and validate a JSON interchange document.
    ///
    /// Input must match the generated JSON Schema before it reaches semantic
    /// catalog validation. Semantic validation constructs and verifies the
    /// normalized `meta-language` network.
    pub fn from_json(json: &str) -> Result<Self, CatalogError> {
        let value: serde_json::Value = serde_json::from_str(json)?;
        let schema = serde_json::to_value(schemars::schema_for!(InterchangeDocument))?;
        let validator = jsonschema::validator_for(&schema)
            .map_err(|error| CatalogError::Schema(error.to_string()))?;
        if let Err(error) = validator.validate(&value) {
            return Err(CatalogError::Schema(error.to_string()));
        }

        let document = serde_json::from_value(value)?;
        Self::from_interchange(document)
    }

    /// Construct and validate an ontology from the public contract.
    pub fn from_interchange(mut document: InterchangeDocument) -> Result<Self, CatalogError> {
        for concept in &mut document.concepts {
            if concept.id.is_empty() {
                concept.id = ConceptId::from_name(&concept.name);
            }
            refresh_fingerprint(concept);
        }
        let mut preflight = ValidationReport::default();
        let mut names = BTreeMap::<&str, &Concept>::new();
        let mut ids = BTreeMap::<&ConceptId, &Concept>::new();
        for concept in &document.concepts {
            if let Some(previous) = names.insert(&concept.name, concept) {
                preflight.error(
                    DiagnosticCode::DuplicateName,
                    format!(
                        "concepts {} and {} share canonical name {:?}",
                        previous.id, concept.id, concept.name
                    ),
                    Some(&concept.provenance),
                );
            }
            if let Some(previous) = ids.insert(&concept.id, concept) {
                preflight.error(
                    DiagnosticCode::DuplicateId,
                    format!(
                        "concepts {} and {} share identity {}",
                        previous.name, concept.name, concept.id
                    ),
                    Some(&concept.provenance),
                );
            }
        }
        if !preflight.is_valid() {
            return Err(CatalogError::Validation {
                count: preflight.diagnostics().len(),
                report: preflight,
            });
        }
        for relationship in &mut document.relationships {
            refresh_relationship_fingerprint(relationship);
        }
        let concepts = document
            .concepts
            .iter()
            .map(|concept| (concept.name.clone(), concept.clone()))
            .collect();
        let network_source = interchange_lino(&document);
        let ontology = Self {
            schema_version: document.schema_version,
            concepts,
            relationships: document.relationships,
            allowlist: document.allowlist.into_iter().collect(),
            languages: document.languages.into_iter().collect(),
            network: LinkNetwork::parse(&network_source, "lino", ParseConfiguration::default()),
        };
        let report = ontology.validate();
        if report.is_valid() {
            Ok(ontology)
        } else {
            Err(CatalogError::Validation {
                count: report.diagnostics().len(),
                report,
            })
        }
    }

    /// Produce a side-effect-free, deterministic import plan.
    #[must_use]
    pub fn plan_import(&self, candidate: &InterchangeDocument) -> ImportPlan {
        let mut current = self
            .concepts
            .values()
            .map(|concept| {
                (
                    (ImportObjectKind::Concept, concept.id.to_string()),
                    concept_fingerprint(concept),
                )
            })
            .collect::<BTreeMap<_, _>>();
        current.extend(self.relationships.iter().map(|relationship| {
            (
                (
                    ImportObjectKind::Relationship,
                    relationship_key(relationship),
                ),
                relationship_fingerprint(relationship),
            )
        }));
        let mut incoming = candidate
            .concepts
            .iter()
            .map(|concept| {
                let mut normalized = concept.clone();
                if normalized.id.is_empty() {
                    normalized.id = ConceptId::from_name(&normalized.name);
                }
                (
                    (ImportObjectKind::Concept, normalized.id.to_string()),
                    concept_fingerprint(&normalized),
                )
            })
            .collect::<BTreeMap<_, _>>();
        incoming.extend(candidate.relationships.iter().map(|relationship| {
            (
                (
                    ImportObjectKind::Relationship,
                    relationship_key(relationship),
                ),
                relationship_fingerprint(relationship),
            )
        }));
        let objects = current
            .keys()
            .chain(incoming.keys())
            .cloned()
            .collect::<BTreeSet<_>>();
        let changes = objects
            .into_iter()
            .map(|(object_kind, id)| {
                let key = (object_kind, id.clone());
                let before = current.get(&key).cloned();
                let after = incoming.get(&key).cloned();
                let kind = match (&before, &after) {
                    (None, Some(_)) => ChangeKind::Created,
                    (Some(_), None) => ChangeKind::Deprecated,
                    (Some(left), Some(right)) if left == right => ChangeKind::Unchanged,
                    (Some(_), Some(_)) => ChangeKind::Updated,
                    (None, None) => unreachable!("identity came from one input"),
                };
                ImportChange {
                    object_kind,
                    id,
                    kind,
                    before,
                    after,
                }
            })
            .collect();
        ImportPlan { changes }
    }

    /// Apply a full normalized import after validation.
    pub fn apply_import(&self, document: InterchangeDocument) -> Result<Self, CatalogError> {
        Self::from_interchange(document)
    }

    /// Search identity, name, label, aliases, and definitions.
    #[must_use]
    pub fn search(&self, query: &str, limit: usize) -> Vec<SearchMatch<'_>> {
        let query = query.trim().to_lowercase();
        if query.is_empty() || limit == 0 {
            return Vec::new();
        }
        let mut matches =
            self.concepts
                .values()
                .filter_map(|concept| {
                    let name = concept.name.to_lowercase();
                    let label = concept.label.to_lowercase();
                    let identity = concept.id.as_str().to_lowercase();
                    let mut score = 0;
                    if name == query || identity == query {
                        score += 100;
                    } else if name.contains(&query) || identity.contains(&query) {
                        score += 50;
                    }
                    if label == query {
                        score += 80;
                    } else if label.contains(&query) {
                        score += 40;
                    }
                    if concept
                        .allolexes
                        .iter()
                        .chain(&concept.aliases)
                        .any(|alias| alias.to_lowercase().contains(&query))
                    {
                        score += 30;
                    }
                    if concept.definitions.iter().any(|definition| {
                        definition.words.join(" ").to_lowercase().contains(&query)
                    }) {
                        score += 20;
                    }
                    (score > 0).then_some(SearchMatch { concept, score })
                })
                .collect::<Vec<_>>();
        matches.sort_by(|left, right| {
            right
                .score
                .cmp(&left.score)
                .then_with(|| left.concept.name.cmp(&right.concept.name))
        });
        matches.truncate(limit);
        matches
    }

    pub(crate) fn refresh_catalog_state(&mut self) {
        for concept in self.concepts.values_mut() {
            if concept.id.is_empty() {
                concept.id = ConceptId::from_name(&concept.name);
            }
            refresh_fingerprint(concept);
        }
        for relationship in &mut self.relationships {
            refresh_relationship_fingerprint(relationship);
        }
    }

    pub(crate) fn rebuild_network(&mut self) {
        let source = interchange_lino(&self.to_interchange());
        self.network = LinkNetwork::parse(&source, "lino", ParseConfiguration::default());
    }
}

fn concept_fingerprint(concept: &Concept) -> String {
    let mut normalized = concept.clone();
    normalized.provenance.fingerprint.clear();
    let bytes = serde_json::to_vec(&normalized).expect("Concept serialization is infallible");
    format!("{:x}", Sha256::digest(bytes))
}

fn refresh_fingerprint(concept: &mut Concept) {
    concept.provenance.fingerprint = concept_fingerprint(concept);
}

fn relationship_key(relationship: &Relationship) -> String {
    format!(
        "{}:{}:{}",
        relationship.from, relationship.kind, relationship.to
    )
}

fn relationship_fingerprint(relationship: &Relationship) -> String {
    let mut normalized = relationship.clone();
    normalized.provenance.fingerprint.clear();
    let bytes = serde_json::to_vec(&normalized).expect("Relationship serialization is infallible");
    format!("{:x}", Sha256::digest(bytes))
}

fn refresh_relationship_fingerprint(relationship: &mut Relationship) {
    relationship.provenance.fingerprint = relationship_fingerprint(relationship);
}

fn interchange_lino(document: &InterchangeDocument) -> String {
    let mut output = format!("(dataset (schema_version {}))\n", document.schema_version);
    for concept in &document.concepts {
        output.push('(');
        output.push_str(&concept.name);
        output.push_str(" (id ");
        output.push_str(concept.id.as_str());
        output.push_str("))\n");
    }
    for relation in &document.relationships {
        output.push_str("(relation ");
        output.push_str(relation.from.as_str());
        output.push(' ');
        output.push_str(&relation.kind);
        output.push(' ');
        output.push_str(relation.to.as_str());
        output.push_str(")\n");
    }
    output
}
