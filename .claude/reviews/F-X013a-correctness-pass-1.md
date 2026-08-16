# F-X013a, correctness, pass 1

**Reviewed**: the uncommitted working tree. One source file,
`crates/rdocx-layout/src/engine.rs`, 6 added lines of implementation and 138
added lines of test. The other four changed files are sprint scaffolding for
S41 and carry no product code.
**Verdict**: 1 defect, 2 smells, 2 nitpicks

## Defects

### D1, the advance ignores tabs and images, so it does not mirror the body path
`crates/rdocx-layout/src/engine.rs:414`

The fix advances `x` only inside the `LineItem::Text(seg) | LineItem::Marker(seg)`
arm. The body path it claims to mirror advances for four item kinds, not two:

- `paginator.rs:926`, `x += effective_width` for `Text` and `Marker`
- `paginator.rs:946`, `x += width` for `Tab`
- `paginator.rs:967`, `x += width` for `Image`

The design plan asserts that non-text items "do not advance the cursor, which
matches the body path's treatment of them". That statement is false, and it was
written without checking `paginator.rs`.

Trigger: a footnote whose paragraph contains a tab stop or an inline image. The
tab or image is not rendered at all, because the `if let` filters it out, and
every segment after it is then drawn `width` points too far left. A note like
`1.\tSee above` loses its tab gap and pulls the following text into the marker.

This is narrower than the defect being fixed, since it needs a tab or image
inside a note rather than any multi-run note at all, but it is the same class of
error and the story's own framing is that footnote lines advance the way body
lines do. Leaving it means the fix is correct for the common case and silently
wrong for the next one.

## Smells

### S1, footnote lines ignore their own indent and justification
`crates/rdocx-layout/src/engine.rs:406`

The note path hardcodes `let indent = 12.0` and starts every line at
`geometry.margin_left + indent`. The body path computes its start from
`line.indent_left` plus a `jc`-dependent offset at `paginator.rs:764` through
`paginator.rs:774`, honouring `Center`, `End` and `Justify`.

A footnote paragraph carrying an indent or a non-left justification therefore
renders left-aligned at a fixed offset. This predates the story and is out of
its scope, but the two paths continuing to diverge is what produced the defect
being fixed here in the first place. The durable answer is one shared
line-placement routine that both callers use. Worth an F-ID rather than a patch.

### S2, the harness cannot see this code path at all
`scripts/hash_baseline.json`

None of the seven corpus documents contains a footnote, and no generator under
`crates/rdocx/examples/` emits one. `render_page_footnotes` is therefore
unexercised by the 28-entry harness, so its "28 of 28 match" result carries no
information about note rendering.

This matters beyond this story. F-X013b and F-X013c will both report a flat
harness for the same reason, and a reader who does not know this will read that
as evidence those stories changed no output. Recorded in the design plan's hash
harness section. Closing it means adding a corpus document with notes.

## Nitpicks

- `crates/rdocx-layout/src/engine.rs:1524`, `footnote_glyph_x` identifies note
  glyphs as those below the separator's y. That is exactly the predicate
  F-X013b exists to make true, so the helper is sound today only because the
  test documents are short enough not to overlap. Selecting on a note-owned
  field would be sturdier.
- `crates/rdocx-layout/src/engine.rs:1553`, `assert!(xs.len() >= 4)` is looser
  than the test needs. The count is exactly 4 and asserting equality would
  catch an unexpected extra glyph run rather than passing through it.

## Not found

Checked and produced nothing:

- **panics**. The implementation adds one `f64` accumulator and no indexing,
  slicing, unwrap or expect. The `expect` and `pages[0]` uses are confined to
  test code, where a panic is the intended failure mode.
- **ooxml**. The diff touches no parser, no serialiser, no namespace prefix and
  no element ordering. Nothing to check.
- **structure**. No new trait, generic parameter, wrapper, crate, module, file
  or feature flag. One local `mut` binding in an existing function, and tests
  added to the existing `mod tests` rather than a new `tests/` binary, per the
  extra-link-target rule in `CLAUDE.md`.
- **contract**. The implementation does what the plan's approach section
  described and nothing beyond it. The plan's factual claim about the body path
  is wrong, which is D1, but the code written matches what was promised.
