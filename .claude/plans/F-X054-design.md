# F-X054, Integrate PRs 47 through 52

**Status**: completed
**Sprint**: S56
**Size**: L
**Depends on**: F-X033

## Problem

The reader exposes exact direct body order through `Document::body_items`,
while older paragraph and table accessors retain recursive flattened semantics
at `crates/rdocx/src/document.rs:53` and
`crates/rdocx/src/document.rs:1072`. It does not expose the same direct order
for cell, paragraph, hyperlink, or run children. The low-level models already
retain the needed ownership and raw boundaries in
`crates/rdocx-oxml/src/table.rs:1229` and
`crates/rdocx-oxml/src/text.rs:305`.

Unknown numbering values are currently converted to `Decimal`, and public
`ST_NumberFormat` is `Copy` at `crates/rdocx-oxml/src/numbering.rs:2065`.
Visible `w:t` and `w:delText` parse failures are erased through
`unwrap_or_default` at `crates/rdocx-oxml/src/text.rs:571`. PRs 47 through 52
propose these outcomes, but the backlog requires namespace-aware classification
through the existing parser, no fabricated raw bytes, non-exhaustive open item
enums, complete consumer reconciliation, and exact attribution.

## Spec reference

- `docs/hld/03-architecture.md`, "Crate-level conventions" and "Facade
  conventions".
- `docs/hld/04-opc-and-packaging.md`, numbering preservation and package
  integrity.
- `docs/hld/06-presentationml-model.md`, "Preservation strategy".
- `docs/hld/10-bindings-spec.md`, "Native Word facade stability".
- `docs/hld/12-testing-strategy.md`, "Test taxonomy" and named regressions.
- `docs/hld/14-development-backlog.md`, "F-X054, Integrate PRs 47 through 52".
- PRs [47](https://github.com/tensorbee/rdocx/pull/47),
  [48](https://github.com/tensorbee/rdocx/pull/48),
  [49](https://github.com/tensorbee/rdocx/pull/49),
  [50](https://github.com/tensorbee/rdocx/pull/50),
  [51](https://github.com/tensorbee/rdocx/pull/51), and
  [52](https://github.com/tensorbee/rdocx/pull/52).

## Approach

Land one current-tree hardened implementation rather than six blind merges.
All six records are initially classified as hardened equivalents. Record the
final direct or hardened classification and exact deviation for each one.

Retain PR 47's additive cell outcome with a new non-exhaustive
`CellItemRef<'a>` carrying direct paragraphs, tables, content controls, and
unsupported XML. `CellRef::items` merges typed content and indexed raw XML in
source order. Existing recursive `CellRef::paragraphs` stays unchanged.

Retain PR 48's run outcome with a non-exhaustive `RunItemRef<'a>` for text,
deleted text, tab, break, drawing, field, notes, comment references, and
unsupported XML. Add borrowed `DrawingRef`, `FieldRef`, and `BreakKind` facts.
Use the existing retained field instruction, cache, and dirty state rather than
adding a second lossy PAGE and NUMPAGES taxonomy.

Retain PR 49's direct `ParagraphRef::items` and `HyperlinkRef::items` outcomes
with non-exhaustive enums. Expose runs, hyperlinks, content controls,
revisions, comment and bookmark boundaries, and unsupported XML without
flattening. Mirror complete serializer ordering so multiple sidecar kinds at
one boundary are each reported once. Existing flattened accessors stay intact.

Retain PR 50's compatibility outcome with non-exhaustive
`BodyContentRef<'a>` and `UnsupportedXmlRef<'a>`. Modeled unsupported facts
return `None` from `raw_xml()`. Raw facts borrow their exact bytes.
`UnsupportedXmlRef` exposes optional qualified name and namespace URI, required
local name, and a child-content fact. Derive these with `quick_xml` and resolve
raw-local plus inherited retained namespace declarations. Keep this narrow
facade beside `BodyContentRef` in `document.rs`, with no new module.

Retain PR 51's producer value as `ST_NumberFormat::Other(String)`, change
`to_str` to borrow, and remove `Copy`. Update every current consumer in
`rdocx-oxml`, `rdocx-layout`, `rdocx-html`, `rdocx`, and RTF export. Unknown
formats round-trip unchanged and do not invent decimal markers. Exporters that
cannot represent them return their existing lossy diagnostic. Reconcile the
independent current-base Python error match so the S55 `Html` and `Odt` errors
map to the existing generic `RdocxError`.

Retain PR 52's fail-closed parser change for both `w:t` and `w:delText`.
Propagate text decode failures as `OxmlError::InvalidValue` instead of
publishing empty visible content. Valid entity decoding remains unchanged.

Every open-ended public item enum is non-exhaustive from birth. Iterators borrow
the retained tree and allocate at most linearly in direct children and sidecar
counts. Add no trait, generic parameter, crate, feature, module, or file.

The ordered-reader and unsupported-fact APIs are additive. PR 51 is an
intentional pre-1.0 source incompatibility because the enum gains a data-bearing
variant, loses `Copy`, and returns a value-backed string. This is the one named
compatibility change for v0.10.0. Python behavior stays compatible because new
native errors map to an existing exception class.

Do not merge, comment on, or close the GitHub pull requests during F-X054.
F-X055 owns release-bound comments and closure after v0.10.0 verifies.

## Rejected alternatives

- Merging the GitHub PRs would bypass the repository lifecycle and misstate
  hardened-equivalent integration.
- Cherry-picking unchanged misses namespace, non-exhaustive, collision, deleted
  text, and current-tree reconciliation requirements.
- Adding `unsupported_xml.rs` creates an unapproved module for a narrow facade.
- Returning empty bytes for modeled facts fabricates raw XML.
- Keeping `Decimal` plus a parallel raw attribute leaves the public semantic
  model false.
- A second field-kind enum duplicates richer retained field facts.
- Changing established flattened accessors violates compatibility.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression, gate | `ordered_reader_items_keep_every_direct_child_and_preserved_boundary` | Exact body, cell, paragraph, hyperlink, and run order includes all typed variants and raw boundaries. |
| regression | `ordered_reader_items_resolve_aliases_without_flattening_containers` | Word aliases type only Word children, foreign lookalikes remain raw, and nested containers stay at direct boundaries. |
| regression | `modeled_unsupported_body_facts_do_not_invent_raw_xml` | Modeled facts have names but no raw bytes, while raw content retains exact bytes and inherited namespace identity. |
| regression | `producer_defined_number_formats_survive_save_and_reopen` | `Other("chicago")` parses, serializes, reopens, and remains identical without decimal substitution. |
| unit | `producer_defined_number_formats_do_not_invent_layout_markers` | Unknown formats emit no decimal marker and modeled formats remain unchanged. |
| regression | `undecodable_ordinary_and_deleted_text_are_rejected` | Invalid ordinary and deleted visible text fails while valid entities decode. |
| regression | `legacy_flattened_accessors_keep_their_recursive_results` | Existing accessors retain their prior recursive and flattened semantics. |
| round-trip | `ordered_reader_source_survives_save_and_reopen` | Ordered public facts and every raw subtree remain equal after save and reopen. |
| unit | `import_errors_map_to_the_generic_public_error_class` | HTML and ODT errors map to the existing Python exception. |
| documentation | stable API review from `v0.9.0` | Every addition is documented and PR 51 is the only intentional existing-source incompatibility. |

The **test gate** is regression. Add public regressions to the existing
`crates/rdocx/tests/regression_test.rs` binary. Low-level parser tests remain in
their source modules.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`

## Risk routing

- **Any parser or serialiser**. Read HLD 04 and HLD 06 preservation strategy.
  Add alias, inherited-scope, fixed-prefix, schema-order, raw-subtree, and
  save-reopen tests for unsupported XML, numbering, and visible text.
- **Public API of a published crate**. State the additive `rdocx` surface and
  source-incompatible `rdocx-oxml` change. Run documentation warnings, inspect
  the stable API diff from v0.9.0, and run patched package and archive gates.
- **Layout, pagination, line breaking, text shaping**. Unknown numbering no
  longer invents a marker. Run focused layout numbering tests and the
  deterministic 49-entry harness without changing its baseline.
- **WASM or PyO3 bindings**. The Python error match changes. Run the Python
  binding checks and both wasm32 targets with the required workspace excludes.

No crate graph, oracle, feature flag, unit conversion, colour, release script,
new module, or file-move row is triggered.

## Hash harness

Expected unchanged across all 49 entries. The facade projections are read-only,
invalid-input rejection affects no sample, and samples use no producer-defined
numbering value. Any delta blocks integration.

## Implementation checklist

- [x] Reconfirm all six PR heads, author identity, and checks.
- [x] Add non-exhaustive borrowed cell, run, paragraph, hyperlink, and body facts.
- [x] Mirror complete retained sidecar order and keep legacy accessors unchanged.
- [x] Classify unsupported XML through `quick_xml` and inherited namespace scope.
- [x] Add `ST_NumberFormat::Other(String)`, remove `Copy`, borrow in `to_str`, and update every consumer.
- [x] Preserve unknown numbering without inventing markers or hiding lossy exports.
- [x] Map S55 HTML and ODT Python errors to the existing exception.
- [x] Reject undecodable ordinary and deleted visible text.
- [x] Add all source-built regressions to existing binaries and modules.
- [x] Review the complete stable API diff and record the PR 51 break.
- [x] Record each PR's classification, deviation, link, and contributor credit.
- [x] Leave all GitHub mutations to F-X055 after publication.
- [x] Run focused checks, all risk riders, full verification, and the unchanged harness.
- [x] Update exactly the listed HLD files.

## Open questions

None. The backlog fixes behavior, compatibility, and attribution. Modeled facts
use no fabricated raw bytes, unknown numbering has no invented marker, and all
six records remain hardened equivalents unless the final diff proves a patch
landed without semantic deviation.
