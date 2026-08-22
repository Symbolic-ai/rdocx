# F-X037, all, pass 2

**Reviewed**: complete 13-file working-tree diff, 1,144 changed lines, including the pass 1 field-projection remediation
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Not found

Pass 1 D1 is resolved. `Field::projected_text` now owns the parsed-complex
projection decision used by both `CT_R::text` and the scalar offset walk, so
identical cached and literal text cannot shift an attributed range. The new
regression covers ambiguous repeated literals adjacent to parsed-complex and
simple field forms.

No findings remain in correctness, contract conformance, panic paths, OOXML
preservation, test sensitivity, or structural rules. The full diff was also
rechecked for both scalar split stages, deterministic Word path allocation,
accepted and tracked revision projections, generated-text attribution, every
supported Word story, public and WASM surface compatibility, and raw layout
result parity. The complete `rdocx-layout` and `oxml-layout` suites passed,
including doctests. `git diff --check`, prose validation, and generated-skill
sync also passed.
