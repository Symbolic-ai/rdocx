# F-174, all, pass 1

**Reviewed**: Working diff against claim base `6bede077c03e93aa55a2555ad5adc3fe456d5e30`, including 12 tracked files with 482 insertions and 38 deletions, two untracked text files with 293 lines, and the approved untracked 3,024-byte ICC profile across 15 files.
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, archival link annotations omit the mandatory print flag

`crates/oxml-pdf/src/writer.rs:888`

The preflight accepts ordinary HTTP, HTTPS, mailto, and tel links, but the
shared annotation writer emits only subtype, rectangle, border, and URI action.
It does not emit the annotation `F` entry with the Print flag set. PDF/A-2 and
PDF/A-3 require that entry and flag for annotations. A layout containing an
accepted link therefore returns bytes labelled as PDF/A that fail the declared
profile. The veraPDF fixture contains no link, so the differential gate does
not exercise this path.

### D2, archival preflight accepts a paint that the writer drops

`crates/oxml-pdf/src/conformance.rs:168`

`Paint::Tile` passes preflight unconditionally, while the writer resolves that
same public paint variant to `None` and emits no fill or stroke. An archival
request with tile paint therefore returns success after silently removing the
requested content. This violates the plan's requirement to reject unsupported
state before writing, and the regression matrix contains no tile-paint case.

## Smells

None.

## Nitpicks

None.

## Not found

No additional findings in correctness, contract, panics, PDF and PDF/A
metadata, tagged structure retention, OOXML behavior, tests, public API,
package asset provenance, WASM and binding scope, structure, or determinism.
