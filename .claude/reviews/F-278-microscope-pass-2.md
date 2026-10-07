# F-278, microscope, pass 2

**Reviewed**: Frozen working diff on work/f-278-codex. Eleven tracked files,
1103 insertions and 33 deletions, plus the pass 1 review. Approved contract,
all four planned HLD owners, archive measurements and added regressions read.
**Verdict**: 1 defect, 0 smells, 0 nitpicks.

## Defects

### D3, nested source replacement remains invisible beyond one level

`crates/rdocx-oxml/src/text.rs:9516`

The new unchanged guard at `crates/rdocx-oxml/src/text.rs:822` correctly detects
a replaced direct operand. Its source identity comparison still checks only
the immediate parsed source ID, without descending into that field's
instruction. In a parsed outer field containing a parsed middle field,
replace the middle field's nested operand with a checked field having the same
instruction and display but different cached run formatting. The middle
field's own source ID remains unchanged. PartialEq ignores the replacement's
cache properties, so the outer field still passes both guards and emits its
original raw XML. The replacement is lost. Compare source identity recursively
through nested arguments and switches before any enclosing unchanged-source
shortcut. Extend the regression beyond a direct child and cover both operand
locations. This is the remaining depth case of pass 1 D3.

## Smells

None found.

## Nitpicks

None found.

## Not found

Correctness and contract: pass 1 D1 and D2 are resolved. Unknown typed operands
retain switch positions, known tested flags retain positional boundaries, and
typed page and column breaks pass attachment without accepting invalid display
replacement strings. D3 is resolved only for direct replacement.
OOXML: pass 1 D4 is resolved by expanded-name checking in retained namespace
scope. Foreign lookalikes and aliased real Word controls have opposing tests.
Lock and dirty attribute edits preserve the checked three-state contract.
Panics: no new panic path found on checked authoring inputs.
Structure: no new trait, generic, module, source file or forwarding wrapper.
Tests: no additional gap beyond recursive D3 identified in the declared gate.
Documentation: planned HLD files reflect the implemented native surface and
pre-1.0 projection impact. Archive measurements agree with recorded constants.
