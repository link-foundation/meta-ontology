# Popular World Ontologies — Research Notes

Survey for requirement **R1** (collect data about the most popular ontologies)
and **R3** (include cyclic / prime concepts from world ontologies).

## Survey table

| Ontology | Top‑level "primitive" layer | Comparable to NSM primes? |
|---|---|---|
| **Schema.org** | Single root `Thing`; ~12 immediate subclasses (CreativeWork, Event, Organization, Person, Place, Product, Action, Intangible, …). RDF/RDFS‑based. | No — markup vocabulary. |
| **OWL / RDF(S)** | Built‑ins: `owl:Thing`, `owl:Nothing`, `rdf:Property`, `rdfs:Class`, plus logical constructors. | No — formal logic primitives. |
| **WordNet** | ~25 "unique beginners" (lexicographer files: `noun.act`, `noun.animal`, …); inheritance via hypernymy. | Loosely; not claimed universal nor decompositional. |
| **ConceptNet** | No fixed primitive set. Uses ~36 typed relations (`IsA`, `UsedFor`, `AtLocation`, `CapableOf`, …) over millions of multilingual concept nodes. Imported a high‑level ontology from OpenCyc. | Closest analogue at the *relation* level; concept layer is open. |
| **BabelNet** | No primitive set. Multilingual graph integrating WordNet + Wikipedia/Wikidata + others into Babel synsets. | No — but addresses multilingual mapping. |
| **Cyc / OpenCyc / ResearchCyc** | Thousands of upper‑level constants (`Thing`, `Individual`, `Collection`, `Event`, `TemporalThing`, `SpatialThing`, …). Heavily axiomatised in CycL. | Partially — at vastly larger scale, aimed at logical reasoning. |
| **SUMO / MILO** | ~1 000 top‑level concepts (Entity → Physical/Abstract → Object/Process → …). Mapped completely to WordNet 3.0. IEEE P1600.1 standard. | Partially — closer in spirit. |
| **DBpedia** | ~700‑class ontology auto‑derived from Wikipedia infoboxes; root `owl:Thing`. | No — encyclopedic taxonomy. |
| **Wikidata** | No fixed top set; uses `instance of` (P31) + `subclass of` (P279) over Q‑items. Top items like `entity (Q35120)` exist but are emergent. | No — open‑ended crowd ontology. |
| **FrameNet** | ~1 200 frames with ~10 000 frame elements (semantic roles). Built on Fillmore's frame semantics. | Conceptually closer — recurring meaning structures, small typed FE inventories — but no closed universal "prime" list. |

## Key contrasts

- **NSM is decompositional and lexical**: every other concept is paraphrased via
  the 65 primes in natural language (explications / cultural scripts).
- **Mainstream ontologies are taxonomic and formal**: concepts are classified
  under classes, with logic‑based axioms or graph relations, not paraphrased.
- **NSM is small and claimed universal across languages**; Cyc/SUMO are large and
  language‑agnostic via IDs; Wikidata is open‑ended.
- **Closest hybrids**: ConceptNet's small relation set, FrameNet's frame‑element
  inventory, and SUMO's curated upper‑level all rhyme with NSM in spirit, but
  none commits to a fixed semantic‑primitive vocabulary that doubles as a
  writable natural‑language metalanguage.

## "Prime" / cyclic concepts to include (R3)

The issue specifically asks for "all prime concepts from all worlds ontologies
(the concepts that use themselves in a loop or just defined only as themselves)".
Concrete candidates we should ingest as primes alongside NSM:

| Concept | Source | Why it qualifies |
|---|---|---|
| `thing` | Schema.org `Thing`; OWL `owl:Thing`; Wikidata `entity (Q35120)` | Universal root; defined only as "the most general type". Self‑referential in many uppers. |
| `class` | RDF Schema `rdfs:Class` | A class is defined as an instance of `rdfs:Class`. Cyclic. |
| `property` | RDF `rdf:Property` | Itself a property. Cyclic. |
| `relation` | OWL / SUMO | Defined relative to other relations; appears at top of every ontology. |
| `concept` | SKOS `skos:Concept`; everyday usage | Self‑defining (a "concept" is itself a concept). |
| `set` | Set theory, Cyc `Collection` | Foundational primitive in many uppers. |
| `entity` | SUMO `Entity`; Wikidata `Q35120` | Highest‑level concept. |
| `link` | Links Notation; this project's chosen substrate | The vehicle of every other definition. |
| `reference` | Links Notation | A reference *is* a kind of link to a name. |
| `name` | Universal | A name is a reference to a thing. |

Mapping table format in `data/concepts/mappings.lino` (Milestone **M2**):

```
mapping (
  schema:Thing
  ≈
  thing
)
mapping (
  owl:Thing
  ≈
  thing
)
mapping (
  wikidata:Q35120
  ≈
  thing
)
```

Cross‑ontology equivalences are stored as **additional links**, not as
inheritance, so the meta‑ontology stays a network rather than a tree.

## Hybrid recommendation

```
+----------------------------------------------------+
| Upper layer (SUMO/Wikidata‑style, formal axioms)   |
+----------------------------------------------------+
| Cross‑ontology mapping links (Schema.org, OWL, …)  |
+----------------------------------------------------+
| NSM 65 primes (universal lexical kernel)           |
+----------------------------------------------------+
| Self‑referential lino primes (link, thing, …)      |
+----------------------------------------------------+
```

Build outward from the bottom. Every higher layer is **paraphrasable in primes**.

## References

- [Schema.org full hierarchy](https://schema.org/docs/full.html)
- [Schema.org Thing](https://schema.org/Thing)
- [OWL 2 Primer](https://www.w3.org/TR/owl2-primer/)
- [WordNet 3.1](https://wordnet.princeton.edu/)
- [ConceptNet 5](https://conceptnet.io)
- [BabelNet](https://babelnet.org)
- [Cyc / OpenCyc](https://www.cyc.com/)
- [SUMO / MILO](https://www.ontologyportal.org/)
- [DBpedia ontology](https://www.dbpedia.org/resources/ontology/)
- [Wikidata Q35120 entity](https://www.wikidata.org/wiki/Q35120)
- [FrameNet](https://framenet.icsi.berkeley.edu/)
- [Upper ontology — Wikipedia](https://en.wikipedia.org/wiki/Upper_ontology)
