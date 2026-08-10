# S29 sprint review, pass 1

**Reviewed**: `sprint/s29` at `6559983a3d840c60d7e0835c4167e575ad55c042`
against merge base `e3a4b82a91e85e88128f7d090ec887c03b39e3b1`, 34 files,
6,622 changed lines, comprising 6,560 insertions and 62 deletions, crates:
`oxml-drawing`, `oxml-opc`, `oxml-sml`, and `rpptx-chart`
**Verdict**: 1 blocking, 2 should-fix, 0 nice-to-have

## Blocking

### B1, caller-controlled ChartML text can produce invalid XML

`crates/rpptx-chart/src/lib.rs:157`
`crates/rpptx-chart/src/lib.rs:1521`

`StringRef::new` validates only that the formula is nonempty and the point count
fits. It does not validate the formula or cached string values against the XML
1.0 character set. `NumericData::new` has the same gap for its formula and
number format at `crates/rpptx-chart/src/lib.rs:225`. The common writer then
passes every value to `BytesText::new` at
`crates/rpptx-chart/src/lib.rs:1534`, which escapes XML metacharacters but does
not remove or reject forbidden control characters. A direct public-API check
with a cached value containing U+0001 observed that construction and
serialization both returned `Ok`, and the output `c:v` contained byte `0x01`.
That byte is not legal XML 1.0, so a valid API call can produce malformed
ChartML that Office viewers cannot consume. Validate every caller-controlled
formula, format code, and string cache value before construction and again
before serialization, then add a regression that rejects forbidden XML
characters while retaining normal metacharacter escaping.

## Should-fix

### S1, the current rpptx-chart package size is stale in the HLD

`docs/hld/15-build-and-toolchain.md:152`

The publishing section still records the F-118 package as a 12.8 KiB archive.
F-119 then expanded the same source file by more than two thousand lines. A
package inspection at the reviewed head now produces the same five files but a
155.6 KiB unpacked package and a 23.3 KiB compressed archive. HLD files describe
current intent and evidence, so the retained pre-F-119 measurement is false for
the integrated sprint. Replace it with the current measurement or remove the
volatile exact size while retaining the file-count and unpublished-package
claims.

### S2, the implemented ChartML API example does not match the public type

`docs/hld/09-charts-spec.md:88`
`crates/rpptx-chart/src/lib.rs:2389`

The HLD presents the implemented `CT_ChartSpace` with public `ShapeProperties`,
`TextBody`, and `raw_children` fields. The code exposes
`CT_ShapeProperties` and `CT_TextBody`, keeps `raw_children` private, and offers
a read-only accessor at `crates/rpptx-chart/src/lib.rs:2513`. This is not only a
naming abbreviation because it changes what later chart stories can construct
or replace directly. Update the implemented API example to the actual public
surface, or label it as a conceptual model and describe the accessor boundary.

## Nice-to-have

None.

## Milestone gate

The M12 end-of-milestone gate is: "a chart created by rpptx opens in PowerPoint,
its data is editable, and it renders" at
`docs/hld/14-development-backlog.md:915`.

The milestone gate does not yet hold, and S29 does not claim that it does. The
active sprint explicitly establishes the separate workbook and ChartML data
paths before they converge in a later `add_chart` story at
`docs/sprints/CURRENT_SPRINT.md:40`. F-124, which creates a chart and connects
the embedded workbook, remains pending at `docs/sprints/BACKLOG.md:251`, and
F-125, which begins chart rendering, remains pending at
`docs/sprints/BACKLOG.md:252`. There is therefore no `rpptx` chart creation path
whose PowerPoint editing or rendering could satisfy the end-of-milestone gate.

The S29 slice has concrete gate evidence. The F-117 viewer gate at
`crates/oxml-sml/src/lib.rs:903` binds the generated workbook to the recorded
SHA-256, and the completion record names the performed Excel 16.104 and
LibreOffice Calc 26.2.5.2 observations at
`docs/sprints/AS_BUILT.md:3836`. The F-118 corpus gate at
`crates/rpptx-chart/src/lib.rs:3517` parses, writes, reparses, and compares every
chart part in the pinned 50-deck corpus. The F-119 focused gate derives formula,
point count, indexes, and cached literals from one source vector at
`crates/rpptx-chart/src/lib.rs:3117`, while its corpus gate reparses the typed
series projection at `crates/rpptx-chart/src/lib.rs:3343`. An independent
focused rerun passed 7 automated `oxml-sml` tests with the pinned viewer test
ignored, all 14 `rpptx-chart` tests against the 50-deck corpus, and all 19
`oxml-opc` tests. The integrated completion records report the full workspace
gate passed and all 28 deterministic hashes unchanged at
`docs/sprints/AS_BUILT.md:3844`, `docs/sprints/AS_BUILT.md:3885`, and
`docs/sprints/AS_BUILT.md:3927`. B1 still blocks sprint readiness because the
new public writer accepts an input that cannot be represented as valid XML.

## Not found

- Interaction: zero cross-feature findings. The sprint intentionally keeps the
  workbook and ChartML paths separate, and the shared OPC constants are
  consumed only by `oxml-sml`. No F-117 behavior changes the F-118 or F-119
  parse and write path.
- Duplication: zero findings. SpreadsheetML packaging, ChartML core parsing,
  and ChartML series parsing have distinct schema responsibilities. No
  equivalent sprint helper was added under a second name.
- Layering: zero findings. `oxml-sml` depends on `oxml-opc` plus external XML
  and error crates at `crates/oxml-sml/Cargo.toml:12`. `rpptx-chart` has only
  `oxml-core`, `oxml-drawing`, and `quick-xml` as direct normal dependencies at
  `crates/rpptx-chart/Cargo.toml:14`. No changed `oxml-*` manifest adds an edge
  to `rdocx-*` or `rpptx-*`. The transitive `oxml-drawing` to `rdocx-oxml`
  `Theme` adapter is the documented pre-existing exception.
- Harness: zero findings. The reviewed range changes neither the harness nor
  its baseline. All three plans declare an unchanged result, and the integrated
  completion records agree on all 28 hashes.
- Gate evidence: zero unsupported claims. The three S29 story gates name
  executable tests or performed viewer observations. The later M12 gate is
  recorded above as pending rather than inferred from the data-layer tests.
- Dependencies: zero findings. Every new normal dependency has a named current
  consumer in the approved F-117 or F-118 plan. F-119 adds no dependency.
- Public surface scope: zero unrequested API findings. `Workbook`, `Column`, the
  ChartML core types, `StringRef`, `NumericData`, `AxisData`, `Series`, and the
  caller-selected text-body parser are each required by an approved sprint
  plan. B1 concerns validation of that requested surface, not an extra surface.
