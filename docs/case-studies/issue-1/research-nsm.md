# NSM Semantic Primes — Research Notes

Source of truth for [`data/primes/nsm.lino`](../../../data/primes/nsm.lino).

## The 65 canonical primes (English, v19, 2017)

Tildes (`~`) mark allolexes — different surface forms of the same prime
(e.g. `SOMETHING ~ THING`).

### Substantives
- I
- YOU
- SOMEONE
- SOMETHING ~ THING
- PEOPLE
- BODY

### Relational substantives
- KIND
- PART

### Determiners
- THIS
- THE SAME
- OTHER ~ ELSE

### Quantifiers
- ONE
- TWO
- SOME
- ALL
- MUCH ~ MANY
- LITTLE ~ FEW

### Evaluators
- GOOD
- BAD

### Descriptors
- BIG
- SMALL

### Mental predicates
- KNOW
- THINK
- WANT
- DON'T WANT
- FEEL
- SEE
- HEAR

### Speech
- SAY
- WORDS
- TRUE

### Actions, events, movement
- DO
- HAPPEN
- MOVE

### Location, existence, specification, possession
- BE (SOMEWHERE)
- THERE IS
- BE (SOMEONE/SOMETHING)
- (IS) MINE

### Life and death
- LIVE
- DIE

### Time
- WHEN ~ TIME
- NOW
- BEFORE
- AFTER
- A LONG TIME
- A SHORT TIME
- FOR SOME TIME
- MOMENT

### Space
- WHERE ~ PLACE
- HERE
- ABOVE
- BELOW
- FAR
- NEAR
- SIDE
- INSIDE
- TOUCH (CONTACT)

### Logical concepts
- NOT
- MAYBE
- CAN
- BECAUSE
- IF

### Intensifier, augmentor
- VERY
- MORE

### Similarity
- LIKE ~ AS ~ WAY

**Total: 65 primes in 16 categories.**

## Conventions

- Primes are written in **SMALL CAPS** in NSM literature; in our `.lino` files we
  use lowercase identifiers and store the canonical English label as a separate
  property.
- `DON'T WANT` is counted as a separate prime (the negative mental predicate).
- Some publications list `HAVE` instead of `(IS) MINE`. The current canonical chart
  uses the four‑part `BE (SOMEWHERE)` / `THERE IS` / `BE (SOMEONE/SOMETHING)` /
  `(IS) MINE` block.

## Language coverage (top‑20 most‑spoken)

In NSM theory, translations of primes are **exponents**. Empirical exponent tables
have been published for 30+ languages across 16+ language families. Coverage check
against the user‑specified top‑20:

| # | Language | Official NSM exponent table? | Notes |
|---|----------|-------------------------------|-------|
| 1 | Mandarin Chinese | ✅ | nsm-approach.net; also Cantonese |
| 2 | Spanish | ✅ | Major NSM language |
| 3 | English | ✅ | Reference language |
| 4 | Hindi | ❌ | Gap |
| 5 | Arabic | ✅ | |
| 6 | Bengali | ❌ | Gap |
| 7 | Portuguese | ⚠ | Discussed; no standalone table |
| 8 | Russian | ✅ | Wierzbicka L1, extensive |
| 9 | Japanese | ✅ | 11+ studies |
| 10 | German | ✅ | |
| 11 | Korean | ✅ | |
| 12 | French | ✅ | |
| 13 | Turkish | ❌ | Gap |
| 14 | Vietnamese | ✅ | |
| 15 | Italian | ✅ | |
| 16 | Polish | ✅ | Wierzbicka L1 |
| 17 | Ukrainian | ❌ | Gap |
| 18 | Persian (Farsi) | ✅ | |
| 19 | Punjabi | ❌ | Gap |
| 20 | Indonesian | ⚠ | Use Malay table as proxy |

Other documented NSM exponent tables: Amharic, Cèmuhi, Czech, Danish, Dutch, Ewe,
Finnish, Hebrew, Longgu, Serbian, Trini (Trinidadian Creole), Wolof, Yankunytjatjara,
Arrernte, East Cree, Mbula, Koromu.

### Distribution venues

- Primary catalogue: <https://nsm-approach.net>
- Griffith University NSM portal (chart PDFs)
- Online "Minimal English" tools at <https://learnthesewordsfirst.com>
- Academic publishers: Oxford University Press, John Benjamins, Springer/Palgrave

### Implication for the meta‑ontology

- **~13 / 20** of the top languages have authoritative tables we can ingest.
- For the remaining **~7** (Hindi, Bengali, Turkish, Ukrainian, Punjabi; partly
  Portuguese & Indonesian) we either (a) rely on the NSM theoretical claim of
  universality and elicit our own provisional exponents, (b) treat them as
  open data slots in the ontology, marked with `coverage: provisional`, or
  (c) fall back to a related language (Malay → Indonesian).

The roadmap addresses this in Milestone **M2 — Multilingual exponents**.

## References

- [Natural Semantic Metalanguage — Wikipedia](https://en.wikipedia.org/wiki/Natural_semantic_metalanguage)
- [NSM Approach — Resources](https://nsm-approach.net/resources)
- [Semantic Primes catalogue](https://nsm-approach.net/archives/category/nsm-toolkit/semantic-primes)
- [Griffith chart v19 (PDF)](https://intranet.secure.griffith.edu.au/__data/assets/pdf_file/0019/346033/NSM_Chart_ENGLISH_v19_April_12_2017_Greyscale.pdf)
- [Casey Keith — Semantic Primes summary](https://caseykeith.me/semantic-primes/)
