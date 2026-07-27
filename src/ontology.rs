//! Core ontology types and queries.

use std::collections::{BTreeMap, BTreeSet};

use meta_language::LinkNetwork;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::catalog::{ConceptId, Governance, Provenance, Relationship, CURRENT_SCHEMA_VERSION};

/// A single concept in the meta-ontology.
///
/// Cycles between concepts are allowed and expected — `thing`, `concept`,
/// and `link` all reference each other in seed data.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Concept {
    /// Stable opaque identity. Names and labels may change without changing it.
    pub id: ConceptId,
    /// Canonical lowercase identifier, e.g. `thing`.
    pub name: String,
    /// Human‑readable label (English by default), e.g. `Thing`.
    pub label: String,
    /// Origin tag: `lino`, `nsm`, `schema`, `owl`, …
    pub origin: String,
    /// NSM category (substantive, quantifier, …) or empty.
    pub category: String,
    /// Allolexes (alternate surface forms in NSM theory).
    pub allolexes: Vec<String>,
    /// Explicit lookup aliases. Unlike NSM allolexes, these must be globally
    /// unambiguous.
    pub aliases: Vec<String>,
    /// All definitions, in insertion order.
    pub definitions: Vec<Definition>,
    /// Cross‑ontology equivalences.
    pub mappings: Vec<Mapping>,
    /// Per‑language exponents loaded from `data/primes/exponents/<lang>.lino`.
    pub exponents: BTreeMap<String, String>,
    /// Evidence describing where this entity came from.
    pub provenance: Provenance,
    /// Ownership, lifecycle, classification, and tags.
    pub governance: Governance,
}

/// One natural‑language definition of a concept.
///
/// A definition is a flat list of words (concept names) — the loader
/// preserves them in document order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Definition {
    /// Ordered concept words used by the definition.
    pub words: Vec<String>,
}

/// A cross‑ontology mapping — `thing ~ schema_Thing` and friends.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Mapping {
    /// External name like `schema_Thing`, `wikidata_Q35120`.
    pub external: String,
    /// Relation kind: `~` (equivalent), `<` (narrower), `>` (broader).
    pub kind: String,
}

/// In‑memory representation of the meta‑ontology.
///
/// Built by [`crate::loader`] from `.lino` files.
#[derive(Debug, Clone)]
pub struct Ontology {
    /// Version of the public catalog/interchange contract.
    pub(crate) schema_version: u32,
    /// All concepts, keyed by canonical name. `BTreeMap` keeps `names()`
    /// alphabetical without an extra sort.
    pub(crate) concepts: BTreeMap<String, Concept>,
    /// Explicit, typed edges between stable concept identities.
    pub(crate) relationships: Vec<Relationship>,
    /// Words that never need a concept — proper nouns, identifiers, etc.
    pub(crate) allowlist: BTreeSet<String>,
    /// Languages declared in any exponent file.
    pub(crate) languages: BTreeSet<String>,
    /// Normalized links-network representation supplied by `meta-language`.
    pub(crate) network: LinkNetwork,
}

impl Default for Ontology {
    fn default() -> Self {
        Self {
            schema_version: CURRENT_SCHEMA_VERSION,
            concepts: BTreeMap::new(),
            relationships: Vec::new(),
            allowlist: BTreeSet::new(),
            languages: BTreeSet::new(),
            network: LinkNetwork::new(),
        }
    }
}

impl Ontology {
    pub(crate) fn concept_mut_or_insert(
        &mut self,
        name: &str,
        source_uri: &str,
        source_line: usize,
    ) -> &mut Concept {
        self.concepts
            .entry(name.to_string())
            .or_insert_with(|| Concept {
                id: ConceptId::from_name(name),
                name: name.to_string(),
                provenance: Provenance::for_source(source_uri, source_line),
                ..Concept::default()
            })
    }

    /// Iterate every concept name, alphabetically.
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.concepts.keys().map(String::as_str)
    }

    /// Look up a concept by name (case sensitive — names are canonical
    /// lowercase identifiers).
    #[must_use]
    pub fn find(&self, name: &str) -> Option<&Concept> {
        self.concepts.get(name)
    }

    /// Number of concepts in the ontology.
    #[must_use]
    pub fn len(&self) -> usize {
        self.concepts.len()
    }

    /// Whether the ontology has no concepts.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.concepts.is_empty()
    }

    /// Iterate the names of concepts referenced by `name`'s definitions
    /// (out‑neighbours in the definition graph).
    ///
    /// Words that are not themselves concept names are skipped. Returns
    /// an empty iterator if `name` is unknown.
    pub fn neighbors<'a>(&'a self, name: &'a str) -> impl Iterator<Item = &'a Concept> + 'a {
        self.concepts
            .get(name)
            .into_iter()
            .flat_map(|c| c.definitions.iter())
            .flat_map(|def| def.words.iter())
            .filter_map(move |word| self.concepts.get(word.as_str()))
    }

    /// Check whether a lower‑cased word is allow‑listed (proper noun,
    /// identifier, version string, etc.).
    #[must_use]
    pub fn is_allowed(&self, word: &str) -> bool {
        self.allowlist.contains(word)
    }

    /// Whether a lower‑cased word is either a known concept (or its
    /// allolex) or on the allow‑list.
    #[must_use]
    pub fn covers(&self, word: &str) -> bool {
        if self.concepts.contains_key(word) || self.is_allowed(word) {
            return true;
        }
        // Match allolexes
        self.concepts
            .values()
            .any(|c| c.allolexes.iter().any(|a| a == word))
    }

    /// Iterate all languages with at least one declared exponent.
    pub fn languages(&self) -> impl Iterator<Item = &str> {
        self.languages.iter().map(String::as_str)
    }
}

#[cfg(test)]
mod tests {
    use crate::loader::load_from_dir;

    fn data_dir() -> std::path::PathBuf {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        std::path::PathBuf::from(manifest_dir).join("data")
    }

    #[test]
    fn loads_seed_data() {
        let o = load_from_dir(data_dir()).expect("load");
        assert!(o.find("thing").is_some(), "thing must be in seed data");
        assert!(o.find("concept").is_some(), "concept must be in seed data");
        assert!(o.find("link").is_some(), "link must be in seed data");
        assert!(
            o.find("language").is_some(),
            "the language concept must not be treated as a language declaration"
        );
        assert!(
            o.find("relation").is_some(),
            "the relation concept must not be treated as an edge declaration"
        );
        assert!(
            o.find("exponent").is_some(),
            "the exponent concept must not be treated as an exponent declaration"
        );
        assert!(
            o.find("dataset").is_some(),
            "the dataset concept must not be treated as schema metadata"
        );
        assert!(
            o.find("governance").is_some(),
            "the governance concept must not be treated as governance metadata"
        );
    }

    #[test]
    fn cycle_exists_thing_concept_thing() {
        let o = load_from_dir(data_dir()).expect("load");
        // Walk neighbours of `thing` — `concept` should be reachable, and
        // walking from `concept` should reach `thing` again, completing a
        // cycle. We only need to assert one step each direction.
        let from_thing: Vec<&str> = o.neighbors("thing").map(|c| c.name.as_str()).collect();
        assert!(
            from_thing.contains(&"concept"),
            "thing -> concept must exist, got {from_thing:?}"
        );
        let from_concept: Vec<&str> = o.neighbors("concept").map(|c| c.name.as_str()).collect();
        assert!(
            from_concept.contains(&"thing"),
            "concept -> thing must exist (cycle), got {from_concept:?}"
        );
    }

    #[test]
    fn nsm_primes_present() {
        let o = load_from_dir(data_dir()).expect("load");
        for prime in [
            "i", "you", "someone", "people", "body", "kind", "part", "this", "other", "one", "two",
            "good", "bad", "big", "small", "know", "think", "want", "feel", "see", "hear", "say",
            "words", "true", "do", "happen", "move", "live", "die", "now", "here", "not", "if",
            "very", "more", "like",
        ] {
            assert!(o.find(prime).is_some(), "missing NSM prime: {prime}");
        }
    }

    #[test]
    fn multiple_definitions_preserved() {
        let o = load_from_dir(data_dir()).expect("load");
        let thing = o.find("thing").expect("thing");
        assert!(
            thing.definitions.len() >= 2,
            "thing must have multiple definitions, got {}",
            thing.definitions.len()
        );
    }

    #[test]
    fn english_exponents_loaded() {
        let o = load_from_dir(data_dir()).expect("load");
        let lang_codes: Vec<&str> = o.languages().collect();
        assert!(
            lang_codes.contains(&"en"),
            "english must be present, got {lang_codes:?}"
        );
    }

    #[test]
    fn allowlist_rejects_non_listed_word() {
        let o = load_from_dir(data_dir()).expect("load");
        assert!(o.is_allowed("github"));
        assert!(!o.is_allowed("totallyimaginarywordxyz"));
    }
}
