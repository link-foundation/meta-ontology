//! Load `.lino` files from disk into an [`Ontology`].
//!
//! The loader walks a directory recursively, parses every `*.lino` file
//! with `links-notation`, and folds each top-level link into the
//! ontology. Cycles are allowed and never rejected.

use std::path::{Path, PathBuf};

use links_notation::{parse_lino_to_links, LiNo};
use thiserror::Error;
use walkdir::WalkDir;

use crate::catalog::{ConceptId, LifecycleState, Provenance, Relationship, ReviewState};
use crate::ontology::{Definition, Mapping, Ontology};

/// Errors returned by [`load_from_dir`] and friends.
#[derive(Debug, Error)]
pub enum LoadError {
    #[error("io error reading {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("walk error: {0}")]
    Walk(#[from] walkdir::Error),
    #[error("parse error in {path}: {message}")]
    Parse { path: PathBuf, message: String },
}

/// Load the in‑repo `data/` folder relative to the current crate
/// (works from `cargo test` and `cargo run`).
pub fn load_default() -> Result<Ontology, LoadError> {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    load_from_dir(Path::new(manifest_dir).join("data"))
}

/// Load every `*.lino` file under `dir` (recursively) into one ontology.
pub fn load_from_dir<P: AsRef<Path>>(dir: P) -> Result<Ontology, LoadError> {
    let mut ontology = Ontology::default();
    let mut paths = Vec::new();
    for entry in WalkDir::new(dir.as_ref()) {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("lino") {
            continue;
        }
        paths.push(path.to_path_buf());
    }
    paths.sort();
    for path in paths {
        absorb_file(&path, &mut ontology)?;
    }
    ontology.refresh_catalog_state();
    ontology.rebuild_network();
    Ok(ontology)
}

/// Load a single `.lino` file into the ontology.
pub fn load_file(path: &Path, ontology: &mut Ontology) -> Result<(), LoadError> {
    absorb_file(path, ontology)?;
    ontology.refresh_catalog_state();
    ontology.rebuild_network();
    Ok(())
}

fn absorb_file(path: &Path, ontology: &mut Ontology) -> Result<(), LoadError> {
    let text = std::fs::read_to_string(path).map_err(|source| LoadError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let links = parse_lino_to_links(&text).map_err(|e| LoadError::Parse {
        path: path.to_path_buf(),
        message: format!("{e}"),
    })?;

    let source_uri = path.to_string_lossy();
    let mut search_from = 0;
    for link in links {
        let source_line = source_line(&text, &link, &mut search_from);
        absorb_top_level(link, ontology, &source_uri, source_line);
    }
    Ok(())
}

fn absorb_top_level(
    node: LiNo<String>,
    ontology: &mut Ontology,
    source_uri: &str,
    source_line: usize,
) {
    match node {
        LiNo::Ref(_) => {
            // bare ref at top level — ignore (informational tokens like
            // `(language en)` get matched below via the Link arm)
        }
        LiNo::Link { values, .. } => {
            // Top level entries take one of these shapes:
            //   (concept_name (label …) (origin …) (definition …) …)
            //   (mapping a ~ b)
            //   (exponent <concept> <lang> <token...>)
            //   (allowlist <word> (reason …))
            //   (language <code>)
            //   (dataset (schema_version 1))
            //   (relation <from> <kind> <to>)
            //   (governance <concept> ...)
            //   (language_label <name>)
            //   (coverage full|provisional)
            let head = first_ref(&values).map(String::as_str);
            let metadata_form = values
                .get(1)
                .is_some_and(|value| matches!(value, LiNo::Ref(_)));
            let dataset_form = values.get(1).is_some_and(|value| {
                matches!(
                    value,
                    LiNo::Link { values: fields, .. }
                        if first_ref(fields).map(String::as_str) == Some("schema_version")
                )
            });
            match head {
                Some("mapping") if metadata_form => absorb_mapping(&values, ontology),
                Some("exponent") if metadata_form => absorb_exponent(&values, ontology),
                Some("allowlist") if metadata_form => absorb_allowlist(&values, ontology),
                Some("language") if metadata_form => absorb_language(&values, ontology),
                Some("dataset") if dataset_form => absorb_dataset(&values, ontology),
                Some("relation") if metadata_form => {
                    absorb_relation(&values, ontology, source_uri, source_line);
                }
                Some("governance") if metadata_form => {
                    absorb_governance(&values, ontology, source_uri, source_line);
                }
                Some("language_label" | "coverage") | None => {} // metadata only for now
                Some(_) => absorb_concept(values, ontology, source_uri, source_line),
            }
        }
    }
}

fn absorb_concept(
    values: Vec<LiNo<String>>,
    ontology: &mut Ontology,
    source_uri: &str,
    source_line: usize,
) {
    let mut iter = values.into_iter();
    let Some(LiNo::Ref(name)) = iter.next() else {
        return;
    };
    let concept = ontology.concept_mut_or_insert(&name, source_uri, source_line);
    concept.provenance.source_uri = source_uri.to_string();
    concept.provenance.source_line = source_line;
    for child in iter {
        match child {
            LiNo::Link { values: kids, .. } => {
                let key = first_ref(&kids).cloned().unwrap_or_default();
                let rest: Vec<String> = kids.iter().skip(1).flat_map(flatten_words).collect();
                match key.as_str() {
                    "id" => {
                        if let Some(id) = rest.first() {
                            concept.id = ConceptId::new(id);
                        }
                    }
                    "label" => {
                        concept.label = rest.join(" ");
                    }
                    "origin" => {
                        if let Some(o) = rest.first() {
                            concept.origin.clone_from(o);
                        }
                    }
                    "category" => {
                        if let Some(c) = rest.first() {
                            concept.category.clone_from(c);
                        }
                    }
                    "allolex" => {
                        if let Some(a) = rest.first() {
                            concept.allolexes.push(a.clone());
                        }
                    }
                    "alias" => {
                        if let Some(alias) = rest.first() {
                            concept.aliases.push(alias.clone());
                        }
                    }
                    "definition" => {
                        concept.definitions.push(Definition { words: rest });
                    }
                    "mapping" => {
                        if let Some(ext) = rest.first() {
                            concept.mappings.push(Mapping {
                                external: ext.clone(),
                                kind: "~".to_string(),
                            });
                        }
                    }
                    "owner" => concept.governance.owners.extend(rest),
                    "classification" => {
                        concept.governance.classifications.extend(rest);
                    }
                    "tag" => concept.governance.tags.extend(rest),
                    "lifecycle" => {
                        if let Some(state) = rest.first() {
                            concept.governance.lifecycle = parse_lifecycle(state);
                        }
                    }
                    "provenance" => absorb_provenance(&kids, &mut concept.provenance),
                    _ => {}
                }
            }
            LiNo::Ref(_) => {}
        }
    }
}

fn absorb_dataset(values: &[LiNo<String>], ontology: &mut Ontology) {
    for value in values.iter().skip(1) {
        let LiNo::Link { values: fields, .. } = value else {
            continue;
        };
        if first_ref(fields).map(String::as_str) != Some("schema_version") {
            continue;
        }
        if let Some(version) = fields.iter().skip(1).find_map(as_ref) {
            if let Ok(version) = version.parse() {
                ontology.schema_version = version;
            }
        }
    }
}

fn absorb_relation(
    values: &[LiNo<String>],
    ontology: &mut Ontology,
    source_uri: &str,
    source_line: usize,
) {
    let parts = values.iter().filter_map(as_ref).collect::<Vec<_>>();
    if parts.len() < 4 {
        return;
    }
    ontology.relationships.push(Relationship {
        from: relation_identity(parts[1]),
        kind: parts[2].to_string(),
        to: relation_identity(parts[3]),
        provenance: Provenance::for_source(source_uri, source_line),
    });
}

fn absorb_governance(
    values: &[LiNo<String>],
    ontology: &mut Ontology,
    source_uri: &str,
    source_line: usize,
) {
    let Some(name) = values.iter().skip(1).find_map(as_ref) else {
        return;
    };
    let concept = ontology.concept_mut_or_insert(name, source_uri, source_line);
    for value in values.iter().skip(2) {
        let LiNo::Link { values: fields, .. } = value else {
            continue;
        };
        let key = first_ref(fields).map(String::as_str);
        let rest = fields
            .iter()
            .skip(1)
            .flat_map(flatten_words)
            .collect::<Vec<_>>();
        match key {
            Some("owner") => concept.governance.owners.extend(rest),
            Some("classification") => {
                concept.governance.classifications.extend(rest);
            }
            Some("tag") => concept.governance.tags.extend(rest),
            Some("lifecycle") => {
                if let Some(state) = rest.first() {
                    concept.governance.lifecycle = parse_lifecycle(state);
                }
            }
            Some("provenance") => absorb_provenance(fields, &mut concept.provenance),
            _ => {}
        }
    }
}

fn absorb_provenance(values: &[LiNo<String>], provenance: &mut Provenance) {
    for value in values.iter().skip(1) {
        let LiNo::Link { values: fields, .. } = value else {
            continue;
        };
        let key = first_ref(fields).map(String::as_str);
        let rest = fields
            .iter()
            .skip(1)
            .flat_map(flatten_words)
            .collect::<Vec<_>>()
            .join(" ");
        match key {
            Some("source_uri") => provenance.source_uri = rest,
            Some("agent") => provenance.agent = rest,
            Some("license") => provenance.license = rest,
            Some("review_state") => {
                provenance.review_state = match rest.as_str() {
                    "draft" => ReviewState::Draft,
                    "rejected" => ReviewState::Rejected,
                    _ => ReviewState::Reviewed,
                };
            }
            _ => {}
        }
    }
}

fn absorb_mapping(values: &[LiNo<String>], ontology: &mut Ontology) {
    // (mapping <local> <kind> <external>)
    let parts: Vec<&str> = values.iter().filter_map(as_ref).collect();
    if parts.len() < 4 {
        return;
    }
    let (local, kind, external) = (parts[1], parts[2], parts[3]);
    if let Some(c) = ontology.concepts.get_mut(local) {
        c.mappings.push(Mapping {
            external: external.to_string(),
            kind: kind.to_string(),
        });
    }
}

fn absorb_exponent(values: &[LiNo<String>], ontology: &mut Ontology) {
    // (exponent <concept> <lang> <word> ... [(coverage ...)])
    let mut parts = values.iter();
    let _ = parts.next(); // skip `exponent`
    let Some(LiNo::Ref(concept_name)) = parts.next() else {
        return;
    };
    let Some(LiNo::Ref(lang)) = parts.next() else {
        return;
    };
    let mut tokens = Vec::new();
    for v in parts {
        match v {
            LiNo::Ref(s) => tokens.push(s.clone()),
            LiNo::Link { values: inner, .. } => {
                // Skip metadata sub-links like (coverage full); use
                // grouped sub-links as the multi-word token.
                if first_ref(inner).map(String::as_str) == Some("coverage") {
                    continue;
                }
                let words: Vec<String> = inner.iter().flat_map(flatten_words).collect();
                tokens.push(words.join(" "));
            }
        }
    }
    let lang = lang.clone();
    ontology.languages.insert(lang.clone());
    if let Some(c) = ontology.concepts.get_mut(concept_name) {
        c.exponents.insert(lang, tokens.join(" "));
    }
}

fn absorb_allowlist(values: &[LiNo<String>], ontology: &mut Ontology) {
    // (allowlist <word> (reason ...))
    let parts: Vec<&str> = values.iter().filter_map(as_ref).collect();
    if let Some(word) = parts.get(1) {
        ontology.allowlist.insert((*word).to_string());
    }
}

fn absorb_language(values: &[LiNo<String>], ontology: &mut Ontology) {
    // (language <code>)
    let parts: Vec<&str> = values.iter().filter_map(as_ref).collect();
    if let Some(code) = parts.get(1) {
        ontology.languages.insert((*code).to_string());
    }
}

// --- helpers ---------------------------------------------------------

fn source_line(text: &str, node: &LiNo<String>, search_from: &mut usize) -> usize {
    let Some(head) = (match node {
        LiNo::Link { values, .. } => first_ref(values),
        LiNo::Ref(_) => None,
    }) else {
        return 1;
    };
    let marker = format!("({head}");
    let offset = text[*search_from..]
        .find(&marker)
        .map_or(*search_from, |relative| *search_from + relative);
    *search_from = offset.saturating_add(marker.len());
    text[..offset].bytes().filter(|byte| *byte == b'\n').count() + 1
}

fn relation_identity(value: &str) -> ConceptId {
    if value.contains(':') {
        ConceptId::new(value)
    } else {
        ConceptId::from_name(value)
    }
}

fn parse_lifecycle(value: &str) -> LifecycleState {
    match value {
        "deprecated" => LifecycleState::Deprecated,
        "deleted" => LifecycleState::Deleted,
        _ => LifecycleState::Active,
    }
}

fn first_ref(values: &[LiNo<String>]) -> Option<&String> {
    values.iter().find_map(|v| match v {
        LiNo::Ref(s) => Some(s),
        LiNo::Link { .. } => None,
    })
}

const fn as_ref(v: &LiNo<String>) -> Option<&str> {
    match v {
        LiNo::Ref(s) => Some(s.as_str()),
        LiNo::Link { .. } => None,
    }
}

fn flatten_words(v: &LiNo<String>) -> Vec<String> {
    match v {
        LiNo::Ref(s) => vec![s.clone()],
        LiNo::Link { values, .. } => values.iter().flat_map(flatten_words).collect(),
    }
}
