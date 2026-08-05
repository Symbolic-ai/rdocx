# F-101, correctness, pass 1

**Reviewed**: working diff, 6 files and 277 changed lines
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, fallback direction regressions do not exercise renderer mapping

`crates/rpptx-layout/src/context.rs:2206`

The regression verifies that Mongolian and WordArt text survives resolution and
records each diagnostic, but it stops before the renderer. Reversing the
fallback arms in `oriented_content_box` would keep this test green even though
the documented quarter-turn mapping was wrong. Exercise each fallback direction
through the renderer helper and assert the expected positive or negative
quarter turn.

## Smells

None.

## Nitpicks

None.

## Not found

No additional correctness, contract, panic, OOXML, test, or structure findings.
