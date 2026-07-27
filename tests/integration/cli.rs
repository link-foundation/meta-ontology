use std::path::PathBuf;
use std::process::Command;

fn bin() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("target");
    p.push("debug");
    p.push(if cfg!(windows) {
        "meta-ontology.exe"
    } else {
        "meta-ontology"
    });
    p
}

fn ensure_built() -> PathBuf {
    let path = bin();
    if !path.exists() {
        let status = Command::new("cargo")
            .args(["build", "--bin", "meta-ontology"])
            .status()
            .expect("cargo build");
        assert!(status.success(), "cargo build failed");
    }
    path
}

#[test]
fn list_prints_concepts() {
    let output = Command::new(ensure_built())
        .arg("list")
        .output()
        .expect("run");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("thing"));
    assert!(stdout.contains("concept"));
}

#[test]
fn show_prints_concept_detail() {
    let output = Command::new(ensure_built())
        .args(["show", "thing"])
        .output()
        .expect("run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("name:"));
    assert!(stdout.contains("definitions:"));
}

#[test]
fn check_words_passes_on_known_word() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(dir.path().join("good.md"), "thing concept link").unwrap();
    let output = Command::new(ensure_built())
        .args(["check-words", dir.path().to_str().unwrap()])
        .output()
        .expect("run");
    assert!(
        output.status.success(),
        "stderr: {}\nstdout: {}",
        String::from_utf8_lossy(&output.stderr),
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn check_words_fails_on_unknown_word() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(dir.path().join("bad.md"), "totallyimaginarywordxyz").unwrap();
    let output = Command::new(ensure_built())
        .args(["check-words", dir.path().to_str().unwrap()])
        .output()
        .expect("run");
    assert!(!output.status.success(), "expected failure");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("unknown"));
}

#[test]
fn validate_reports_catalog_contract_summary() {
    let output = Command::new(ensure_built())
        .arg("validate")
        .output()
        .expect("run");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("valid: schema v1"));
    assert!(stdout.contains("relationships"));
}

#[test]
fn export_json_emits_versioned_entities_and_relationships() {
    let output = Command::new(ensure_built())
        .arg("export-json")
        .output()
        .expect("run");
    assert!(output.status.success());
    let document: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid JSON output");
    assert_eq!(document["schema_version"], 1);
    assert!(document["concepts"]
        .as_array()
        .is_some_and(|items| !items.is_empty()));
    assert!(document["relationships"]
        .as_array()
        .is_some_and(|items| !items.is_empty()));
}

#[test]
fn json_schema_command_emits_a_compilable_schema() {
    let output = Command::new(ensure_built())
        .arg("json-schema")
        .output()
        .expect("run");
    assert!(output.status.success());
    let schema: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid JSON Schema output");
    jsonschema::validator_for(&schema).expect("compilable JSON Schema");
}

#[test]
fn plan_import_emits_deterministic_ndjson_without_applying_changes() {
    let dir = tempfile::tempdir().expect("tempdir");
    let export = Command::new(ensure_built())
        .arg("export-json")
        .output()
        .expect("export");
    assert!(export.status.success());
    let path = dir.path().join("candidate.json");
    std::fs::write(&path, export.stdout).expect("write candidate");

    let output = Command::new(ensure_built())
        .args([
            "plan-import",
            path.to_str().expect("UTF-8 fixture path"),
            "--format",
            "ndjson",
        ])
        .output()
        .expect("plan");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        let change: serde_json::Value = serde_json::from_str(line).expect("NDJSON change");
        assert_eq!(change["kind"], "unchanged");
    }
}

#[test]
fn search_prints_ranked_stable_identity() {
    let output = Command::new(ensure_built())
        .args(["search", "body or not a body", "--limit", "1"])
        .output()
        .expect("run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("meta-ontology:thing"));
}
