# F-124, all, pass 2

**Reviewed**: uncommitted working diff, 10 files and 984 changed lines, plus
pass 1 remediation against candidate SHA-256
`e6e9f7eef1c774d0414c5d0c3f1202da1a28635b5d089e15455b7adc3f66cb00`
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, the manual PowerPoint gate has not been performed

`crates/rpptx/tests/integration.rs:435`

The ignored test generates the candidate and binds it to the expected SHA, but
there is still no recorded human observation from pinned PowerPoint that the
deck opened without repair and that Edit Data exposed the authored categories
and values. The approved gate requires both observations at
`.claude/plans/F-124-design.md:111`. The candidate file on disk matches the
expected SHA and its automated package checks pass, but those facts do not
substitute for the required manual application gate.

## Smells

None.

## Nitpicks

None.

## Remediation verified

- D2 is resolved. Chart and workbook numbered families are allocated
  independently from their greatest occupied positive suffix at
  `crates/rpptx/src/lib.rs:1334`, and the sparse fixture asserts `chart3.xml`
  alongside `Workbook8.xlsx` at `crates/rpptx/tests/integration.rs:163`.
- D3 is resolved. Nonpositive chart extents are rejected before package
  staging at `crates/rpptx/src/lib.rs:1330`, with zero and negative coverage at
  `crates/rpptx/tests/integration.rs:346`.
- D4 is resolved. Relationship allocation uses checked rollover and collision
  scanning at `crates/oxml-opc/src/relationship.rs:264`, with high and maximum
  numeric identifier coverage at `crates/oxml-opc/src/relationship.rs:408`.
- D5 is resolved. The cache test compares typed formula-to-series mappings and
  exact worksheet coordinates at `crates/rpptx/tests/integration.rs:214`.

## Not found

- Correctness: no other wrong validation, plot-family construction, cache,
  workbook, numbering, relationship, or package graph logic found.
- Contract: apart from the pending manual gate, the implementation matches the
  approved API, validation, atomic mutation, part naming, and HLD impact.
- Panics: the new relationship allocator does not overflow or collide at the
  `u32` boundary. Remaining `expect` calls consume invariants established by
  validation and successful workbook construction.
- OOXML: fixed prefixes and schema child order are retained for the graphic
  frame, ChartML, relationship scopes, and content-type overrides.
- Tests: no weakness remains in the automated graph, numbering, cache,
  rollback, supported-family, or round-trip coverage.
- Structure: no unjustified trait, generic, wrapper, feature, crate, module, or
  file was introduced.
- Atomicity: all fallible authoring and serialization operates on staged
  values before the live package or slide tree changes.
- Preservation: the new graphic frame uses the existing chart raw-payload
  boundary and does not broaden parsing or discard unrelated XML.

## Checks run

- `cargo test -p oxml-opc high_numeric_relationship_ids_roll_over_without_collision`
- `cargo test -p oxml-opc parsed_u32_max_relationship_id_rolls_to_first_free_positive_id`
- `cargo test -p rpptx-oxml --test integration authored_chart_graphic_frame_round_trips`
- `cargo test -p rpptx --test integration add_chart_`

All focused checks passed.
