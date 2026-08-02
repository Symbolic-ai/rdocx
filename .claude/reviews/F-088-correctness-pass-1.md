# F-088, correctness, pass 1

**Reviewed**: working diff, 5 tracked files, 675 inserted lines and 7 deleted lines
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, normalized evidence omits the ordered shape kind

`crates/rpptx/tests/integration.rs:846`

The approved plan requires every normalized shape record to identify its kind.
The evidence line records bounds, transforms, paints, text, and unsupported
state, but no geometry or content kind. Two different shape categories with the
same bounds and text therefore produce indistinguishable evidence, so the dump
does not freeze the complete renderer-facing ordering it claims to review.

### D2, prompt suppression misses a selected deck's subtitle prompt

`crates/rpptx/tests/integration.rs:544`

The selected `bug58144-headers-footers-2007.pptx` layout contains `Click to edit
Master subtitle style`, but the regression checks only title, generic text, and
add prompts. Reintroducing that exact subtitle placeholder would leave the
named prompt-suppression test green. Check the generic `Click to edit` and
`Click to add` prefixes, or include every selected prompt explicitly.

## Smells

None.

## Nitpicks

None.

## Not found

No other correctness, contract, panic, OOXML, test-gate, or structure finding
was found. The source-sensitive latent policy matches the remediated native
PowerPoint evidence, the type fallback is confined to latent placeholders, the
Python oracle is pinned, and the new crate edge remains development-only.
