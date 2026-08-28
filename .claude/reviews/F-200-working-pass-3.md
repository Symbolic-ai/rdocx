# F-200, working, pass 3

**Reviewed**: working diff against
`cf7627aa280c65a245dbed8fbd2988e80dae9201`, 20 files with 1,819 tracked
insertions and 127 tracked deletions, plus the pass 1 and pass 2 review records
**Verdict**: 3 defects, 0 smells, 0 nitpicks

## Defects

### D1, clearing a duplicated typed direction toggle resurrects the earlier value

When a later valid `w:bidi` or `w:rtl` occurrence replaces the attribute
carrier, `record_modeled_toggle_candidate` removes the carrier flag from every
earlier same-slot occurrence and turns it into ordinary raw XML at
`crates/rdocx-oxml/src/properties.rs:203`. The paragraph and run writers filter
only the one occurrence that still has the carrier flag at
`crates/rdocx-oxml/src/properties.rs:1907` and
`crates/rdocx-oxml/src/properties.rs:2088`. If a caller parses two valid
occurrences and then clears the public scalar with `ppr.bidi = None` or
`rpr.rtl = None`, the generated occurrence disappears but the earlier valid
toggle is replayed as raw XML. Reopening the result therefore restores the old
direction instead of observing the clear. The duplicate regression proves
unchanged repeated serialization at `crates/rdocx-oxml/src/properties.rs:2773`,
but does not exercise typed mutation. Pass 2 D1 is not closed at this public
mutation boundary.

### D2, drawing reflow discards the resolved paragraph base direction

Initial Word line breaking passes the resolved `layout_direction` into the
multilingual breaker at `crates/rdocx-layout/src/engine.rs:5161`, but the
retained `ParagraphReflow` stores only items and line parameters at
`crates/rdocx-layout/src/engine.rs:5196`. When a wrapping drawing causes the
paragraph to be broken again, the paginator unconditionally supplies
`TextDirection::Auto` at `crates/rdocx-layout/src/paginator.rs:1972`. An
explicit LTR or RTL paragraph can consequently switch to a content-inferred
base after fitting changes. Line-local whitespace, demoted hyphenation spans,
numbering markers, and inline objects then receive levels from the wrong base,
even though alignment and indents still use the authored direction. This
violates the paragraph-wide `w:bidi` contract and leaves the drawing-reflow
interaction untested.

### D3, stored field text ignores its run direction override

The stored-field path records the effective run direction in
`multilingual_styles` at `crates/rdocx-layout/src/engine.rs:4953`, but creates
the visible field segment with `TextDirection::Auto` at
`crates/rdocx-layout/src/engine.rs:4966`. `multilingual_candidate` then excludes
every segment carrying `field_kind` at `crates/rdocx-layout/src/engine.rs:6504`,
so `shape_word_multilingual_items` never applies the recorded style to that
text. A cached or computed field result inside a `w:rtl` run therefore follows
only the paragraph's natural direction. The defect also reaches eligible
header and footer fields because they use the same paragraph projection. This
breaks the approved character-level direction contract and the retained-field
interaction inherited from F-199.

## Smells

None.

## Nitpicks

None.

## Not found

- Pass 1 D3 and the ordinary conditional-hyphenation case from pass 2 D2 are
  closed for non-field spans. Explicit directions retain natural numeric
  levels, line whitespace receives the L1 adjustment, and the structural tests
  cover both rich and demoted Latin runs.
- Pass 1 D4 and pass 2 D3 are closed. Word pagination now groups both legacy
  and rich positioned text by exact source range and restores logical payload
  order without changing visual origins at
  `crates/rdocx-layout/src/paginator.rs:2607`.
- Pass 1 D5 and D6 are closed. One inferred base drives absent-direction
  alignment and line layout, while tabs, images, groups, and figures enter the
  bidi paragraph as neutral object replacements at
  `crates/oxml-layout/src/line.rs:642`.
- Malformed multilingual values continue to use the shared validation contract
  before PDF, raster, or SVG indexing. No new untrusted-input panic or unchecked
  source-range conversion was found.
- Direction fields remain in serialized paragraph identity, and raw property
  sidecars keep the affected blocks out of unsafe restart reuse. No separate
  warm-versus-fresh, cached-tail, note, header, or footer invalidation defect
  was found beyond D3's shared field projection.
- DrawingML direction remains typed and namespace-aware. PowerPoint retains
  paragraph-wide bidi state through forced breaks, and the documented
  quarter-turn vertical approximations are unchanged.
- The intentional pre-1.0 exhaustive field additions are documented in the
  bindings HLD. No new module, dependency, public abstraction, reverse crate
  edge, or forwarding-only wrapper was introduced.
- Exactly the five plan-listed HLD files changed. The recorded 49-of-49 hash,
  five-of-five raw oracle, corpus, portability, package, archive, and supply
  chain evidence is internally consistent. No evidence threshold or baseline
  was weakened.
