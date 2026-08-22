# F-X037, all, pass 1

**Reviewed**: complete 12-file working-tree diff, 1,152 changed lines, with the approved design, cited HLD sections, and progress record
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, ambiguous field matching can attribute repeated literal text to the wrong character range

`crates/rdocx-layout/src/engine.rs:354`
`crates/rdocx-layout/src/engine.rs:391`

`CT_R::text()` includes a field's cached result only for a parsed complex
field, but `projected_content_char_starts` does not use that fact. It treats
every field as optionally contributing its cache and prefers the branch that
skips the current field whenever the remaining tokens can still match.

For example, a public `CT_R::content` containing a parsed complex field with
cached text `a`, literal text `a`, and then a `Field::new` with cached text `a`
has the selected projection `aa`. The matcher can skip the first field, consume
the first projected character as the literal, and consume the last simple
field even though the actual projection does the opposite. The literal glyph
then receives `0..1` instead of `1..2`. Its text still equals the selected
slice because both characters are `a`, so the current equality-based
regression does not expose the false edit location. Offset calculation must
follow the field's actual projection behavior, with a regression covering an
ambiguous repeated-text sequence around both field forms.

## Smells

None.

## Nitpicks

None.

## Not found

No additional findings in contract coverage, scalar range splitting, Word
path allocation, accepted or tracked revision selection, generated-text
attribution, body and nested-table traversal, header and footer traversal,
note traversal, public API compatibility, WASM surface stability, raw layout
result parity, panics, OOXML preservation, or structural rules. The focused
provenance, split, revision, generated-text, caller-font, and compatibility
tests all passed during this review.
