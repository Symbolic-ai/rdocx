# F-151, all, pass 5

**Reviewed**: complete remediated working-tree diff against `HEAD`, 12 files, 1,180 changed lines, with 1,132 additions and 48 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, tracked note revisions lose their underline and strike
`crates/rdocx-layout/src/engine.rs:729`
`crates/rdocx-layout/src/paginator.rs:1034`

Paragraph layout correctly places the forced tracked underline or strike on
each note text segment. The dedicated note renderer then lowers that segment
directly to a glyph run without emitting either decoration. An insertion or
move destination in a footnote or endnote therefore gets the remediated change
bar but not its required single underline. A deletion or move source gets the
bar but no strike.

## Smells

None.

## Nitpicks

None.

## Not found

Pass-4 D1, D2, and D3 are resolved. Correctness and contract review produced
only D1 above. Panic safety, OOXML preservation and schema ordering, test
structure, and structural-rule compliance produced no additional findings.
