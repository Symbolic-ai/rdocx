# F-224, correctness, pass 2

**Reviewed**: remediated working tree across 5 files, 2,172 inserted lines and 2 deleted lines
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, the perturbation test does not exercise the browser gate comparator
`crates/rpptx/src/html.rs:1906`

The approved oracle rider requires proof that the actual gate detects
structural, geometry, text, and calibrated pixel perturbations. This test uses
a separate three-field predicate and an arbitrary byte difference instead of
the full-image SSIM helper and 0.95 threshold used by the Chrome gate. A broken
or inverted SSIM gate can therefore pass the perturbation test.

## Smells

None.

## Nitpicks

None.

## Not found

No additional findings in correctness, contract, panics, OOXML, tests, or
structure. The three pass 1 defects are remediated. The apparent duplicate
assignment reported as the pass 1 nitpick was an overlapping inspection output
boundary and is not present in the source.
