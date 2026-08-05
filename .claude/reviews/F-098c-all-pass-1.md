# F-098c, all, pass 1

**Reviewed**: working diff, 2 files, 647 insertions and 8 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, percentage line spacing ignores the effective first-run font size

`crates/rpptx-render/src/text.rs:185`

The design contract requires percentage spacing to be converted against the
effective first-run font size. The current mapping sends the percentage to
`LineSpacing::Multiple`, which multiplies the line's font ascent and descent
instead. A 20 point first run at 120 percent therefore does not produce a 24
point line height unless that font's natural metrics happen to total exactly 20
points. Pass the effective first-run size into the line-break parameter mapping
and use an exact computed point height. Add a regression that distinguishes the
font size from the natural font metrics.

## Smells

None.

## Nitpicks

None.

## Not found

No additional findings in correctness, contract, panics, OOXML, tests, or
structure. The diff adds no parser or serialiser, no public API, no trait, no
generic parameter, no module, and no file outside the required review record.
