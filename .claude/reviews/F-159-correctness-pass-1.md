# F-159, correctness, pass 1

**Reviewed**: working-tree diff against
`5a93dad0e8e4a43fa78525d54a3172e546ed8455`, 13 files and 854 changed lines
**Verdict**: 3 defects, 0 smells, 0 nitpicks

## Defects

### D1, chart themes are read from a conventional path instead of their relationship target

`crates/rdocx/src/document.rs:2659`

The package assembly always reads `/word/theme/theme1.xml`. A valid document
may point its document-level theme relationship at another internal part. Its
chart then renders with the Office fallback rather than the document's
effective theme. Resolve the internal `THEME` relationship against the
document part and add a non-conventional-target regression.

### D2, the golden artifact paths do not exist on the Linux verification runner

`crates/rdocx/src/document.rs:4107`

The gate writes both artifacts under `/private/tmp`, which is a macOS path and
is absent on the Ubuntu CI runner. The test fails before it reaches the pinned
raster comparison. Use `std::env::temp_dir()` with collision-safe filenames
while retaining the two SHA assertions.

### D3, the anchored regression does not prove its configured wrap distances

`crates/rdocx/src/document.rs:3971`

The test sets 12 point left and right distances, but only checks z-order and
the chart transform. An implementation that ignores both distances still
passes. Assert the foreground text starts outside the chart frame plus its
configured clearance so the planned wrapping contract is executable.

## Smells

None.

## Nitpicks

None.

## Not found

No additional correctness, contract, panic, OOXML preservation, test-gate, or
structural findings.
