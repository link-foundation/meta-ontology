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
