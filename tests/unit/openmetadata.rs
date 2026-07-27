use std::path::PathBuf;

use meta_ontology::catalog::{
    CatalogError, ChangeKind, DiagnosticCode, ImportObjectKind, InterchangeDocument,
};
use meta_ontology::loader::{load_default, load_from_dir};
use meta_ontology::Ontology;

fn data_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data")
}

#[test]
fn ontology_is_backed_by_a_verified_meta_language_network() {
    let ontology = load_default().expect("load seed ontology");

    assert!(ontology.network().verify_full_match(None).is_clean());
    assert!(ontology.network().find_term("thing").is_some());

    let snapshot = ontology.snapshot();
    assert_eq!(snapshot.version(), u64::from(ontology.schema_version()));
    assert_eq!(snapshot.provenance(), "meta-ontology catalog");
}

#[test]
fn seed_data_has_stable_identity_typed_relations_and_provenance() {
    let ontology = load_default().expect("load seed ontology");
    let thing = ontology.find("thing").expect("thing");

    assert_eq!(thing.id.as_str(), "meta-ontology:thing");
    assert_eq!(thing.provenance.license, "Unlicense");
    assert!(!thing.provenance.fingerprint.is_empty());
    assert!(thing
        .governance
        .owners
        .iter()
        .any(|owner| owner == "link_foundation"));

    let relation = ontology
        .relationships()
        .find(|relation| relation.from == thing.id)
        .expect("typed relation from thing");
    assert_eq!(relation.to.as_str(), "meta-ontology:concept");
}

#[test]
fn validator_reports_stable_codes_for_contract_violations() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(
        dir.path().join("invalid.lino"),
        "\
(dataset (schema_version 999))
(alpha (id shared) (alias duplicate))
(beta (id shared) (alias duplicate))
(gamma (id other) (alias duplicate))
(relation alpha unsupported missing)
",
    )
    .expect("write fixture");

    let ontology = load_from_dir(dir.path()).expect("syntactically valid fixture");
    let report = ontology.validate();
    let codes = report
        .diagnostics()
        .iter()
        .map(|diagnostic| diagnostic.code)
        .collect::<Vec<_>>();

    assert!(codes.contains(&DiagnosticCode::UnsupportedSchemaVersion));
    assert!(codes.contains(&DiagnosticCode::DuplicateId));
    assert!(
        codes.contains(&DiagnosticCode::AmbiguousAlias),
        "codes: {codes:?}; alpha aliases: {:?}; beta aliases: {:?}",
        ontology.find("alpha").map(|concept| &concept.aliases),
        ontology.find("beta").map(|concept| &concept.aliases)
    );
    assert!(codes.contains(&DiagnosticCode::UnsupportedRelationKind));
    assert!(codes.contains(&DiagnosticCode::DanglingRelationTarget));

    let json = serde_json::to_value(&report).expect("serialize diagnostics");
    assert!(json["diagnostics"]
        .as_array()
        .expect("diagnostic array")
        .iter()
        .any(|diagnostic| diagnostic["code"] == "MO001"));
}

#[test]
fn json_interchange_round_trip_preserves_catalog_contracts() {
    let ontology = load_default().expect("load seed ontology");
    let json = ontology.to_json_pretty().expect("serialize");
    let document: InterchangeDocument = serde_json::from_str(&json).expect("schema-shaped JSON");
    let restored = Ontology::from_json(&json).expect("deserialize");

    assert_eq!(document.schema_version, ontology.schema_version());
    assert_eq!(restored.len(), ontology.len());
    assert_eq!(
        restored.find("thing").map(|concept| &concept.id),
        ontology.find("thing").map(|concept| &concept.id)
    );
    assert_eq!(
        restored.relationships().count(),
        ontology.relationships().count()
    );
    assert!(restored.validate().is_valid());
}

#[test]
fn interchange_rejects_duplicate_names_before_map_normalization() {
    let ontology = load_default().expect("load seed ontology");
    let mut document = ontology.to_interchange();
    let mut duplicate = document
        .concepts
        .iter()
        .find(|concept| concept.name == "thing")
        .expect("thing")
        .clone();
    duplicate.id = meta_ontology::ConceptId::new("fixture:duplicate-thing");
    document.concepts.push(duplicate);

    let error = Ontology::from_interchange(document).expect_err("duplicate canonical name");
    let CatalogError::Validation { report, .. } = error else {
        panic!("expected semantic validation error");
    };
    assert!(report
        .diagnostics()
        .iter()
        .any(|diagnostic| diagnostic.code == DiagnosticCode::DuplicateName));
}

#[test]
fn generated_json_schema_accepts_exports_and_rejects_missing_contract_fields() {
    let ontology = load_default().expect("load seed ontology");
    let schema_json = Ontology::json_schema_pretty().expect("generate schema");
    let schema: serde_json::Value = serde_json::from_str(&schema_json).expect("schema JSON");
    let validator = jsonschema::validator_for(&schema).expect("compile schema");
    let export: serde_json::Value =
        serde_json::from_str(&ontology.to_json_pretty().expect("export")).expect("export JSON");

    validator.validate(&export).expect("export matches schema");

    let missing_relationships = serde_json::json!({
        "schema_version": 1,
        "concepts": [],
        "languages": [],
        "allowlist": []
    });
    assert!(validator.validate(&missing_relationships).is_err());
    assert!(Ontology::from_json(&missing_relationships.to_string()).is_err());
}

#[test]
fn import_plan_is_deterministic_idempotent_and_applies_updates() {
    let ontology = load_default().expect("load seed ontology");
    let unchanged_document = ontology.to_interchange();
    let unchanged = ontology.plan_import(&unchanged_document);
    assert!(unchanged
        .changes()
        .iter()
        .all(|change| change.kind == ChangeKind::Unchanged));

    let mut changed_document = unchanged_document;
    let thing = changed_document
        .concepts
        .iter_mut()
        .find(|concept| concept.name == "thing")
        .expect("thing");
    thing.label = "Stable Thing".to_string();
    thing.provenance.source_uri = "fixture://openmetadata-import".to_string();

    let plan = ontology.plan_import(&changed_document);
    assert_eq!(
        plan.changes()
            .iter()
            .filter(|change| change.kind == ChangeKind::Updated)
            .count(),
        1
    );
    let relationship = changed_document
        .relationships
        .first_mut()
        .expect("seed relationship");
    relationship.provenance.observed_at = "2026-07-27T00:00:00Z".to_string();
    let relationship_plan = ontology.plan_import(&changed_document);
    assert!(relationship_plan.changes().iter().any(|change| {
        change.object_kind == ImportObjectKind::Relationship && change.kind == ChangeKind::Updated
    }));
    let ndjson = plan.to_ndjson().expect("serialize change plan");
    assert_eq!(ndjson.lines().count(), plan.changes().len());

    let updated = ontology
        .apply_import(changed_document.clone())
        .expect("apply validated import");
    assert_eq!(
        updated.find("thing").map(|concept| concept.label.as_str()),
        Some("Stable Thing")
    );
    assert!(updated
        .plan_import(&changed_document)
        .changes()
        .iter()
        .all(|change| change.kind == ChangeKind::Unchanged));
}

#[test]
fn import_plan_normalizes_legacy_empty_ids_before_fingerprinting() {
    let ontology = load_default().expect("load seed ontology");
    let mut legacy_document = ontology.to_interchange();
    legacy_document
        .concepts
        .iter_mut()
        .find(|concept| concept.name == "thing")
        .expect("thing")
        .id = meta_ontology::ConceptId::default();

    assert!(ontology
        .plan_import(&legacy_document)
        .changes()
        .iter()
        .all(|change| change.kind == ChangeKind::Unchanged));
}

#[test]
fn canonical_name_can_change_without_changing_identity_or_edge_endpoints() {
    let ontology = load_default().expect("load seed ontology");
    let mut document = ontology.to_interchange();
    let thing = document
        .concepts
        .iter_mut()
        .find(|concept| concept.name == "thing")
        .expect("thing");
    thing.name = "renamed_thing".to_string();

    let renamed = Ontology::from_interchange(document).expect("rename with stable ID");
    assert_eq!(
        renamed
            .find("renamed_thing")
            .map(|concept| concept.id.as_str()),
        Some("meta-ontology:thing")
    );
    assert!(renamed.relationships().any(|relationship| {
        relationship.from.as_str() == "meta-ontology:thing"
            || relationship.to.as_str() == "meta-ontology:thing"
    }));
}

#[test]
fn search_matches_identity_labels_aliases_and_definitions() {
    let ontology = load_from_dir(data_dir()).expect("load");
    let matches = ontology.search("body or not a body", 5);

    assert!(!matches.is_empty());
    assert_eq!(matches[0].concept.name, "thing");
    assert!(matches[0].score > 0);
}
