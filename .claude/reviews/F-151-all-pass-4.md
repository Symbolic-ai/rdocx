# F-151, all, pass 4

**Reviewed**: complete remediated working-tree diff against `HEAD`, 12 files, 955 changed lines, with 908 additions and 47 deletions
**Verdict**: 3 defects, 0 smells, 0 nitpicks

## Defects

### D1, nested-only revision wrappers lose their visible content
`crates/rdocx-oxml/src/revision.rs:491`
`crates/rdocx-layout/src/engine.rs:104`
`crates/rdocx-layout/src/engine.rs:187`

The parser uses `RevisionContent::Marker` whenever an outer wrapper has no
direct runs, while retaining any nested revisions separately. The pass-3
remediation correctly makes a genuinely empty marker invisible, but the run
projection now returns before visiting those retained children and the
visibility predicate also returns false. An outer insertion containing only a
nested deletion or move therefore contributes nothing to tracked view. An
included nested insertion or move destination can also disappear from accepted
view. This violates the contract to flatten nested wrappers once in preserved
document order.

### D2, PAGEREF targets move ahead of revisions at the same run boundary
`crates/rdocx-layout/src/engine.rs:668`
`crates/rdocx-layout/src/engine.rs:979`
`crates/rdocx-oxml/src/text.rs:1194`

Bookmark parsing now retains `raw_before`, and REF text compares that value with
each projected revision. Target marker emission still groups only by
`run_index` and inserts every target before the first projected run at that
boundary. When a bookmark starts after an included revision but before the next
ordinary run, its zero-width PAGEREF target is therefore placed before the
revision. If that revision text crosses a page boundary, PAGEREF resolves to the
revision's earlier page instead of the page where the bookmark actually starts.

### D3, auxiliary paragraph render paths drop tracked change bars
`crates/rdocx-layout/src/paginator.rs:1964`
`crates/rdocx-layout/src/notes.rs:160`

Header and footer paragraphs go through `layout_paragraph`, so their selected
revision text and tracked decorations are computed and their
`has_visible_revision` flag is set. `render_hf_blocks` renders only their lines
and never emits the bar. Footnote and endnote layout discards each
`ParagraphBlock` after extracting its lines, which loses the same flag before
pagination. A tracked insertion, deletion, or property-only change in any of
these paragraphs therefore renders without the required margin bar.

## Smells

None.

## Nitpicks

None.

## Not found

Pass-3 D1 is resolved for genuinely empty revision wrappers. Panic safety,
OOXML preservation and schema ordering, and structural-rule compliance were
checked and produced no findings. The test aspect produced no independent
finding beyond the uncovered inputs described by D1 through D3.
