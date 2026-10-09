# F-278, microscope, pass 3

**Reviewed**: Frozen working implementation on work/f-278-codex against the
approved design. Eleven tracked files, 1134 insertions and 37 deletions, plus
prior review records. Follow-up review covers recursive D3 remediation,
capability classification and refreshed archive measurements.
**Verdict**: 0 defects, 0 smells, 0 nitpicks.

## Defects

None found.

## Smells

None found.

## Nitpicks

None found.

## Not found

Correctness: D3 now compares source identities recursively through arguments
and switches, with matching lengths. Equal-text replacements at child and
grandchild depths no longer allow an enclosing parsed source shortcut. D1,
D2 and D4 remain resolved as recorded in pass 2.
Contract: explicit simple and complex construction, typed operands, ordered
caches, checked attachment, lock toggles and legacy behavior match the plan.
Panics: no new untrusted-input panic path found in checked constructors,
namespace validation or lock rewriting.
OOXML: schema field sequence is preserved. Expanded namespace names distinguish
real Word delimiters from producer lookalikes. Unknown XML and retained source
bytes remain covered by opposing preservation and rejection regressions.
Tests: depth-sensitive replacement now covers both argument and switch operand
locations. Break attachment covers both representations and both non-line
controls. The declared round-trip gate uses the new checked API and cannot
compile against the original implementation.
Structure: no new trait, generic parameter, module, source file or forwarding
wrapper. Concrete existing CT_R values own cached content.
Documentation: exactly four planned HLD owners changed. DOCX-045 remains partial
for broader rendering and execution while describing completed native checked
construction and cache round trips. Archive constants match refreshed records.

## Review disposition

Pass 1 D1 through D4 and the remaining pass 2 D3 depth case are resolved.
Required scoped verification and package riders must finish before preparation.
