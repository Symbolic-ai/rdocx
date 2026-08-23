# F-X051, all, pass 4

**Reviewed**: uncommitted working diff, 4 files, 938 insertions and 40 deletions, 978 changed lines
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, internal alias-bound machinery expands the public API

`crates/oxml-layout/src/font.rs:270`

`CALLER_ALIAS_MAX_ENTRIES`, `CALLER_ALIAS_MAX_RETAINED_BYTES`, and
`bounded_caller_aliases` are public symbols in the published `oxml-layout`
crate. The approved public surface adds `FontManager::set_caller_aliases` and
the three facade methods. It does not add a generic alias canonicalization
helper or expose its implementation ceilings as caller contracts.
`#[doc(hidden)]` at `crates/oxml-layout/src/font.rs:282` removes the helper from
generated documentation but does not remove it from the public API. Keep the
bound implementation private and route cross-crate plumbing without exporting
these extra symbols.

## Smells

None.

## Nitpicks

None.

## Not found

- Pass 3 D1: explicit aliases now use one deterministic prefix bounded to 256
  entries and 64 KiB of retained ordered and lookup payload. Oversized suffixes
  cannot create a different reusable identity, and the focused bound tests pass.
- Pass 2 D1: constructor caller-label and embedded-family metadata remain
  restored with `base_db` after additional-font replacement.
- Pass 2 D2: exact and alias caller-face selection still delegates weight,
  stretch, and style choice to `fontdb::Database::query`.
- Pass 1 D1 and D2: label aliases retain exact caller face ids, including
  same-family bundled collisions and case-only label differences.
- Correctness: exact embedded families precede explicit and label-derived
  aliases, followed by mapped and generic fallbacks. No additional resolution
  or caller-byte selection defect was found.
- Cache identity: one bounded ordered alias identity reaches the facade,
  engine, retained-work context, and font manager. Changed aliases invalidate
  dependent resolution and reusable block state. Rejected checked transfers
  preserve both engines.
- Panics: no new production panic, unchecked indexing, slicing, or arithmetic
  hazard was found.
- Tests: all 83 `oxml-layout` unit tests, 3 doctests, all 161 `rdocx-layout`
  unit tests, its doctest, and the focused public regression pass. The tests
  cover the prior alias-bound, face-selection, constructor, warm and cold, and
  transfer failures.
- OOXML: the diff does not parse or serialize XML. No schema-order, namespace,
  whitespace, or unmodelled-subtree issue was found.
- Structure beyond D1: no unjustified trait, generic parameter, dynamic
  dispatch, feature flag, crate, module, or file was introduced.
