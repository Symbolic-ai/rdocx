# F-098c, all, pass 2

**Reviewed**: revised working diff, 2 implementation files, 682 insertions and 8 deletions
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, the justified alignment branch has no regression

`crates/rpptx-render/src/text.rs:235`

The alignment regression covers left, centre, right, and distributed behavior,
but it never selects `ParagraphAlignment::Justified`. Removing this branch or
stretching the last line would leave every test green. Add assertions that a
non-final line distributes its remaining width across word gaps and that the
last line retains its natural width.

### D2, production draw order and unclipped overflow are not proved

`crates/rpptx-render/src/lib.rs:314`

The private stacking tests bypass `lower_shape`, so they do not prove that the
production group appends text after the shape path or leaves the group clip
unset. Reversing this append or adding a content-box clip would leave the test
suite green while violating two explicit design requirements. Add one
production-path regression that inspects path-before-text order, an overflowing
glyph run, and `clip: None` on the shape group.

## Smells

None.

## Nitpicks

None.

## Not found

No additional findings in correctness, contract, panics, OOXML, tests, or
structure. Pass 1's percentage line-spacing defect is fixed by an exact
first-run-size computation and a distinguishing regression.
