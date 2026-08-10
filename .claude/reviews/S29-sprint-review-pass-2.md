# S29 sprint review, pass 2

**Reviewed**: `sprint/s29` at `63bf438fa7b81c92a17ded08bfc7de9a8c18a1c8`
against merge base `e3a4b82a91e85e88128f7d090ec887c03b39e3b1`, 35 files,
6,805 changed lines, comprising 6,740 insertions and 65 deletions, crates:
`oxml-drawing`, `oxml-opc`, `oxml-sml`, and `rpptx-chart`
**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have

## Blocking

None.

## Should-fix

None.

## Nice-to-have

None.

## Pass 1 follow-up

Pass 1 B1 is resolved. `StringRef` validates its formula and every cached text
value during construction, parsing, and serialization at
`crates/rpptx-chart/src/lib.rs:157` and
`crates/rpptx-chart/src/lib.rs:190`. `NumericData` applies the same shared
formula and format-code validation at `crates/rpptx-chart/src/lib.rs:230`. The
validator implements the XML 1.0 character ranges at
`crates/rpptx-chart/src/lib.rs:1593`. The regression at
`crates/rpptx-chart/src/lib.rs:3273` rejects U+0001 through constructors and
post-construction public-field edits, then proves that ordinary ampersands and
angle brackets still serialize as escaped text. All 15 `rpptx-chart` tests pass
against the pinned 50-deck corpus, and the focused strict Clippy and formatting
checks are clean.

Pass 1 S1 is resolved. HLD 15 no longer retains the stale F-118 archive size.
It records the durable five-file package boundary and the crates.io size limit
at `docs/hld/15-build-and-toolchain.md:152`. Independent package inspection at
the reviewed head produced those exact five files in a 23.7 KiB compressed
archive, safely below 10 MiB.

Pass 1 S2 is resolved. The implemented API example now names
`CT_ShapeProperties` and `CT_TextBody`, lists only the actual public fields, and
describes preservation state as private behind `raw_children()` at
`docs/hld/09-charts-spec.md:88`. This matches `CT_ChartSpace` and its accessor at
`crates/rpptx-chart/src/lib.rs:2442` and
`crates/rpptx-chart/src/lib.rs:2566`.

## Milestone gate

The M12 end-of-milestone gate is: "a chart created by rpptx opens in PowerPoint,
its data is editable, and it renders" at
`docs/hld/14-development-backlog.md:915`.

The milestone gate does not yet hold, and S29 does not claim it. The sprint
contract keeps the workbook and ChartML data paths separate until the later
`add_chart` story at `docs/sprints/CURRENT_SPRINT.md:44`. F-124 chart creation
and workbook connection remains pending at `docs/sprints/BACKLOG.md:251`, and
F-125 chart rendering remains pending at `docs/sprints/BACKLOG.md:252`.

The completed S29 slice has concrete evidence. The F-117 viewer gate at
`crates/oxml-sml/src/lib.rs:904` binds its workbook to the SHA recorded with the
performed Excel and LibreOffice observations at
`docs/sprints/AS_BUILT.md:3836`. The F-118 corpus gate at
`crates/rpptx-chart/src/lib.rs:3570` structurally round-trips every chart part in
the pinned corpus. The F-119 focused gate at
`crates/rpptx-chart/src/lib.rs:3138` derives cache metadata and literals from one
source vector, and its corpus gate at `crates/rpptx-chart/src/lib.rs:3397`
reparses every supported series projection. Current focused reruns passed 7
automated `oxml-sml` tests with the native viewer gate ignored, all 15
`rpptx-chart` tests, all 19 `oxml-opc` tests, strict `rpptx-chart` Clippy,
workspace formatting, prose, adapter synchronization, and package inspection.
The integrated full verification remains recorded with all 28 deterministic
hashes unchanged at `docs/sprints/AS_BUILT.md:3844`,
`docs/sprints/AS_BUILT.md:3885`, and `docs/sprints/AS_BUILT.md:3927`.

## Not found

- Interaction: zero findings. F-117 remains isolated behind the later
  authoring story, while F-119 attaches to F-118's preserved plot-area seam.
  The XML validation remediation changes only caller-controlled typed ChartML
  text and does not alter preserved payloads or the workbook path.
- Duplication: zero findings. SpreadsheetML packaging, ChartML core parsing,
  ChartML series parsing, and the shared ChartML text validator have distinct
  concrete responsibilities. No equivalent helper was added under another
  name.
- Layering: zero findings. `oxml-sml` depends on `oxml-opc` plus external
  libraries. `rpptx-chart` directly depends only on `oxml-core`,
  `oxml-drawing`, and `quick-xml`. No changed `oxml-*` manifest adds an edge to
  `rdocx-*` or `rpptx-*`. The existing `oxml-drawing` to `rdocx-oxml` `Theme`
  adapter remains the documented exception.
- Harness: zero findings. The sprint changes neither the hash harness nor its
  baseline. All three design plans and completion records declare the same
  unchanged 28-entry result. The pass 1 remediation is confined to the
  unpublished ChartML crate and HLD text.
- Gate evidence: zero unsupported claims. Every S29 story gate names a current
  executable test or a performed viewer observation. The later M12 milestone
  gate remains explicitly pending.
- Docs: zero findings. HLD 09 now matches the implemented workbook, core,
  series, preservation, and public-access boundaries. HLD 15 matches the
  current unpublished dependency and package boundary.
- Dependencies: zero findings. Each new normal dependency has a named current
  consumer in an approved sprint plan. F-119 and the review remediation add no
  dependency.
- Public surface: zero findings. Every public workbook, ChartML, series, cache,
  and shared text-body addition is required by an approved plan. The remediation
  narrows invalid input without adding an unrequested API.
