# F-155, all, pass 1

**Reviewed**: Working diff from base `ad73c45`, 7 files and 600 changed lines, comprising 594 insertions and 6 deletions. The count includes the untracked 439-line `crates/rdocx-oxml/src/settings.rs` file.
**Verdict**: 0 defects, 1 smell, 0 nitpicks

## Defects

None.

## Smells

### S1, private writer generic has only one concrete instantiation
`crates/rdocx-oxml/src/settings.rs:322`

`write_document_protection<W: Write>` introduces a generic parameter, but the
complete diff instantiates it only through the single `Writer<Vec<u8>>` created
by `CT_Settings::to_xml`. This violates the structural rule that a new generic
parameter needs two existing instantiations today. The helper can accept the
concrete writer type used by this read-only settings model.

## Nitpicks

None.

## Not found

Correctness, contract, panics, OOXML, and tests produced no findings. Structure
produced S1 above.
