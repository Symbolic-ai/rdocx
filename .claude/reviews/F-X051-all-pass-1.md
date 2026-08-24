# F-X051, all, pass 1

**Reviewed**: uncommitted working diff, 4 files, 520 insertions and 14 deletions, 534 changed lines
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, a label alias can select a bundled face instead of the caller face

`crates/oxml-layout/src/font.rs:407`

The label-derived table retains only the embedded family string, not the face
that was loaded from the labelled bytes. In the bundled-fallback manager, the
bundled database is populated before caller fonts. If caller bytes embed a
family that is also bundled, such as a modified Caladea labelled `Document
Serif`, resolving the alias queries all Caladea faces and selects the earlier
bundled face when style and weight tie. The document-facing label then uses
bundled bytes instead of the supplied caller bytes, contrary to caller-face
priority and the test gate's correct-byte requirement. The regression fixture
at `crates/rdocx/tests/regression_test.rs:5652` supplies the exact bundled
Caladea bytes, so its byte comparison cannot distinguish the two faces.

### D2, case-only label differences are discarded even though lookup is case-sensitive

`crates/oxml-layout/src/font.rs:405`

The loader suppresses an alias with `eq_ignore_ascii_case`, but family queries
at `crates/oxml-layout/src/font.rs:766` use the caller spelling directly. A
caller that labels embedded `Caladea` bytes as `caladea` therefore stores no
alias. The exact query misses the case-sensitive database name and resolution
falls through to the mapped or generic chain, selecting the wrong face. A
case-only difference must either retain a usable alias or be normalized in the
actual family lookup.

## Smells

None.

## Nitpicks

None.

## Not found

- Panics: no new production panic, unchecked indexing, slicing, or arithmetic
  hazard was found.
- OOXML: this diff does not parse or serialize XML, and no schema-order,
  namespace, whitespace, or unmodelled-subtree issue was found.
- Structure: no unjustified trait, generic, wrapper, feature flag, crate,
  module, or file was introduced.
- Reusable-engine transfer: no defect beyond the font-resolution findings was
  found in exact alias identity, rejection preservation, or cache-context
  invalidation.
- Facade coverage: the approved default, option-taking, and checked-transfer
  alias-aware entry points are present.
