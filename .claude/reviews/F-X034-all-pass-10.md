# F-X034, all, pass 10

**Reviewed**: uncommitted `work/f-x034-codex` diff, 8 files and 1,402 added or deleted lines
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, escaped punctuation alone is promoted to meaningful content
`scripts/sprint_workflow.py:221`

Every CommonMark escape pair is replaced with U+25CF black circle, whose `So`
category satisfies the final meaningful-character predicate. Required sections
containing only `\*`, `\\`, `\[`, or `\!` therefore pass, although they render
only one punctuation character and contain none of the text or pictographs the
validator otherwise requires. Direct probes reproduced all four cases. The
new regression matrix explicitly accepts the escaped-asterisk trigger, so it
now protects the bypass rather than the ceremony's semantic-empty invariant.

## Smells

None.

## Nitpicks

None.

## Not found

The pass 9 escaped-element defect is remediated. CommonMark escape parity is
preserved, escaped element names and custom elements remain visible, and
escaped comments, processing instructions, declarations, CDATA, entities,
and brackets no longer enter raw HTML classification. Unescaped raw HTML stays
inert, and the renderer preserves the original reviewed bytes.

No additional defects were found in links, references, autolinks, images,
inline or fenced code, block markers, Unicode visibility, raw HTML, heading
state, tag validation, required section order, placeholders, family
separation, check and render immutability, pre-publication ordering, exact
release-body comparison, final approval, contributor evidence, or
post-publication verification. Generated adapters remain synchronized.

The five focused workflow tests passed. The generated-skill drift check,
prose check, and `git diff --check` also passed.
