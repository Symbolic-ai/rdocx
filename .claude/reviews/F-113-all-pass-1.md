# F-113, all, pass 1

**Reviewed**: working tree diff, 8 files, 1,442 changed lines, comprising 1,441 insertions and 1 deletion
**Verdict**: 3 defects, 0 smells, 0 nitpicks

## Defects

### D1, merge leaves model-accepted source cells without the required empty paragraph
`crates/rpptx/src/lib.rs:2192`

Only cells whose `text_body` is already `Some` enter `moved_bodies` and later
receive a body again. `CT_TableCell` accepts and represents a cell with no text
body, so merging a rectangle containing such a source cell succeeds but leaves
that continuation cell with `text_body == None`. This violates the approved
contract that each source cell is left with one empty paragraph and the updated
HLD statement that each source keeps one empty paragraph.

### D2, the graphic-frame constructor does not validate its table argument
`crates/rpptx-oxml/src/graphic_frame.rs:99`

`CT_GraphicFrame::new_table` returns `Ok(Self)` without serializing or otherwise
validating the supplied `CT_Table`. A caller can create or mutate a table with
no rows, pass it to this public constructor successfully, and receive the error
only when the containing frame is later serialized. The approved plan calls
for a validated graphic-frame constructor, and the otherwise unused `Result`
return type promises failure at this boundary.

### D3, the preservation test does not prove byte-identical raw XML
`crates/rpptx/tests/integration.rs:270`

`table_mutation_preserves_unmodelled_xml_and_schema_order` checks that selected
marker strings remain present, but it never compares the captured unsupported
subtrees with their original bytes. Attribute order, whitespace, comments,
processing instructions, or other bytes can change while every assertion still
passes. This does not satisfy the approved preservation test, whose contract is
that unsupported XML remains byte-identical, or the parser and serializer risk
rider requiring a byte-for-byte preservation check.

## Smells

None.

## Nitpicks

None.

## Not found

- Correctness and contract: no additional findings beyond D1 and D2.
- Panics: zero findings. Indexed access exposed by the facade is guarded, and
  the internal indexed access is protected by the lifetime-bound validated
  handle.
- OOXML: no production prefix or schema-order finding. D3 records the missing
  byte-preservation proof.
- Tests: no additional findings beyond D3. The named gate, merge-pattern,
  formatting, negative, round-trip, and pinned differential tests are present.
- Structure: zero findings. The borrowed handles are behavior-bearing and
  required by the approved plan, with no new trait, generic, module, file,
  feature, or dependency.
