# F-X037, Trace Word glyphs to source paragraphs

**Status**: completed
**Sprint**: S51
**Size**: M
**Depends on**: F-009, F-151

## Problem

`GlyphRun` carries positioned glyphs and its displayed text but no identity for
the Word paragraph that produced it. An external viewer or editor must match
short displayed strings back to document text. Repeated phrases, CJK line
segments, table cells, headers, footers, and notes make that reconstruction
ambiguous or impossible.

The format-neutral layout crate owns shaping, breaking, and positioned glyph
output. The Word layout crate owns story traversal and can name the source
paragraph. Provenance therefore needs a small neutral span carried through the
shared pipeline plus a Word-specific result-local side table. A fixed-width
Word path cannot be encoded safely in one integer because table nesting is
unbounded and header and footer relationship identifiers are variable length.

## Spec reference

- `docs/hld/03-architecture.md`, "What stays put" and dependency direction.
- `docs/hld/08-rendering-spec.md`, "The seam that makes this cheap",
  "Performance", and "Word revision views".
- `docs/hld/10-bindings-spec.md`, the intentional 0.8 low-level Rust boundary
  and native Word facade stability.
- `docs/hld/12-testing-strategy.md`, "Test taxonomy" and "The hash harness".
- `docs/hld/14-development-backlog.md`, "F-X037, Trace Word glyphs to source
  paragraphs".

## Approach

Add format-neutral result-local identity and exclusive Unicode-scalar ranges in
`oxml-layout`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SourceNodeId(NonZeroU32);

impl SourceNodeId {
    pub const fn new(value: u32) -> Option<Self>;
    pub const fn get(self) -> u32;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceSpan {
    pub node: SourceNodeId,
    pub char_start: u32,
    pub char_end: u32,
}
```

Add `source: Option<SourceSpan>` to exhaustive public `TextSegment` and
`GlyphRun`. Both line-splitting sites advance `char_start` and `char_end` by
counting Unicode scalar values before their byte split. The range is not a
UTF-8 byte range, UTF-16 range, grapheme range, or glyph-cluster map.

Add Word-specific source ownership in `rdocx-layout`:

```rust
pub enum WordStory {
    Document,
    Header { relationship_id: String },
    Footer { relationship_id: String },
    Footnote { id: i32 },
    Endnote { id: i32 },
}

pub struct WordSourcePath {
    pub story: WordStory,
    pub children: Vec<usize>,
}

pub struct WordLayoutResult {
    pub layout: oxml_layout::LayoutResult,
    pub revision_view: RevisionView,
    source_nodes: Vec<WordSourcePath>,
}

impl WordLayoutResult {
    pub fn source_node(&self, id: SourceNodeId) -> Option<&WordSourcePath>;
    pub fn into_layout_result(self) -> oxml_layout::LayoutResult;
}
```

`children` indexes the current modeled content vector at each nesting level.
A direct body paragraph uses `[body_item]`. A table-cell paragraph uses
`[body_item, row, cell, cell_item]`. Nested tables continue with another row,
cell, and cell-item sequence. Header, footer, and note stories start at their
paragraph index. IDs are one-based side-table indexes local to one result and
must never be compared across layouts.

Add `layout_document_with_provenance` and its deterministic counterpart. Keep
the existing `layout_document` functions source-compatible by delegating and
discarding the map. F-X032 will expose `WordLayoutResult` through the document
facade after this story lands.

Ordinary paragraph text, tracked deleted text, tables, nested tables, headers,
footers, footnote bodies, and endnote bodies receive provenance. Every
attributed run must equal the selected paragraph projection at its recorded
range. Repeated layout of the same header or note reuses the same node.
Generated list markers, tab leaders, bookmark markers, note-reference labels,
dynamically evaluated fields, and non-bijective display transformations use
`None`. Content controls currently skipped by layout remain skipped. This story
does not add rendering behavior.

## Rejected alternatives

- Encode a Word path in `u64`. Variable relationship identifiers and arbitrary
  table depth cannot fit without collision or truncation.
- Store opaque Word bytes or strings in `LayoutResult`. That weakens the
  format-neutral boundary and forces callers to depend on an undocumented
  codec.
- Make `LayoutResult` generic over source path. There is only one current
  provenance implementation and no second instantiation to justify a generic.
- Attribute generated text approximately. A false edit location is worse than
  a truthful `None`.
- Add glyph-cluster-to-character mapping. This story provides exact run-level
  provenance, not caret geometry inside a ligature.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `line_splitting_preserves_contiguous_unicode_source_ranges` | ASCII, emoji, and CJK splits advance exclusive scalar ranges at both split sites |
| regression, gate | `every_sourced_glyph_run_resolves_to_its_exact_word_text` | Body, table, nested-table, header, footer, footnote, and endnote paths resolve and their projected scalar slice equals run text |
| regression | `repeated_text_and_repeated_stories_keep_distinct_source_nodes` | Duplicate phrases do not alias and repeated header or note layout reuses its node |
| regression | `accepted_and_tracked_views_record_projection_local_ranges` | Inserted and deleted text ranges address the selected revision projection |
| regression | `generated_or_transformed_text_remains_unattributed` | List markers, leaders, labels, evaluated fields, and non-bijective transformations use `None` |
| compatibility | existing low-level layout functions | Existing callers still receive byte-identical `LayoutResult` output |
| integration | caller-font and deterministic provenance variants | Both variants return complete maps and use the exact fonts that shaped their runs |
| boundary | public struct literals and WASM | The planned 0.4 and 0.8 source break is explicit and both WASM targets compile |

The **test gate** is regression. Every attributed glyph run resolves to one
exact Word paragraph path and Unicode-scalar range whose selected projection
equals the run text across every supported story and both revision views. Both
split stages preserve contiguous ranges, generated text remains unattributed,
and all 49 output hashes remain unchanged.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/08-rendering-spec.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- **Layout, pagination, line breaking, text shaping**. Read HLD 08. Use
  deterministic fonts for provenance assertions and require the complete hash
  harness to remain unchanged.
- **Public API of published crates**. Read HLD 10 and the structural rules.
  `TextSegment` and `GlyphRun` field additions are an intentional pre-1.0
  source break for incubating 0.4.0 and stable 0.8.0. Run package dry-runs and
  enforce the 10 MiB archive ceiling.
- **WASM or PyO3 bindings**. Read HLD 10. Python surfaces stay unchanged. Run
  both WASM target checks and the binding-excluded workspace test command.

No parser, serializer, external oracle, dependency, feature flag, new module,
or new file is introduced.

## Hash harness

Expected unchanged across all 49 entries. Provenance is metadata and must not
move glyphs, change fonts, or alter generated documents and renders.

## Implementation checklist

- [x] Add neutral source ids and exclusive scalar spans.
- [x] Carry spans through shaping and both splitting stages.
- [x] Allocate deterministic result-local Word paragraph paths.
- [x] Cover body, nested tables, headers, footers, footnotes, and endnotes.
- [x] Keep generated and non-bijective text truthfully unattributed.
- [x] Add provenance-returning normal and deterministic layout functions.
- [x] Preserve existing low-level layout API output and hash behavior.
- [x] Run public-package, WASM, full verification, and archive-size riders.
- [x] Update exactly the HLD files listed above.
- [x] Record the low-level 0.4.0 and 0.8.0 migration in release notes.

## Open questions

None. The user requested issue 38 in S51. Paths use a typed Word side table,
ranges use Unicode scalar indices, generated text remains unattributed, and
content currently skipped by layout stays outside this story.
