# F-118, ChartML core types

**Status**: completed
**Sprint**: S29
**Size**: L
**Depends on**: F-063

## Problem

The workspace currently treats a chart-bearing graphic frame as opaque bytes
at `crates/rpptx-oxml/src/graphic_frame.rs:30`. There is no `rpptx-chart` crate
in the workspace member list at `Cargo.toml:3`, even though the architecture
assigns ChartML modeling and rendering to that crate. Later plot, authoring,
and rendering stories cannot build on a schema-aware chart root.

The core boundary must type `c:chartSpace`, `c:chart`, `c:plotArea`, title,
legend, chart flags, and DrawingML shape and text properties without claiming
the plot, series, or axis work owned by later F-IDs. Unsupported ChartML must
remain byte-preserved in its original schema slot.

## Spec reference

- `docs/hld/03-architecture.md`, "Three families, one workspace" and "The
  dependency rule".
- `docs/hld/04-opc-and-packaging.md`, XML preservation and prefix contract.
- `docs/hld/05-drawingml-model.md`, "Text" and "Preservation".
- `docs/hld/06-presentationml-model.md`, "The shape tree".
- `docs/hld/09-charts-spec.md`, "The ChartML model".
- `docs/hld/12-testing-strategy.md`, "The deck corpus".
- `docs/hld/13-risks-and-open-questions.md`, "R5, schema child ordering".
- `docs/hld/14-development-backlog.md`, "F-118, ChartML core types".
- `docs/hld/15-build-and-toolchain.md`, "Publishing".

## Approach

Create `crates/rpptx-chart` as an unpublished workspace crate at version 0.0.0.
The story authorizes the minimal `Cargo.toml` and `src/lib.rs`. Register it as a
workspace member and dependency after the existing PresentationML crates. Its
normal dependencies are `oxml-core`, `oxml-drawing`, and `quick-xml`. It does
not depend on `rpptx`, `rdocx`, or their facades.

Implement the core model in the single crate root:

```rust
pub struct CT_ChartSpace {
    pub chart: CT_Chart,
    pub sp_pr: Option<CT_ShapeProperties>,
    pub tx_pr: Option<CT_TextBody>,
}

pub struct CT_Chart {
    pub title: Option<CT_Title>,
    pub auto_title_deleted: bool,
    pub plot_area: CT_PlotArea,
    pub legend: Option<CT_Legend>,
    pub plot_vis_only: bool,
    pub disp_blanks_as: DispBlanksAs,
}

pub struct CT_Title { /* preserved attributes and ordered children */ }
pub struct CT_PlotArea { /* preserved attributes and ordered children */ }
pub struct CT_Legend { /* preserved attributes and ordered children */ }
```

`CT_ChartSpace::from_xml` and `to_xml` read any bound ChartML prefix and write
the fixed `c:`, `a:`, and `r:` prefixes. The root types reject missing required
`c:chart` and `c:plotArea`, duplicate modeled children, malformed booleans, and
unknown `dispBlanksAs` values. Optional absent booleans use their ECMA defaults.
Title, plot area, and legend are behavior-bearing shells that preserve their
complete current children in ordered raw slots. F-119 types series values, and
F-120 through F-122 type axes and plot variants.

Reuse `CT_ShapeProperties` for `c:spPr`. Extend the existing concrete
`CT_TextBody` parser with a caller-selected root local name so the two current
consumers, `a:txBody` and `c:txPr`, share the body, list-style, paragraph, and
run implementation. Its existing `write_xml_as` already writes a caller-owned
root tag. This is the existing second use that justifies the shared method.

Corpus tests enumerate every `/ppt/charts/chartN.xml` part in the pinned
50-deck set. They parse, serialize, reparse, compare modeled values, and assert
that every unmodeled subtree stays byte-identical at its boundary. A small
inline chart remains the negative and prefix-alias fixture so the crate tests
do not become vacuous when the external corpus is absent.

## Rejected alternatives

- Type charts inside `rpptx-oxml`. The architecture assigns the ChartML model
  and later renderer to `rpptx-chart`.
- Make `rpptx-chart` depend on the Presentation facade. That reverses the
  intended layering and gives a format model package ownership it does not
  need.
- Type plot kinds, axes, or series in this story. F-119 through F-122 own those
  surfaces and the structural rules reject speculative types.
- Keep shape and text properties opaque. Existing DrawingML implementers are
  usable today and the HLD explicitly includes them in `CT_ChartSpace`.
- Create separate model, parser, namespace, and test modules now. One source
  file keeps the initial core locally understandable until two real concerns
  justify a split.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `chart_space_reads_aliases_and_writes_fixed_prefixes_in_schema_order` | Aliased input, required roots, defaults, and fixed `c`, `a`, and `r` output |
| negative | `malformed_core_chart_values_return_errors_without_panicking` | Missing, duplicate, invalid boolean, and unknown enum inputs return contextual errors |
| preservation | `core_chart_shells_preserve_unmodelled_children_byte_for_byte` | Title, plot-area, legend, extension, and unknown root payloads retain bytes and positions |
| round-trip, gate | `every_corpus_chart_part_round_trips_structurally` | Every pinned corpus chart part parses, serializes, reparses, and compares as the same core model |
| unit | `rpptx_chart_is_an_unpublished_workspace_member` | Version 0.0.0, publication disabled, and only allowed normal dependencies |

The test gate is: a corpus chart part round-trips.

## HLD impact

- `docs/hld/09-charts-spec.md`
- `docs/hld/15-build-and-toolchain.md`

Document the implemented core boundary, the plot and axis preservation seam,
the shared `c:txPr` text-body path, corpus counts, dependency direction, and
unpublished crate status.

## Risk routing

- Any parser or serialiser. Read HLD 04 and HLD 06. Add prefix-alias,
  fixed-prefix, schema-order, malformed-value, structural round-trip, and
  byte-preservation checks over every corpus chart part.
- Crate dependency graph and new cross-family uses. Run
  `cargo tree -p rpptx-chart --edges normal` and prove that no `oxml-*` crate
  gains an `rdocx-*` or `rpptx-*` dependency. The ChartML crate may consume
  format-neutral OOXML crates only in this story.
- A new crate, module, or file. The explicit F-118 invocation authorizes the
  minimal manifest and single source file. The shared text parser change stays
  in its existing module and names its two current consumers.
- Version strings and packaging. Inspect the root manifest, new manifest,
  lockfile, HLD publishing text, and package contents. Require version 0.0.0,
  `publish = false`, an archive below 10 MiB, no release allowlist change, no
  tag, and no publication.

## Hash harness

Expected unchanged. The new unpublished ChartML model is not consumed by Word
sample generation or rendering. All 28 hashes must match.

## Implementation checklist

- [x] Register the minimal unpublished `rpptx-chart` crate.
- [x] Add ChartML namespaces, errors, enums, and core root types in one source
      file.
- [x] Add the caller-selected `c:txPr` root path to the existing DrawingML text
      body implementation.
- [x] Preserve unmodeled plot, title, legend, extension, and root XML in schema
      slots.
- [x] Add inline negative and prefix fixtures plus the corpus-wide gate.
- [x] Update exactly HLD 09 and HLD 15.
- [x] Run focused checks, dependency and package riders, microscope, and worker
      preparation.

## Open questions

None. F-118, the architecture, and the chart HLD explicitly authorize the
unpublished crate and fix the core type boundary. Later F-IDs own plots, axes,
and authoring.
