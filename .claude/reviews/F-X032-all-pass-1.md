# F-X032, all aspects, pass 1

**Reviewed**: uncommitted working diff, 2 files and 324 changed lines, with 266 insertions and 58 deletions
**Verdict**: 3 defects, 0 smells, 0 nitpicks

## Defects

### D1, the caller-font regression passes when caller fonts are ignored
`crates/rdocx/src/document.rs:4902`

The fixture supplies the exact Carlito Regular bytes that `FontManager::new`
already bundles, asks the run to use Carlito, then accepts any returned font
with those same family and bytes. An implementation that performs a fresh
ordinary layout and ignores `font_files` produces the same font entry and
passes every assertion in this test. The regression gate therefore does not
prove that a caller-provided font controls shaping or that its bytes are owned
by the returned result. Use input that is distinguishable from the normal
bundled result and assert that the glyph run's `font_id` resolves to that exact
caller font.

### D2, tracked layout is never checked against an existing accepted cache
`crates/rdocx/src/document.rs:4944`

Both tracked calls happen before the accepted cache is populated. The later
accepted pair proves that tracked layout did not populate an empty cache, but
it cannot detect a tracked call that replaces or clears an already populated
accepted cache. That misses the plan's explicit "without populating or
replacing" ownership boundary. Populate the accepted cache first, retain its
`Arc`, run tracked layout, then require the next accepted call to be
`Arc::ptr_eq` to the original and to add no accepted-layout invocation.

### D3, the caller-font options accessor never exercises a non-default view
`crates/rdocx/src/document.rs:4909`

`layout_with_fonts_and_options` is called only with
`RenderOptions::default()`. Removing the assignment of
`options.revision_view` at `crates/rdocx/src/document.rs:3242` leaves all new
tests green, so the public option-taking caller-font path can silently render
the accepted projection for a tracked request. Exercise a document whose
accepted and tracked projections differ, pass `RevisionView::Tracked`, and
assert both the returned `revision_view` and visible layout text.

## Smells

None.

## Nitpicks

None.

## Not found

No additional findings in correctness, contract, panics, OOXML ordering and
preservation, public API shape, cache implementation, `Arc` identity,
caller-font ownership implementation, PDF and raster reuse, threading, or
structure.
