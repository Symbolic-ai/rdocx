# F-095, Arrowheads

**Status**: approved
**Sprint**: S23
**Size**: S
**Depends on**: F-093

## Problem

`oxml-drawing::line::CT_LineProperties` models head and tail ends, but
`crates/rpptx-layout/src/context.rs` currently collapses only paint, width,
cap, join, and dash into `oxml_layout::Stroke`. Endpoint kind and size are
dropped before the frozen renderer boundary, so `rpptx-render` cannot lower
them into the filled paths required by the rendering spec.

Arrowheads need no backend-specific primitive. They do require a narrow,
backend-neutral endpoint value to survive resolution and a renderer helper
that derives the endpoint tangent from concrete path geometry.

## Spec reference

- `docs/hld/05-drawingml-model.md`, the `line.rs` module contract.
- `docs/hld/07-inheritance-and-resolution.md`, "The output contract".
- `docs/hld/08-rendering-spec.md`, "Why Group is the whole design" and "The
  PDF backend".
- `docs/hld/14-development-backlog.md`, "F-095, Arrowheads".

## Approach

Amend the existing unpublished `ResolvedShape` contract in
`crates/rpptx-layout/src/lib.rs` with optional neutral head and tail endpoint
values. Define `ResolvedLineEnd`, `ResolvedLineEndKind`, and
`ResolvedLineEndSize` beside `ResolvedShape`, covering the six DrawingML kinds
and three width or length sizes that already exist in `oxml-drawing`. These are
owned values and expose no source-model type.

Update the existing line resolution in `context.rs` to copy endpoint kind,
width, and length after inheritance. A missing kind or `none` becomes `None`.
Missing sizes use DrawingML's medium defaults.

In `crates/rpptx-render/src/lib.rs`, inspect the first and last non-degenerate
path segments. Lower each endpoint into an additional closed filled path using
the stroke paint and width-scaled endpoint dimensions. Reverse the start
tangent for head ends. Pin small, medium, and large width or length factors to
2, 3, and 5 times the stroke width. Triangle uses a tip and rectangular base,
stealth adds a half-length inset notch, diamond centres its widest points at
half length, oval uses four cubic segments, and arrow uses a closed chevron
whose arm thickness is one stroke width. Emit no path for `none`. F-093's
existing shape group contains the geometry and later F-094 transform work
extends that same group without changing endpoint lowering.

## Rejected alternatives

- Add arrowheads to `oxml_layout::Stroke`. They are a DrawingML lowering input,
  while the shared stroke type also serves Word output and backend primitives.
- Let each backend draw endpoint decorations. That duplicates geometry in PDF
  and raster code and contradicts the HLD's filled-path lowering.
- Preserve `oxml_drawing::LineEnd` in `ResolvedShape`. The frozen boundary must
  remain source-model free.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `line_end_resolution_keeps_kind_width_and_length` | DrawingML endpoint metadata becomes owned neutral values with correct defaults |
| integration | `triangular_tail_end_emits_an_extra_filled_path` | The backlog gate finds one extra closed path filled with the line paint at the path end |
| unit | `head_end_uses_the_reversed_start_tangent` | Head and tail decorations point away from opposite endpoints |
| unit | `all_supported_line_end_kinds_produce_finite_geometry` | Triangle, stealth, diamond, oval, and arrow variants emit bounded paths |
| regression | `zero_length_segment_omits_arrowhead_without_panicking` | Degenerate input remains safe and records no invalid coordinates |

The backlog test gate is
`triangular_tail_end_emits_an_extra_filled_path`.

## HLD impact

- `docs/hld/07-inheritance-and-resolution.md`
- `docs/hld/08-rendering-spec.md`

## Risk routing

- Layout and rendering. Read `docs/hld/08-rendering-spec.md`. Use generated
  in-memory paths and deterministic raster evidence. No system-font baseline
  is recorded.

## Hash harness

Expected to be unchanged. Arrowhead lowering is confined to unpublished
PowerPoint crates.

## Implementation checklist

- [ ] Add neutral resolved line-end values without exposing DrawingML types.
- [ ] Preserve inherited endpoint kind and dimensions during line resolution.
- [ ] Derive stable start and end tangents from concrete paths.
- [ ] Lower supported ends into additional filled paths using stroke paint.
- [ ] Prove triangle tail, direction, all kinds, and degenerate safety.
- [ ] Reconcile the HLD output contract with the approved narrow amendment.

## Open questions

Resolved. The user approved the narrow source-neutral endpoint amendment to
the unpublished F-087 output contract. Endpoint geometry uses the factors and
closed shapes pinned in this plan and HLD08.
