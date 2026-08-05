# F-099, correctness, pass 1

**Reviewed**: working diff, 2 files, 494 changed lines
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, a wide marker moves first-line text past its paragraph margin

`crates/rpptx-render/src/text.rs:74`

The marker slot takes the larger of the shaped marker width and the hanging
indent. A long automatic marker therefore moves first-line text to the right of
`marL`, while wrapped lines still begin at `marL`. The approved indent contract
places the marker at `marL + indent` and text at `marL`, even when the marker is
wider than the hanging slot. Use the hanging width as the marker slot and add a
regression with an automatic marker wider than the hanging indent.

## Smells

None.

## Nitpicks

None.

## Not found

No additional correctness, contract, panic, OOXML, test, or structure findings.
