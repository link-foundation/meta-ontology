//! Load `.lino` files from disk into an [`Ontology`].
//!
//! The loader walks a directory recursively, parses every `*.lino` file
//! with `links-notation`, and folds each top-level link into the
//! ontology. Cycles are allowed and never rejected.

use std::path::{Path, PathBuf};

use links_notation::{parse_lino_to_links, LiNo};
use thiserror::Error;
use walkdir::WalkDir;

use crate::ontology::{Concept, Definition, Mapping, Ontology};

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
    for entry in WalkDir::new(dir.as_ref()) {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("lino") {
            continue;
        }
        load_file(path, &mut ontology)?;
    }
    Ok(ontology)
}

/// Load a single `.lino` file into the ontology.
pub fn load_file(path: &Path, ontology: &mut Ontology) -> Result<(), LoadError> {
    let text = std::fs::read_to_string(path).map_err(|source| LoadError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let links = parse_lino_to_links(&text).map_err(|e| LoadError::Parse {
        path: path.to_path_buf(),
        message: format!("{e}"),
    })?;

    for link in links {
        absorb_top_level(link, ontology);
    }
    Ok(())
}

fn absorb_top_level(node: LiNo<String>, ontology: &mut Ontology) {
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
            //   (language_label <name>)
            //   (coverage full|provisional)
            let head = first_ref(&values).map(String::as_str);
            match head {
                Some("mapping") => absorb_mapping(&values, ontology),
                Some("exponent") => absorb_exponent(&values, ontology),
                Some("allowlist") => absorb_allowlist(&values, ontology),
                Some("language") => absorb_language(&values, ontology),
                Some("language_label" | "coverage") | None => {} // metadata only for now
                Some(_) => absorb_concept(values, ontology),
            }
        }
    }
}

fn absorb_concept(values: Vec<LiNo<String>>, ontology: &mut Ontology) {
    let mut iter = values.into_iter();
    let Some(LiNo::Ref(name)) = iter.next() else {
        return;
    };
    let concept = ontology
        .concepts
        .entry(name.clone())
        .or_insert_with(|| Concept {
            name: name.clone(),
            ..Concept::default()
        });
    for child in iter {
        match child {
            LiNo::Link { values: kids, .. } => {
                let key = first_ref(&kids).cloned().unwrap_or_default();
                let rest: Vec<String> = kids.iter().skip(1).flat_map(flatten_words).collect();
                match key.as_str() {
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
                    _ => {}
                }
            }
            LiNo::Ref(_) => {}
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

fn first_ref(values: &[LiNo<String>]) -> Option<&String> {
    values.iter().find_map(|v| match v {
        LiNo::Ref(s) => Some(s),
        LiNo::Link { .. } => None,
    })
}

fn as_ref(v: &LiNo<String>) -> Option<&str> {
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
