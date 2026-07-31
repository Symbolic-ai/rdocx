# F-043, correctness, pass 1

**Reviewed**: working diff, 5 files, 653 additions and 51 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, the pattern matrix omits the page coordinate flip

`crates/oxml-pdf/src/writer.rs:241`

The registry stores only the accumulated group transform in the pattern
matrix, while page content enters top-left coordinates through the global
vertical flip at `crates/oxml-pdf/src/writer.rs:844`. Pattern coordinates are
therefore inverted vertically relative to path coordinates. The rotated gate
exposes this: the red left endpoint rotates to the visual top under the
declared positive 90 degree transform, but the implementation rasterises blue
at the top and the assertion at `crates/oxml-pdf/src/writer.rs:1772` accepts
that inversion. Compose the accumulated element transform with the page flip
for the pattern matrix, then restore the expected red-top and blue-bottom
samples.

## Smells

None.

## Nitpicks

None.

## Not found

No additional contract, panic, OOXML, test-gate, or structure findings.
