//! Meta‑ontology library.
//!
//! Loads `.lino` files into an in-memory [`Ontology`] and exposes
//! lookups and graph traversals. The data layer is a network (cycles
//! allowed), backed by a name → [`Concept`] map plus a list of
//! parsed links.
//!
//! See `docs/plan/02-library.md` for the design.

pub mod loader;
pub mod ontology;
pub mod words;

pub use ontology::{Concept, Definition, Mapping, Ontology};
