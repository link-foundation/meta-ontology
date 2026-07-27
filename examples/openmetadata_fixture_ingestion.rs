//! Runs the source-neutral ingestion pipeline with OpenMetadata-shaped fixtures.

use meta_ontology::ingestion::{run_connector, ConnectorConfig, FixtureConnector};
use meta_ontology::loader::load_default;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ontology = load_default()?;
    let configuration = ConnectorConfig::new(
        "openmetadata-fixture",
        "fixture://openmetadata/tables",
        "CC0-1.0",
        "2026-07-27T00:00:00Z",
    );
    let records = vec![
        serde_json::json!({
            "record_type": "concept",
            "id": "openmetadata:table",
            "name": "table",
            "label": "Table",
            "owners": ["metadata-team"],
            "classifications": ["internal"],
            "tags": ["catalog"]
        }),
        serde_json::json!({
            "record_type": "relationship",
            "from": "openmetadata:table",
            "kind": "related",
            "to": "meta-ontology:concept"
        }),
    ];
    let mut connector = FixtureConnector::new(configuration, records);
    let report = run_connector(&ontology, &mut connector, None)?;

    print!("{}", report.plan.to_ndjson()?);
    eprintln!(
        "checkpoint={}, quarantined={}",
        report.checkpoint.next_offset,
        report.errors.len()
    );
    Ok(())
}
