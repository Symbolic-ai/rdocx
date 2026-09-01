# F-X073, correctness, pass 4

**Reviewed**: claim base `ef34d1f31eb9334d993bf333762079682633319f`
through the complete current working-tree delta, the approved revised plan,
cited HLD sections, progress record, all three prior correctness reviews, and
untracked state. The tracked delta is 7 files with 910 changed lines, 743
additions and 167 deletions. The three untracked prior reviews add 335 reviewed
lines.
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None. Count: 0.

## Smells

None. Count: 0.

## Nitpicks

None. Count: 0.

## Not found

- **Pass 3 D1, expanded-name cardinality** produced zero findings. The raw
  guard resolves every candidate `id` and `name` attribute through its local
  namespace declarations, counts Word-expanded identities across different
  aliases, and requires exactly one `id` plus exactly one start `name` or zero
  end names at `crates/rdocx-layout/src/engine.rs:2499`. Same-prefix and
  same-URI aliased duplicates fail closed independently of their values.
- **Valid aliases and namespace shadows** produced zero findings. Root and
  attribute identity use namespace URI plus local name rather than literal
  prefix at `crates/rdocx-layout/src/engine.rs:2494`. A locally declared Word
  alias remains eligible, while a used alias or fixed prefix rebound to a
  foreign URI fails before typed-marker correspondence. The positive alias and
  duplicate-alias regressions are at
  `crates/rdocx-layout/src/engine.rs:11222` and
  `crates/rdocx-layout/src/engine.rs:11276`.
- **Complete bookmark root validation** produced zero findings. The candidate
  must be one lexical empty element, reparsing must yield exactly one paragraph
  raw entry and one bookmark marker, and the parsed raw bytes, marker type,
  required values, and empty paragraph state must agree at
  `crates/rdocx-layout/src/engine.rs:2514`. Foreign roots, malformed XML,
  nested content, trailing content, missing or invalid IDs, missing start
  names, and unexpected end names fail closed.
- **Raw and typed sidecar correspondence** produced zero findings. Every raw
  entry must pair with one marker at the same run boundary and with the exact
  number of preceding raw children at that boundary at
  `crates/rdocx-layout/src/engine.rs:2453`. Reordering vectors or retaining a
  stale `BookmarkMarker::raw_before` cannot publish restart state.
- **Source field safety and substitution pairs** produced zero findings. Every
  direct source `RunContent::Field`, including instructions whose rendered
  `field_kind` is `None`, prevents checkpoint pagination at
  `crates/rdocx-layout/src/engine.rs:1417`. Supported PAGE, NUMPAGES, and
  PAGEREF page pairs remain eligible for the complete restart record, as pinned
  at `crates/rdocx-layout/src/engine.rs:11665`.
- **Aggregate restart budget** produced zero findings. Candidate admission uses
  checked arithmetic across current and pending paragraph, table, header or
  footer state plus the complete restart candidate at
  `crates/rdocx-layout/src/engine.rs:2180`. The 5,216-entry and 64 MiB aggregate
  ceilings, component caps, retained-capacity accounting, complete body
  identity, candidate replacement, and overflow rejection remain intact.
- **Split fallback** produced zero findings. Every actual paragraph split sets
  the recorded-pass flag at `crates/rdocx-layout/src/paginator.rs:2170`. The
  engine then disables restart publication and reruns complete pagination, so
  no partial paragraph checkpoint or old tail can be retained.
- **Warm and fresh equality** produced zero findings. Late edit, insertion,
  deletion, and undo use deterministic fonts and compare the complete warm
  result with a fresh engine at `crates/rdocx-layout/src/engine.rs:11073`.
  Exact retained context, body identity, note order, font trace, provenance
  mode, substitution inputs, and suffix equality remain authoritative.
- **Bounded work** produced zero findings. The 700-paragraph late-edit gate
  bounds page recomputation while requiring fresh equality at
  `crates/rdocx-layout/src/engine.rs:11141`. The 1,000-page facade and
  release-mode deterministic performance riders also passed.
- **Panics and arithmetic** produced zero findings. Raw indices derive from
  enumeration, nonempty slicing is guarded, XML parse failure returns false,
  aggregate addition is checked, and retained-byte calculations saturate to a
  rejected candidate. The production delta adds no unbounded recursion.
- **OOXML preservation and schema order** produced zero findings. The cache
  classifier never rewrites bookmark raw XML. No serializer, schema sequence,
  relationship, part, or package behavior changed, and exact body identity
  continues to include the retained raw bytes.
- **Public API and structure** produced zero findings. No public API,
  dependency, feature, module, file, trait, generic, binding, or package graph
  changed. The private raw classifier and cardinality closure remain beside the
  existing restart source-safety predicates.
- **HLD and hash expectations** produced zero findings. HLD 08, 12, 14, and 15
  consistently describe complete-boundary restart eligibility, field-pair
  retention, aggregate admission, and the retired independent byte ceiling.
  The deterministic hash harness matched all 49 entries unchanged.
- **Focused and routed evidence** produced zero findings. The bookmark
  adversarial regression, split fallback, source-field and substitution-pair
  cases, ordinary-prose cases, and aggregate-budget cases passed. Full
  `rdocx-layout` passed 225 unit tests and one doctest. The `rdocx` library
  passed 329 tests with four ignored, and its regression binary passed 180
  tests with one ignored. All-target, all-feature `rdocx-layout` Clippy, both
  WASM facade target checks, workspace formatting, prose, diff hygiene, the
  deterministic 49-entry hash harness, and the exact release-mode 1,000-page
  performance regression passed.
