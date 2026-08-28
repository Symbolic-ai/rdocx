# F-200, working, pass 10

**Reviewed**: working diff against
`cf7627aa280c65a245dbed8fbd2988e80dae9201`, 21 files with 3,780 tracked
insertions and 166 tracked deletions, plus the pass 1 through pass 9 review
records
**Verdict**: 3 defects, 0 smells, 0 nitpicks

## Defects

### D1, non-body paragraph containers discard the resolved base direction

`crates/rdocx-layout/src/engine.rs:4450`

The body paragraph path captures `layout_direction` beside its shared block,
but `layout_paragraph_with_source_in_table` passes no direction output. The
table sidecar then assigns `TextDirection::Auto` to every cell paragraph at
`crates/rdocx-layout/src/engine.rs:2511`. The owned table paths also retain only
one outer `Auto` value at `crates/rdocx-layout/src/engine.rs:2001` and
`crates/rdocx-layout/src/engine.rs:2069`, so this is not limited to a cache hit.

Header and footer paragraphs take the same direction-dropping helper at
`crates/rdocx-layout/src/engine.rs:6204`, then rendering supplies `Auto` at
`crates/rdocx-layout/src/paginator.rs:3244`. A table cell, header, or footer
with explicit `w:bidi`, Latin or numeric text, and a source-less leader or
conditional hyphen therefore lays out its first line with the right base but
later reconstructs logical text with the wrong base. A wrapping reflow also
falls back to the legacy breaker when the individual runs are `Auto`. The
paragraph direction contract applies to every Word story, not body paragraphs
only.

### D2, note stories emit visual line order as logical backend order

`crates/rdocx-layout/src/paginator.rs:1440`

`NoteRegistry` keeps only flattened visual `LayoutLine` values after calling
the direction-dropping paragraph helper at
`crates/rdocx-layout/src/notes.rs:176`. `draw_note` then walks each line item in
paint order and appends positioned text in that same order. It does not retain
the paragraph reflow sequence or invoke the logical reconstruction used for
body, table, and header/footer paragraphs. A mixed RTL footnote or endnote with
multiple source spans, a stored field, a leader, or a selected conditional
hyphen consequently exposes visual run order to PDF and SVG extraction. The
individual rich run text remains logical, but the paragraph-wide searchable
sequence does not.

### D3, the cache regressions construct semantics that production never supplies

`crates/rdocx-layout/src/paginator.rs:4143`

The body and table test runs the same manually assembled `ParagraphBlock`
twice and changes only a diagnostic string and source id. It manually supplies
`RightToLeft` semantics at `crates/rdocx-layout/src/paginator.rs:4175`, while
real table semantics are always `Auto`. The header test similarly constructs a
`ParagraphView` with semantics at `crates/rdocx-layout/src/paginator.rs:4277`,
although `render_hf_blocks` always uses `semantics: None`. The direct rebinding
test at `crates/rdocx-layout/src/engine.rs:9780` uses a body paragraph block and
does not pass through the header/footer cache, a leader, or backend rendering.

These tests correctly prove the private normalizer and the isolated
`HyphenatedText` rebinder, but they do not prove the requested cache-safe table
or header/footer behavior. Both stay green while D1 remains active, and no
end-to-end regression covers the related-story source and extraction contract.

## Smells

None.

## Nitpicks

None.

## Not found

- Pass 9 D1 is closed for shared body paragraphs. Retained cache spans are
  normalized through the current `ParagraphSemantics` source node before exact
  matching, source-less items remain unattributed, and positioned visual
  origins do not move.
- `rebind_paragraph_source` now includes `InlineItem::HyphenatedText`, so an
  owned cloned header or footer block no longer retains the cache source node.
  The defects above concern missing direction transport and end-to-end proof,
  not the new rebinding operation itself.
- The pass 1 through pass 8 parser closures remain intact. Unsupported toggle
  attributes, valid and malformed occurrence order, canonical final carriers,
  typed clearing, schema placement, namespace handling, and repeated
  serialization retain their reviewed behavior.
- Natural UAX 9 levels, line-local L1 then L2 ordering, explicit run overrides,
  conditional-hyphen demotion, inline objects, stored fields, and private body
  drawing reflow remain on the shared bidi path. No new panic, unchecked public
  slice, arithmetic overflow, or suppressed parser error was found.
- PDF rich-run `ActualText`, raster glyph placement, SVG searchable text, and
  PowerPoint visual painting retain the reviewed per-run contracts. The note
  defect is paragraph-wide ordering before those backend consumers.
- Public `ParagraphReflow` keeps its established shape. No new public API,
  module, dependency, reverse crate edge, wrapper, trait, generic, or feature
  flag was introduced.
- Exactly the five plan-listed HLD files changed and describe current intended
  behavior. The recorded deterministic 49-of-49 hash, five-page output binding,
  package archive ceiling, portability, and supply-chain evidence are
  internally consistent. Those gates do not cover D1 or D2.
