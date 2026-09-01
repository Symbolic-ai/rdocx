# F-226, correctness, pass 1

**Reviewed**: working diff, 5 tracked paths plus the design plan, 1,400 diff lines
**Verdict**: 3 defects, 0 smells, 0 nitpicks

## Defects

### D1, unmatched notes placeholders are silently discarded

`crates/rpptx/src/lib.rs:6721`

The notes-slide placeholders are collected and matched against placeholders in
the notes master, but the consumed set is never compared with the collected
overlay set. The later append loop copies only non-placeholder children. A
notes slide with a valid placeholder that has no matching master placeholder
therefore loses that content without either rendering it from its direct
geometry or failing closed. The contract requires notes-slide placeholder
content to survive the hierarchy rather than disappear.

### D2, handout master graphics are painted above every thumbnail

`crates/rpptx/src/lib.rs:6622`

The handout compositor builds thumbnail elements first and then appends every
resolved handout-master element. Presentation master graphics form the
background layer for generated page content. With the current order, a normal
master background rectangle or decorative image that overlaps a thumbnail is
painted last and obscures the source slide. The fixture uses only metadata in
the margins, so it cannot expose this reversal.

### D3, the golden tests do not prove the geometry and sensitivity contract

`crates/rpptx/tests/integration.rs:7095`

The notes golden checks only page count and extracted word membership. The
handout golden at `crates/rpptx/tests/integration.rs:7132` likewise checks only
page count and words. Neither test measures page geometry, thumbnail bounds,
thumbnail order, slide-number positions, clipping, or three-up note rules.
The rejection test at `crates/rpptx/tests/integration.rs:7188` covers one
external theme and three DPI values, but not the promised duplicate,
wrong-type, missing, or malformed relationship cases. There is also no
one-point placement sensitivity check. Large geometry changes or removal of
the note rules can therefore leave all six named tests green despite the test
contract in `.claude/plans/F-226-design.md:95`.

## Smells

None.

## Nitpicks

None.

## Not found

No public API shape, dependency, feature-flag, namespace, schema-order,
unbounded raster-allocation, source-package mutation, deterministic-font,
panic, or ordinary slide-rendering regression was found. The normal verify
gate passed, including the full workspace suite with its LibreOffice boundary,
workspace Clippy, 49 of 49 hash entries, no-default fonts, WASM, rustdoc, and
README inventories.
