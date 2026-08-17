# F-157, Word chart part and embedded workbook

**Status**: completed
**Sprint**: S45
**Size**: M
**Depends on**: F-156

## Problem

The Word facade currently understands document relationships for styles,
numbering, images, hyperlinks, notes, and metadata, but not charts. Its save
path serializes only those known owned parts at
`crates/rdocx/src/document.rs:277`, and the package-to-layout path dispatches
image relationships but ignores charts at
`crates/rdocx/src/document.rs:2461`.

Word drawing parsing also assumes every `a:graphicData` payload is a picture.
`CT_Inline` exposes only `embed_id` at
`crates/rdocx-oxml/src/drawing.rs:803`, and the structured writer always emits
`pic:pic` at `crates/rdocx-oxml/src/drawing.rs:957`. A native chart therefore
cannot retain a typed document relationship ID or be authored in schema order,
even though `oxml-sml` already writes the complete embedded workbook.

## Spec reference

- `docs/hld/03-architecture.md`, "The dependency rule" and "What stays put".
- `docs/hld/04-opc-and-packaging.md`, deterministic saves, relationship
  targets, content types, and collision-safe part naming.
- `docs/hld/09-charts-spec.md`, "Why a chart needs three parts", "oxml-sml,
  deliberately minimal", "The ChartML model", and "Cached values are not
  optional".
- `docs/hld/12-testing-strategy.md`, "Test taxonomy" and external application
  acceptance evidence.
- `docs/hld/14-development-backlog.md`, "F-157, Word chart part and embedded
  workbook".

## Approach

Extend the existing Word drawing model in `drawing.rs` with an optional chart
relationship ID alongside the existing picture embed ID. Parsing recognizes a
`c:chart r:id` only inside ChartML `a:graphicData`, while leaving the captured
raw inline or anchor bytes as the sole write-back source for opened documents.
Add focused `CT_Inline::new_chart` and anchored chart construction that emit the
WordprocessingDrawing sequence with `a:graphicData` before the fixed
`c:chart r:id` payload. Structured constructors ensure a new drawing is either
a picture or a chart and reject an ambiguous payload before serialization.

Add private Word package assembly in `Document` for:

- `/word/charts/chartN.xml`
- `/word/embeddings/WorkbookN.xlsx`
- the document-to-chart `CHART` relationship
- the chart-to-workbook `PACKAGE` relationship
- chart and embedded-workbook content-type overrides

Allocate each numbered family independently after its greatest occupied
positive suffix. Stage the package mutation on a clone and publish it only once
the chart XML, workbook bytes, relationship graph, content types, and drawing
payload are valid. F-157 keeps this assembly private and exercises it through a
crate-local test helper with a minimal typed chart. F-158 supplies the public
data-first authoring API.

The same-sprint dependency F-156 runs first, so this implementation consumes
`oxml-chart` and never adds an `rdocx` edge to the deprecated shim.

Add `quick-xml` as a direct private `rdocx` dependency for the package guard
that rejects a chart which already contains `c:externalData`. The scanner
resolves aliases declared on `c:chartSpace`, ignores foreign namespace
lookalikes, and avoids expanding the published `oxml-chart` raw or typed model
API for one facade-only duplicate check.

## Rejected alternatives

- Store charts as images. That loses the editable workbook and fails the
  milestone's native-chart contract.
- Treat a chart relationship as an image `embed_id`. The relationship types and
  graphic-data payloads differ, and overloading the string makes invalid states
  indistinguishable during layout.
- Preserve new chart markup only as opaque raw XML. F-159 needs the relationship
  ID and extents to render the chart, so the behavior-bearing seam must be typed.
- Add a second package abstraction or builder. `Document` already owns the OPC
  package, and the assembly has concrete inputs and one implementation.
- Add a new source module. The existing drawing and document files own the two
  behaviors, and the structural rule favors extending them locally.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| round-trip, gate | `word_chart_part_and_workbook_round_trip` | A saved document reopens with the chart part, document relationship, content type, chart relationship, and workbook bytes intact |
| unit | `word_chart_drawing_writes_schema_order_and_fixed_prefixes` | Inline and anchor chart payloads emit the required `wp`, `a`, `c`, and `r` sequence and reparse the same relationship ID |
| preservation | `opened_chart_drawing_preserves_unmodelled_xml` | Producer attributes, alternate payloads, and unmodelled children remain byte-identical through save |
| regression | `word_chart_parts_allocate_after_sparse_suffixes` | Chart and workbook numbered families independently allocate maximum positive suffix plus one without collision |
| negative | `invalid_chart_package_assembly_is_atomic` | Serialization or workbook failure leaves document XML, package parts, relationships, and content types unchanged |
| differential | `word_opens_native_chart_without_repair` | Pinned Microsoft Word opens the SHA-bound generated document without a repair warning and exposes an editable workbook |

The test gate is round-trip. A document with a chart part saves with the part,
its relationship, its content type, and its embedded workbook, and Word opens
it without repair.

## HLD impact

- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/09-charts-spec.md`

Add the canonical Word chart and embedding paths, the Word relationship graph,
the typed drawing seam, deterministic allocation, and SHA-bound native Word
acceptance evidence.

## Risk routing

- Any parser or serialiser. Read HLD 04 and HLD 06. Add fixed-prefix,
  schema-order, malformed-value, structural round-trip, and byte-preservation
  tests for inline and anchored chart payloads.
- Crate dependency graph and new cross-family uses. Read HLD 03. Confirm
  `rdocx -> oxml-chart`, `rdocx -> oxml-sml`, and the direct `quick-xml`
  parser dependency resolve in affected package dry-runs. Confirm that the
  cross-family edges point inward and that no `oxml-*` crate gains an
  `rdocx-*` dependency.
- Public API of a published crate. Read HLD 10 and the structural rules. The
  drawing additions are additive, and the package helper remains private. Run
  affected package dry-runs and archive size assertions.
- An external oracle comparison. Follow differential-testing guidance. Record
  the exact Microsoft Word version and build, bind the candidate to its SHA,
  and record the no-repair and Edit Data observations as human-action evidence.

## Hash harness

Expected unchanged across all 49 entries. The existing Word sample generator
does not author charts in this story.

## Implementation checklist

- [x] Parse and write typed inline and anchored Word chart relationship payloads.
- [x] Add collision-safe Word chart and embedded-workbook package assembly.
- [x] Preserve opened producer drawing XML as the sole round-trip source.
- [x] Add round-trip, ordering, preservation, collision, and atomicity tests.
- [x] Produce pinned SHA-bound Microsoft Word no-repair evidence.
- [x] Update exactly HLD 04 and HLD 09.

## Open questions

None. The story fixes native editable packaging, the repository already fixes
the relationship and content-type constants, and relationship targets make the
collision-safe `chartN.xml` and `WorkbookN.xlsx` names valid in Word.
