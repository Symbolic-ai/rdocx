# F-X013a, correctness, pass 2

**Reviewed**: the uncommitted working tree after the pass 1 remediation. One
source file, `crates/rdocx-layout/src/engine.rs`, 16 lines of implementation and
216 added lines of test.
**Verdict**: 0 defects, 0 smells, 2 nitpicks

## Defects

None.

### D1 from pass 1, resolved
`crates/rdocx-layout/src/engine.rs:419`

The item loop now yields an advance for every `LineItem` variant rather than
only `Text` and `Marker`. `Tab` and `Image` contribute their `width` without
drawing, so text after them lands in the right place. The match is exhaustive
over the four variants, so a variant added later fails to compile here rather
than silently reintroducing the defect.

Covered by `a_tab_inside_a_footnote_still_advances_the_text_after_it`, which
lays out the same note with and without a tab between its runs and asserts the
trailing run shifts right. That test fails against the pass 1 code.

## Smells

None outstanding.

### S1 from pass 1, partially addressed and re-scoped

The note path still hardcodes its start rather than honouring `line.indent_left`
and `jc`. That remains true and remains out of scope.

What pass 1 missed, and what validating against the contributor's own sample
document exposed, is that the hardcoded indent was not merely cosmetic. Notes
were line-broken at `geometry.content_width()` and drawn at
`margin_left + 12.0`, so every note line overran the right margin by exactly the
indent. Invisible while every segment was stacked at one x, and plainly visible
the moment the advance was fixed.

Now a single `FOOTNOTE_INDENT` constant at `engine.rs:281` feeds both the break
width and the draw position, so the two cannot disagree. Covered by
`a_long_footnote_does_not_overrun_the_right_margin`, which asserts no note glyph
reaches past the right margin and asserts the note actually wrapped, so the test
cannot pass vacuously.

Taking this into F-X013a rather than deferring it was a judgement call. It is
one constant and one subtraction, it shares a root cause with the defect the
story exists to fix, and deferring it would have shipped a story whose stated
outcome is legible footnotes while leaving them running off the page.

### S2 from pass 1, unchanged and still recorded

The harness cannot see this code path. No corpus document holds a footnote, so
the flat 28 of 28 result says nothing about note rendering. Unchanged by this
pass, recorded in the design plan, and it applies to F-X013b and F-X013c too.

Partially mitigated in practice: the story was validated end to end against
`sample1.docx`, the document the external PR used for its screenshots. That is
manual evidence rather than a gate, which is exactly why S2 stays open.

## Nitpicks

- `crates/rdocx-layout/src/engine.rs:1546`, `footnote_glyph_x` still selects
  note glyphs by y position relative to the separator. Carried from pass 1.
  Sound while the test documents are short, and F-X013b makes the predicate
  reliable.
- `crates/rdocx-layout/src/engine.rs:1575`, the `>= 4` length assertion is still
  looser than the exact count the test produces. Carried from pass 1.

## Not found

Checked and produced nothing:

- **panics**. The implementation adds one `f64` accumulator, one constant and
  one subtraction. `content_width() - FOOTNOTE_INDENT` is not guarded against a
  page narrower than 12 points, which is not reachable through any section
  geometry the parser can produce, and `break_into_lines` already clamps a
  non-positive width.
- **ooxml**. No parser, serialiser, prefix or element ordering touched.
- **structure**. No new trait, generic, wrapper, crate, module, file or feature
  flag. One private constant replacing a magic number used in two places, which
  reduces the cases a reader must reconcile rather than adding a place to look.
- **contract**. The implementation matches the plan's approach. The plan's
  claim about the body path was wrong and has been corrected in the plan itself
  alongside the code.
- **tests**. All five note tests fail against the unfixed code except
  `a_single_segment_footnote_keeps_its_original_position`, which is an
  intentional guard that must pass both before and after.
