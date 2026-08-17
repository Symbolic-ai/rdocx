# F-154, Bookmarks and cross-references

**Status**: completed
**Sprint**: S46
**Size**: M
**Depends on**: none

## Problem

Paragraph parsing currently preserves bookmarks only as indexed raw XML
(`crates/rdocx-oxml/src/text.rs:428`). The table-of-contents helper writes new
bookmark strings directly into those raw slots (`crates/rdocx/src/document.rs:1948`).
Simple fields recognise only `PAGE` and `NUMPAGES`, leaving `REF` and `PAGEREF`
opaque (`crates/rdocx-oxml/src/text.rs:554`). The paginator substitutes page
fields after layout but has no bookmark target map (`crates/rdocx-layout/src/engine.rs:204`).

## Spec reference

- `docs/hld/14-development-backlog.md`, "F-154, Bookmarks and
  cross-references".
- `docs/hld/03-architecture.md`, "What stays put" and "Facade conventions".
- `docs/hld/08-rendering-spec.md`, "The seam that makes this cheap" and the
  Word pagination contract.
- `docs/hld/10-bindings-spec.md`, "Native Word facade stability".
- `docs/hld/12-testing-strategy.md`, "Test taxonomy" and "The hash harness".

## Approach

Add typed bookmark start and end markers to the insertion-aware paragraph
sequence. `Document::bookmarks()` correlates unique ids, exposes each name and
current text in document order, and reports malformed or unmatched markers
without dropping them. `Document::add_bookmark(name, RunRange)` validates the
half-open run range, rejects duplicate or reserved names, allocates the first
free nonnegative id, and inserts markers atomically. The `RunRange` value is
shared with F-148 once that dependency-independent story integrates.

Extend `FieldType` with structured `Ref { bookmark }` and
`PageRef { bookmark }` variants while preserving switches and unsupported
instructions. `REF` resolves to the bookmarked text before shaping. `PAGEREF`
travels as a target-bearing field through line layout. Pagination records the
page on which each bookmark start lands, then the existing post-pagination
field pass substitutes the page number without creating a second paginator.
Missing targets retain the stored display value and emit a stable diagnostic.

The existing TOC helper switches from raw strings to the same typed bookmark
insertion path. No new source file is needed if the shared `RunRange` is
approved in F-148.

## Rejected alternatives

- Resolve `PAGEREF` before pagination. A bookmark's page is not known until
  line breaking and page placement finish.
- Run pagination a second time after substituting page numbers. The current
  field model shapes a placeholder then reshapes it after pagination, so a
  second layout path would introduce inconsistent pagination.
- Search raw XML on every call. That would duplicate the parser and make typed
  insertion indistinguishable from preserved producer bytes.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `ref_and_pageref_instructions_keep_targets_and_switches` | Aliased simple fields parse structured targets, retain unsupported switches, and write fixed prefixes |
| round-trip | `bookmark_markers_keep_range_order_and_unmodelled_neighbours` | Cross-paragraph and same-paragraph bookmarks retain ids, names, positions, and adjacent raw producer bytes |
| regression | `a_bookmark_inserted_over_a_range_is_listed_with_its_text` | The public collection returns one matched bookmark and the text from its half-open run range |
| regression | `ref_and_pageref_resolve_to_the_bookmark_text_and_final_page` | `REF` renders bookmarked text and `PAGEREF` renders the page carrying the bookmark after pagination |
| regression | `missing_and_duplicate_bookmark_targets_fail_without_mutation` | Invalid insertion and unresolved fields do not corrupt the body or create duplicate names |

The **test gate**, from the backlog, is regression. A bookmark inserted over a
range is listed, its text is readable, and a cross-reference to it resolves to
the right page after pagination.

Tests join existing crate-local modules and the rdocx regression binary. Any
render evidence uses bundled deterministic fonts.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/08-rendering-spec.md`
- `docs/hld/10-bindings-spec.md`

Record bookmark ownership, the half-open range and collection API, structured
cross-reference fields, and the single-pagination target substitution pass.

## Risk routing

- Layout, pagination, line breaking, and text shaping. Read HLD 08. Run every
  render check with bundled deterministic fonts and do not record a system-font
  baseline.
- Any parser or serialiser. Read HLD 04 and HLD 06. Add alias, schema-order,
  instruction-switch, range-order, round-trip, and byte-preservation tests.
- Public API of a published crate. Read HLD 10 and the structural rules. The
  bookmark collection and insertion API plus structured field variants are
  additive and story-required. Run affected package dry-runs and archive size
  assertions.

## Hash harness

Expected unchanged across all 49 entries. Existing sample output must stay
byte-identical because TOC bookmarks and fields already produce the same
serialized and rendered result through their new typed path.

## Implementation checklist

- [x] Type bookmark markers and correlate valid ranges without losing malformed XML.
- [x] Add bookmark collection, readable text, and atomic range insertion.
- [x] Route TOC bookmark creation through the typed path.
- [x] Parse and write structured `REF` and `PAGEREF` fields.
- [x] Carry bookmark targets into the existing post-pagination substitution pass.
- [x] Add range, round-trip, field, pagination, and failure regressions.
- [x] Run focused checks plus parser, packaging, and deterministic layout riders.
- [x] Update exactly HLD 03, HLD 08, and HLD 10 at completion.

## Open questions

None. Bookmark insertion reuses F-148's approved half-open `RunRange` API.
