# F-124, all, pass 1

**Reviewed**: uncommitted working diff, 9 files and 821 changed lines
**Verdict**: 5 defects, 0 smells, 0 nitpicks

## Defects

### D1, the manual PowerPoint gate has no recorded result

`crates/rpptx/tests/integration.rs:393`

The ignored test creates the SHA-bound candidate and verifies its package can
be reopened, but it does not record the pinned PowerPoint version or build, a
clean open without repair, or the Edit Data observation. The approved plan
requires all of that evidence at `.claude/plans/F-124-design.md:136`, and its
HLD impact requires the evidence in `docs/hld/09-charts-spec.md`. The current
HLD update contains no F-124 acceptance record. A generated candidate is not
the story's manual test gate.

### D2, embedded workbooks using the canonical stem are excluded from numbering

`crates/rpptx/src/lib.rs:1597`

The allocator scans only `Microsoft_Excel_WorksheetN.xlsx`, while the cited OPC
contract names `/ppt/embeddings/WorkbookN.xlsx` at
`docs/hld/04-opc-and-packaging.md:121` and the chart topology repeats that name
at `docs/hld/09-charts-spec.md:21`. A package containing `Workbook7.xlsx` and
`chart2.xml` therefore chooses 3 rather than 8, contrary to the approved test
requirement that sparse chart and embedding suffixes produce maximum suffix
plus one. The implementation and the current HLD also disagree about the
canonical workbook stem.

### D3, negative chart extents serialize as invalid OOXML

`crates/rpptx/src/lib.rs:1351`

`add_chart` sends caller-provided width and height directly into a
`CT_PositiveSize2D`. Neither validation nor frame serialization rejects a
negative value, so `Emu(-1)` becomes a negative `a:ext/@cx` or `a:ext/@cy`
inside `p:xfrm`. That violates the positive-size schema contract and can leave
the caller with a deck that PowerPoint repairs instead of returning an error
without mutation.

### D4, a high relationship id can panic in `add_chart`

`crates/rpptx/src/lib.rs:1338`

The new path calls `Relationships::add`, whose counter increment at
`crates/oxml-opc/src/relationship.rs:223` is unchecked. A parsed slide
relationship scope whose greatest numeric id is `rId4294967294` sets the next
counter to `u32::MAX`. `add_chart` formats that id and then overflows while
incrementing, which panics in the normal debug test profile rather than
returning `Result::Err`. In an overflow-disabled build the counter wraps to
zero and the following relationship allocation can produce `rId0`.

### D5, the cache and workbook consistency test checks presence, not agreement

`crates/rpptx/tests/integration.rs:218`

The test searches the complete chart and worksheet XML for an unordered set of
formula and value substrings. It still passes if the two series swap formula
ranges, if a series name points at the other header, or if values move between
columns, because every searched token remains present somewhere. The approved
test contract at `.claude/plans/F-124-design.md:113` requires formulae, caches,
and worksheet cells to agree for every series. The assertions need to compare
the parsed series mappings and exact worksheet coordinates.

## Smells

None.

## Nitpicks

None.

## Not found

- Correctness: no other wrong plot-family construction, cache generation, or
  part graph logic found.
- Contract: the manual candidate is constructed through the public
  `Presentation::add_chart` API.
- Panics: no other new untrusted-input panic path found.
- OOXML: relationship targets resolve correctly, both content-type overrides
  are present, and the graphic frame and ChartML children use fixed prefixes
  in schema order.
- Tests: no other gate weakness found.
- Structure: no unjustified trait, generic, wrapper, feature, crate, module, or
  file was introduced.
- Atomicity: all fallible chart, workbook, and frame serialization occurs
  against the cloned package before the live package or slide tree is changed.
- Preservation: the chart graphic-frame constructor uses the existing raw
  payload boundary and the existing unrelated raw-subtree coverage remains in
  place.
