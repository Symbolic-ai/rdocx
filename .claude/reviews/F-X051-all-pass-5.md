# F-X051, all, pass 5

**Reviewed**: uncommitted working diff, 4 files, 976 insertions and 40 deletions, 1,016 changed lines
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Not found

- Prior defects: the exact caller face survives bundled-family collisions,
  case-only labels resolve, constructor metadata survives additional-font
  replacement, and caller face selection follows `fontdb` CSS rules.
- Private bounded plumbing: both alias helpers and both ceilings are private.
  The `oxml-layout` and `rdocx-layout` implementations use the same
  deterministic prefix and byte accounting, and boundary tests compare the
  exact retained identities.
- Correctness: exact embedded families precede explicit and label-derived
  aliases, followed by mapped and generic fallbacks. Alias changes invalidate
  resolution-dependent state without discarding stable loaded faces or shaping
  entries.
- Public API and contract: the concrete `FontManager` setter and the approved
  default, option-taking, and checked-transfer facade paths are present.
  Existing strict and bundled-fallback method signatures remain unchanged.
- Cache identity: the same bounded ordered aliases reach font lookup, retained
  block context, restart eligibility, and checked transfer. Equal aliases are
  reusable, changed aliases miss stale work, and rejected transfer preserves
  both engines.
- Panics: no new production panic, unchecked indexing, slicing, or arithmetic
  hazard was found.
- Tests: all 83 `oxml-layout` unit tests and 3 doctests pass. All 161
  `rdocx-layout` unit tests and its doctest pass. The focused alias identity,
  changed-context, warm and cold, and public regression tests pass.
- OOXML: the diff does not parse or serialize XML. No schema-order, namespace,
  whitespace, or unmodelled-subtree issue was found.
- Structure: no unjustified trait, generic parameter, dynamic dispatch,
  wrapper, feature flag, crate, module, or file was introduced. Formatting and
  diff hygiene pass.

No further microscope pass is required.
