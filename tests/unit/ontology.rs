use std::path::PathBuf;

use meta_ontology::loader::load_from_dir;

fn data_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data")
}

#[test]
fn loader_finds_seed_concepts() {
    let o = load_from_dir(data_dir()).expect("load");
    assert!(o.len() > 80, "expected > 80 concepts, got {}", o.len());
    assert!(o.find("thing").is_some());
    assert!(o.find("link").is_some());
    assert!(o.find("concept").is_some());
}

#[test]
fn cycle_thing_concept_thing_present() {
    let o = load_from_dir(data_dir()).expect("load");
    let from_thing: Vec<&str> = o.neighbors("thing").map(|c| c.name.as_str()).collect();
    assert!(from_thing.contains(&"concept"), "{from_thing:?}");
    let from_concept: Vec<&str> = o.neighbors("concept").map(|c| c.name.as_str()).collect();
    assert!(from_concept.contains(&"thing"), "{from_concept:?}");
}

#[test]
fn nsm_count_at_least_64_primes() {
    // 65 canonical primes plus the meta `nsm` entry. Some primes (the
    // multi-word ones) are stored under canonical underscored names, so
    // we count by `origin == nsm`.
    let o = load_from_dir(data_dir()).expect("load");
    let count = o
        .names()
        .filter(|n| {
            o.find(n)
                .is_some_and(|c| c.origin == "nsm" && c.name != "nsm")
        })
        .count();
    assert!(count >= 64, "expected >= 64 NSM primes, got {count}");
}
