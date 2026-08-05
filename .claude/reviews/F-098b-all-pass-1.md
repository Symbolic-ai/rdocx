# F-098b, all aspects, pass 1

**Reviewed**: working diff, 3 files, 312 insertions and 14 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, script-specific typefaces are ignored when Latin is present

`crates/rpptx-render/src/text.rs:34`

Typeface selection always takes `latin_typeface` first. A resolved East Asian,
complex-script, or symbol run that also carries the ordinary Latin slot starts
font resolution from the wrong concrete family. Coverage fallback cannot
recover the requested script-specific family when the Latin font happens to
contain the glyphs. Select the concrete slot from the run text before applying
the ordinary fallback chain, and add regressions for the non-Latin slots.

## Smells

None.

## Nitpicks

None.

## Not found

No additional findings in contract scope, panic safety, OOXML handling, test
gate strength, or structural discipline.
