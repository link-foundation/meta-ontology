use meta_ontology::catalog::ChangeKind;
use meta_ontology::ingestion::{
    run_connector, ConnectorConfig, FixtureConnector, RetryClass, SourceConnector,
};
use meta_ontology::loader::load_default;

#[test]
fn fixture_connector_quarantines_bad_records_and_resumes_from_checkpoint() {
    let base = load_default().expect("load seed ontology");
    let config = ConnectorConfig::new(
        "openmetadata-fixture",
        "fixture://openmetadata",
        "CC0-1.0",
        "2026-07-27T00:00:00Z",
    )
    .with_secret("must-not-leak");
    assert!(!format!("{config:?}").contains("must-not-leak"));

    let records = vec![
        serde_json::json!({
            "record_type": "concept",
            "id": "openmetadata:table",
            "name": "table",
            "label": "Table",
            "owners": ["metadata-team"],
            "classifications": ["internal"]
        }),
        serde_json::json!({
            "record_type": "concept",
            "id": "",
            "label": "Missing a canonical name"
        }),
        serde_json::json!({
            "record_type": "relationship",
            "from": "openmetadata:table",
            "kind": "related",
            "to": "meta-ontology:concept"
        }),
    ];
    let mut connector = FixtureConnector::new(config.clone(), records.clone()).with_batch_size(2);
    let capabilities = connector.capabilities();
    assert!(capabilities.checkpointing);
    assert!(capabilities.partial_failures);

    let first = run_connector(&base, &mut connector, None).expect("first batch");
    assert_eq!(first.errors.len(), 1);
    assert_eq!(first.errors[0].retry, RetryClass::Permanent);
    assert_eq!(first.checkpoint.next_offset, 2);
    let checkpoint_json = serde_json::to_string(&first.checkpoint).expect("serialize checkpoint");
    let restored: meta_ontology::ingestion::Checkpoint =
        serde_json::from_str(&checkpoint_json).expect("deserialize checkpoint");
    assert_eq!(restored, first.checkpoint);
    assert!(first
        .plan
        .changes()
        .iter()
        .any(|change| change.kind == ChangeKind::Created));

    let after_first = base
        .apply_import(first.document)
        .expect("apply valid records from first batch");
    let second =
        run_connector(&after_first, &mut connector, Some(&first.checkpoint)).expect("resume");
    assert!(second.errors.is_empty());
    assert_eq!(second.checkpoint.next_offset, 3);
    let after_second = after_first
        .apply_import(second.document)
        .expect("apply relationship batch");
    assert!(after_second.relationships().any(|relationship| {
        relationship.from.as_str() == "openmetadata:table"
            && relationship.to.as_str() == "meta-ontology:concept"
    }));

    let mut replay = FixtureConnector::new(config, records);
    let replay_report = run_connector(&after_second, &mut replay, None).expect("replay");
    assert_eq!(replay_report.errors.len(), 1);
    assert!(replay_report
        .plan
        .changes()
        .iter()
        .all(|change| change.kind == ChangeKind::Unchanged));

    let mut changed_source = FixtureConnector::new(
        ConnectorConfig::new(
            "openmetadata-fixture",
            "fixture://openmetadata",
            "CC0-1.0",
            "2026-07-27T00:00:00Z",
        ),
        vec![serde_json::json!({
            "record_type": "concept",
            "id": "openmetadata:changed",
            "name": "changed"
        })],
    );
    let stale = run_connector(&after_second, &mut changed_source, Some(&first.checkpoint))
        .expect_err("stale checkpoint");
    assert_eq!(stale.retry, RetryClass::Transient);
}
