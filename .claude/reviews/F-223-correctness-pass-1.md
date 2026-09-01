# F-223 correctness review, pass 1

## Verdict

Ready. I found 0 defects, 0 smells, and 0 nitpicks.

## Findings

### Defects

None.

### Smells

None.

### Nitpicks

None.

## Evidence reviewed

- The six exact modern main-part content types are explicit and unique at
  `crates/oxml-opc/src/content_types.rs:20`.
- Open rejects an absent or unknown main content type before parsing a package
  as a supported class at `crates/rpptx/src/lib.rs:840`.
- `PresentationPackageClass` has one exact content-type mapping in each
  direction at `crates/rpptx/src/lib.rs:408`.
- Ordinary serialization retains the opened package override, while
  `to_bytes_as` changes only a staged copy at `crates/rpptx/src/lib.rs:942` and
  `crates/rpptx/src/lib.rs:981`.
- Class conversion preserves retained signature evidence and records package
  invalidation when the signed content-type table changes at
  `crates/rpptx/src/lib.rs:987`.
- `save_as_show` remains source-compatible and delegates to the ordinary
  slideshow class at `crates/rpptx/src/lib.rs:1319`.
- The round-trip gate covers PPTM, POTX, POTM, PPSX, and PPSM with exact opaque
  binary payload and relationship preservation at
  `crates/rpptx/tests/integration.rs:8353`.
- Conversion, ordinary-save retention, signature invalidation, unknown-class
  rejection, and the legacy show method are independently covered at
  `crates/rpptx/tests/integration.rs:8392`.

## Verification

- Five focused package-class tests passed.
- The complete `rpptx` suite passed with 33 unit tests, 192 active integration
  tests, and the documented external-only ignores.
- `oxml-opc` passed 25 tests and its doctests.
- Scoped all-target, all-feature Clippy passed with warnings denied.
- The hash harness passed 49 of 49 unchanged.
- Formatting and diff hygiene passed.
