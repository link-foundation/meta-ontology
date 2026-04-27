use std::io::Write;

use meta_ontology::loader::load_default;
use meta_ontology::words::scan_path;

#[test]
fn scan_known_word_passes() {
    let dir = tempdir();
    let mut f = std::fs::File::create(dir.path().join("ok.md")).unwrap();
    writeln!(f, "thing concept link").unwrap();
    let o = load_default().expect("load");
    let result = scan_path(dir.path(), &o).expect("scan");
    assert!(result.is_empty(), "{result:?}");
}

#[test]
fn scan_unknown_word_reports_it() {
    let dir = tempdir();
    let mut f = std::fs::File::create(dir.path().join("bad.md")).unwrap();
    writeln!(f, "totallyimaginarywordxyz appears here").unwrap();
    let o = load_default().expect("load");
    let result = scan_path(dir.path(), &o).expect("scan");
    let words: Vec<&str> = result.iter().map(|u| u.word.as_str()).collect();
    assert!(
        words.contains(&"totallyimaginarywordxyz"),
        "expected unknown word, got {words:?}"
    );
}

#[test]
fn scan_skips_code_blocks() {
    let dir = tempdir();
    let mut f = std::fs::File::create(dir.path().join("with-code.md")).unwrap();
    writeln!(f, "Plain word: thing.").unwrap();
    writeln!(f, "```").unwrap();
    writeln!(f, "imaginarytokenxyz").unwrap();
    writeln!(f, "```").unwrap();
    let o = load_default().expect("load");
    let result = scan_path(dir.path(), &o).expect("scan");
    let words: Vec<&str> = result.iter().map(|u| u.word.as_str()).collect();
    assert!(
        !words.contains(&"imaginarytokenxyz"),
        "code-block word should have been skipped, got {words:?}"
    );
}

fn tempdir() -> tempfile::TempDir {
    tempfile::Builder::new()
        .prefix("meta-ontology-")
        .tempdir()
        .expect("tempdir")
}
