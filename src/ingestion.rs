//! Small, source-neutral ingestion contracts.
//!
//! Connectors only discover raw records and checkpoints. Normalization,
//! validation, dry-run planning, and application remain catalog concerns, so a
//! second connector does not add source-specific fields to the core model.

use std::fmt;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::catalog::{
    CatalogError, ConceptId, Governance, ImportPlan, InterchangeDocument, LifecycleState,
    Provenance, Relationship, ReviewState,
};
use crate::ontology::{Concept, Definition, Ontology};

/// Typed connector settings shared by source implementations.
#[derive(Clone)]
pub struct ConnectorConfig {
    /// Stable connector name used as the provenance agent.
    pub name: String,
    /// URI identifying the external source.
    pub source_uri: String,
    /// License or usage note inherited by normalized records.
    pub license: String,
    /// RFC 3339 observation time for this fixture or extraction.
    pub observed_at: String,
    secret: Option<String>,
}

impl ConnectorConfig {
    /// Creates non-secret connector configuration.
    #[must_use]
    pub fn new(
        name: impl Into<String>,
        source_uri: impl Into<String>,
        license: impl Into<String>,
        observed_at: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            source_uri: source_uri.into(),
            license: license.into(),
            observed_at: observed_at.into(),
            secret: None,
        }
    }

    /// Attaches a credential that is always redacted from debug reports.
    #[must_use]
    pub fn with_secret(mut self, secret: impl Into<String>) -> Self {
        self.secret = Some(secret.into());
        self
    }
}

impl fmt::Debug for ConnectorConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ConnectorConfig")
            .field("name", &self.name)
            .field("source_uri", &self.source_uri)
            .field("license", &self.license)
            .field("observed_at", &self.observed_at)
            .field("secret", &self.secret.as_ref().map(|_| "[REDACTED]"))
            .finish()
    }
}

/// Capabilities declared before a connector run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectorCapabilities {
    /// Normalized record kinds the connector can emit.
    pub record_types: Vec<String>,
    /// Whether extraction can resume from a checkpoint.
    pub checkpointing: bool,
    /// Whether malformed records can be quarantined while valid records proceed.
    pub partial_failures: bool,
}

/// Opaque-enough resume position for the fixture connector.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Checkpoint {
    /// First source offset not yet read.
    pub next_offset: usize,
    /// Fingerprint of the source population used to reject stale checkpoints.
    pub source_version: String,
}

/// One source record with stable extraction evidence.
#[derive(Debug, Clone)]
pub struct RawRecord {
    /// Zero-based source offset.
    pub offset: usize,
    /// Source payload kept outside the catalog until normalized.
    pub payload: serde_json::Value,
}

/// Records and the checkpoint to persist after processing them.
#[derive(Debug, Clone)]
pub struct SourceBatch {
    /// Extracted records.
    pub records: Vec<RawRecord>,
    /// Resume position after this batch.
    pub checkpoint: Checkpoint,
}

/// Whether retrying the same operation may succeed without data correction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RetryClass {
    /// Retry may succeed, for example after restarting from a fresh checkpoint.
    Transient,
    /// The source record must be corrected or deliberately skipped.
    Permanent,
}

/// Structured extraction or normalization error.
#[derive(Debug, Clone, Error, PartialEq, Eq, Serialize, Deserialize)]
#[error("{message} ({source_uri}, record {record_offset:?})")]
pub struct IngestionError {
    /// Source URI safe to include in reports.
    pub source_uri: String,
    /// Source offset when the error belongs to one record.
    pub record_offset: Option<usize>,
    /// Machine-actionable retry classification.
    pub retry: RetryClass,
    /// Human-readable problem detail. Secrets and raw payloads are excluded.
    pub message: String,
}

impl IngestionError {
    fn source(
        config: &ConnectorConfig,
        record_offset: Option<usize>,
        retry: RetryClass,
        message: impl Into<String>,
    ) -> Self {
        Self {
            source_uri: config.source_uri.clone(),
            record_offset,
            retry,
            message: message.into(),
        }
    }
}

/// Minimal connector boundary: declare, configure, and read.
pub trait SourceConnector {
    /// Non-secret configuration for provenance and reporting.
    fn configuration(&self) -> &ConnectorConfig;
    /// Supported extraction behavior.
    fn capabilities(&self) -> ConnectorCapabilities;
    /// Read the next batch, optionally resuming from a prior checkpoint.
    fn read(&mut self, checkpoint: Option<&Checkpoint>) -> Result<SourceBatch, IngestionError>;
}

/// Deterministic connector used to prove the complete ingestion contract.
#[derive(Clone)]
pub struct FixtureConnector {
    configuration: ConnectorConfig,
    records: Vec<serde_json::Value>,
    batch_size: usize,
    source_version: String,
}

impl FixtureConnector {
    /// Creates a connector over stable JSON fixture records.
    #[must_use]
    pub fn new(configuration: ConnectorConfig, records: Vec<serde_json::Value>) -> Self {
        let bytes = serde_json::to_vec(&records).expect("serializing JSON values cannot fail");
        let source_version = format!("{:x}", Sha256::digest(bytes));
        Self {
            configuration,
            records,
            batch_size: usize::MAX,
            source_version,
        }
    }

    /// Limits extraction to a deterministic number of records per batch.
    #[must_use]
    pub fn with_batch_size(mut self, batch_size: usize) -> Self {
        self.batch_size = batch_size.max(1);
        self
    }
}

impl SourceConnector for FixtureConnector {
    fn configuration(&self) -> &ConnectorConfig {
        &self.configuration
    }

    fn capabilities(&self) -> ConnectorCapabilities {
        ConnectorCapabilities {
            record_types: vec!["concept".to_string(), "relationship".to_string()],
            checkpointing: true,
            partial_failures: true,
        }
    }

    fn read(&mut self, checkpoint: Option<&Checkpoint>) -> Result<SourceBatch, IngestionError> {
        let offset = checkpoint.map_or(0, |value| value.next_offset);
        if let Some(checkpoint) = checkpoint {
            if checkpoint.source_version != self.source_version {
                return Err(IngestionError::source(
                    &self.configuration,
                    None,
                    RetryClass::Transient,
                    "checkpoint belongs to a different fixture version",
                ));
            }
        }
        if offset > self.records.len() {
            return Err(IngestionError::source(
                &self.configuration,
                None,
                RetryClass::Permanent,
                "checkpoint offset is beyond the source",
            ));
        }
        let end = offset
            .saturating_add(self.batch_size)
            .min(self.records.len());
        let records = self.records[offset..end]
            .iter()
            .cloned()
            .enumerate()
            .map(|(index, payload)| RawRecord {
                offset: offset + index,
                payload,
            })
            .collect();
        Ok(SourceBatch {
            records,
            checkpoint: Checkpoint {
                next_offset: end,
                source_version: self.source_version.clone(),
            },
        })
    }
}

/// Side-effect-free result of extracting, normalizing, and validating a batch.
#[derive(Debug)]
pub struct IngestionReport {
    /// Full normalized candidate suitable for [`Ontology::apply_import`].
    pub document: InterchangeDocument,
    /// Deterministic dry-run change decisions.
    pub plan: ImportPlan,
    /// Quarantined record errors.
    pub errors: Vec<IngestionError>,
    /// Checkpoint to persist after handling the batch.
    pub checkpoint: Checkpoint,
}

/// Runs one connector batch without mutating the current ontology.
pub fn run_connector(
    ontology: &Ontology,
    connector: &mut impl SourceConnector,
    checkpoint: Option<&Checkpoint>,
) -> Result<IngestionReport, IngestionError> {
    let configuration = connector.configuration().clone();
    let batch = connector.read(checkpoint)?;
    let mut document = ontology.to_interchange();
    let mut errors = Vec::new();

    for raw in batch.records {
        match normalize(&configuration, &raw) {
            Ok(record) => {
                let mut candidate = document.clone();
                apply_record(&mut candidate, record);
                match Ontology::from_interchange(candidate.clone()) {
                    Ok(_) => document = candidate,
                    Err(error) => errors.push(IngestionError::source(
                        &configuration,
                        Some(raw.offset),
                        RetryClass::Permanent,
                        catalog_error_message(&error),
                    )),
                }
            }
            Err(message) => errors.push(IngestionError::source(
                &configuration,
                Some(raw.offset),
                RetryClass::Permanent,
                message,
            )),
        }
    }

    let plan = ontology.plan_import(&document);
    Ok(IngestionReport {
        document,
        plan,
        errors,
        checkpoint: batch.checkpoint,
    })
}

#[derive(Debug, Deserialize)]
#[serde(tag = "record_type", rename_all = "snake_case")]
enum IncomingRecord {
    Concept {
        id: String,
        name: String,
        #[serde(default)]
        label: String,
        #[serde(default)]
        origin: String,
        #[serde(default)]
        category: String,
        #[serde(default)]
        allolexes: Vec<String>,
        #[serde(default)]
        aliases: Vec<String>,
        #[serde(default)]
        definitions: Vec<Vec<String>>,
        #[serde(default)]
        owners: Vec<String>,
        #[serde(default)]
        classifications: Vec<String>,
        #[serde(default)]
        tags: Vec<String>,
        #[serde(default)]
        lifecycle: LifecycleState,
    },
    Relationship {
        from: String,
        kind: String,
        to: String,
    },
}

enum NormalizedRecord {
    Concept(Box<Concept>),
    Relationship(Box<Relationship>),
}

fn normalize(configuration: &ConnectorConfig, raw: &RawRecord) -> Result<NormalizedRecord, String> {
    let record: IncomingRecord =
        serde_json::from_value(raw.payload.clone()).map_err(|error| error.to_string())?;
    let provenance = connector_provenance(configuration, raw.offset);
    match record {
        IncomingRecord::Concept {
            id,
            name,
            label,
            origin,
            category,
            allolexes,
            aliases,
            definitions,
            owners,
            classifications,
            tags,
            lifecycle,
        } => {
            if id.trim().is_empty() || name.trim().is_empty() {
                return Err("concept id and name must not be empty".to_string());
            }
            Ok(NormalizedRecord::Concept(Box::new(Concept {
                id: ConceptId::new(id),
                name,
                label,
                origin,
                category,
                allolexes,
                aliases,
                definitions: definitions
                    .into_iter()
                    .map(|words| Definition { words })
                    .collect(),
                mappings: Vec::new(),
                exponents: std::collections::BTreeMap::new(),
                provenance,
                governance: Governance {
                    owners,
                    classifications,
                    tags,
                    lifecycle,
                },
            })))
        }
        IncomingRecord::Relationship { from, kind, to } => {
            if from.trim().is_empty() || kind.trim().is_empty() || to.trim().is_empty() {
                return Err("relationship endpoints and kind must not be empty".to_string());
            }
            Ok(NormalizedRecord::Relationship(Box::new(Relationship {
                from: ConceptId::new(from),
                kind,
                to: ConceptId::new(to),
                provenance,
            })))
        }
    }
}

fn connector_provenance(configuration: &ConnectorConfig, offset: usize) -> Provenance {
    Provenance {
        source_uri: configuration.source_uri.clone(),
        source_line: offset + 1,
        source_record: offset.to_string(),
        observed_at: configuration.observed_at.clone(),
        agent: format!("connector:{}", configuration.name),
        review_state: ReviewState::Draft,
        license: configuration.license.clone(),
        fingerprint: String::new(),
    }
}

fn apply_record(document: &mut InterchangeDocument, record: NormalizedRecord) {
    match record {
        NormalizedRecord::Concept(concept) => {
            let concept = *concept;
            if let Some(existing) = document
                .concepts
                .iter_mut()
                .find(|existing| existing.id == concept.id)
            {
                *existing = concept;
            } else {
                document.concepts.push(concept);
                document
                    .concepts
                    .sort_by(|left, right| left.name.cmp(&right.name));
            }
        }
        NormalizedRecord::Relationship(relationship) => {
            let relationship = *relationship;
            if let Some(existing) = document.relationships.iter_mut().find(|existing| {
                existing.from == relationship.from
                    && existing.kind == relationship.kind
                    && existing.to == relationship.to
            }) {
                *existing = relationship;
            } else {
                document.relationships.push(relationship);
                document.relationships.sort_by(|left, right| {
                    (&left.from, &left.kind, &left.to).cmp(&(&right.from, &right.kind, &right.to))
                });
            }
        }
    }
}

fn catalog_error_message(error: &CatalogError) -> String {
    match error {
        CatalogError::Validation { report, .. } => report
            .diagnostics()
            .iter()
            .map(|diagnostic| format!("{} {}", diagnostic.code, diagnostic.message))
            .collect::<Vec<_>>()
            .join("; "),
        _ => error.to_string(),
    }
}
