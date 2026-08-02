# F-088, correctness, pass 3

**Reviewed**: full final remediated implementation diff from claim base, 9 implementation and HLD files, 855 inserted lines and 20 deleted lines
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Not found

No correctness, contract, panic, OOXML, test-gate, or structure finding was
found. Each automated visual test returns early only after the shared helper
has confirmed that the configured external corpus directory is absent and the
required flag is unset. The final helper split keeps the existing corpus
resolution body unchanged. The same path fails when
`RDOCX_PPTX_CORPUS_REQUIRED=1`. An existing directory with a missing selected
deck still fails. The manual acceptance record remains independent of the
external corpus. The real required corpus, pinned Python comparison, native
evidence, exact-colour gate, latent-placeholder repair, development-only crate
edge, and unchanged hash expectation remain intact.
