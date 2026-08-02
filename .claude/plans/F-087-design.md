# F-087, ResolvedSlide contract

**Status**: completed
**Sprint**: S21
**Size**: M
**Depends on**: F-082, F-083, F-084, F-085, F-086

## Problem

The resolver currently exposes effective DrawingML values one property at a
time from `crates/rpptx-layout/src/lib.rs:8`. No public owned slide boundary
combines the flattener with concrete geometry, paint, text, and diagnostics.
Without that boundary, the M10 renderer could consume `p:` and `a:` model types
directly and duplicate theme and inheritance logic.

The backend-neutral primitives already exist in `oxml-layout`, including
`Rect`, `MediaId`, `Diagnostic`, `Paint`, and `Stroke` at
`crates/oxml-layout/src/output.rs:14` and
`crates/oxml-layout/src/paint.rs:12`. `rpptx-layout` does not yet depend on that
crate, and it has no frozen `ResolvedSlide` type set.

## Spec reference

- `docs/hld/05-drawingml-model.md`, "Colour, the part everyone gets wrong",
  "Geometry", "Fills", "Lines and arrowheads", "Effects", and "Text".
- `docs/hld/07-inheritance-and-resolution.md`, "The output contract", "Draw
  order", "The chains", and "The resolver".
- `docs/hld/08-rendering-spec.md`, "The seam that makes this cheap" and
  "Extending PositionedElement".
- `docs/hld/14-development-backlog.md`, "F-087, ResolvedSlide contract".

## Approach

Add a one-way dependency from `rpptx-layout` to the existing unpublished
`oxml-layout` crate. Define the complete public contract in the existing
`rpptx-layout/src/lib.rs`, using backend-neutral owned values only:

```rust
pub struct ResolvedSlide {
    pub size: (f64, f64),
    pub background: Option<Paint>,
    pub shapes: Vec<ResolvedShape>,
    pub diagnostics: Vec<Diagnostic>,
}

pub struct ResolvedShape {
    pub group_transform: Transform,
    pub bounds: Rect,
    pub rotation_deg: f64,
    pub flip_h: bool,
    pub flip_v: bool,
    pub geometry: ResolvedGeometry,
    pub fill: Option<Paint>,
    pub line: Option<Stroke>,
    pub shadow: Option<Effect>,
    pub content: ResolvedContent,
    pub unsupported: Option<&'static str>,
}
```

Define the remaining geometry, text-body, table, crop, and content types beside
these in `lib.rs`. Reuse `oxml-layout` primitives where they already express
the renderer-neutral value. Do not expose a PresentationML or DrawingML model
type. Media content uses `MediaId`, matching the frozen HLD contract. The later
RenderInput story owns relationship-to-media resolution before constructing
image content.

Add a concrete resolving entry point on `ResolveCtx` in `context.rs`. It
consumes F-086's ordered view, skips shapes without a usable extent, converts
EMU to points without changing the pinned constructor behavior, resolves every
colour through the context colour map and theme, and copies only concrete text
properties. Unsupported chart, SmartArt, ink, OLE, and unknown geometry remain
present as bounding-box fallbacks with diagnostics rather than leaking source
XML or disappearing.

The two required 50-deck resolver gates add `oxml-opc` as a test-only
dependency of `rpptx-layout`. Those tests are the existing concrete consumers.
They open each pinned package and resolve its slide, layout, master, theme, and
presentation-default chain. Production dependencies and production code do
not use `oxml-opc`. The manifest change updates the `rpptx-layout` dependency
entry in `Cargo.lock` without adding a package because `oxml-opc` is already a
workspace member.

Keep all work in existing files. No trait, generic parameter, wrapper-only
type, module, source file, or feature flag is introduced.

## Rejected alternatives

- Re-export DrawingML fill, line, and text values. That leaves theme references
  in the renderer contract and defeats the milestone.
- Return borrowed source nodes. The renderer boundary must be owned and must
  not retain part-tree lifetimes.
- Drop unsupported content. The rendering contract requires a visible fallback
  and diagnostic rather than silent data loss.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| integration | `corpus_slide_resolves_without_theme_references` | The backlog gate finds no unresolved theme references anywhere in output |
| unit | `resolved_contract_contains_no_presentation_or_drawing_types` | Public fields use only owned resolver and backend-neutral types |
| unit | `transform_resolves_to_points_rotation_and_flips` | Bounds and orientation match the effective transform exactly |
| unit | `theme_style_resolves_to_concrete_paint_line_and_shadow` | Theme and placeholder colours are fully collapsed |
| unit | `unsupported_content_keeps_bounds_and_diagnostic` | Unsupported content remains visible to the renderer contract |
| unit | `resolved_shapes_follow_the_flattened_order` | Contract order exactly matches F-086 output |
| corpus | `all_corpus_slides_resolve_without_panics` | Every resolvable corpus slide produces an owned contract or contextual error |

The backlog test gate is named explicitly:
`corpus_slide_resolves_without_theme_references`.

## HLD impact

- `docs/hld/07-inheritance-and-resolution.md`
- `docs/hld/08-rendering-spec.md`

## Risk routing

- Unit conversion, `Emu`, and points. Read the units glossary and preserve
  pinned truncation constructors. Assert exact point conversion at the output
  boundary and require an unchanged hash harness.
- Theme colour, tint, shade, and colour mapping. Use only the spec-correct
  `oxml-drawing` resolver, do not touch the Word path, and run exact-colour
  tests.
- Layout and rendering boundary. Record no baseline in system-font mode. Any
  render comparison uses deterministic font mode.
- Crate dependency graph. Run `cargo tree -p rpptx-layout --edges normal` and
  confirm the edge points from `rpptx-layout` to `oxml-layout`, with no reverse
  dependency from an `oxml-*` crate. Also run
  `cargo tree -p rpptx-layout --edges dev` and confirm the test-only
  `rpptx-layout -> oxml-opc` edge does not enter the normal graph.
- Public API in unpublished crates. Record that the contract freezes
  `rpptx-layout` at version 0.0.0 with publication disabled. Inspect the
  manifest and lockfile diff and run the full publication dry-run gate without
  publishing.

## Hash harness

Expected to be unchanged. The new PowerPoint output contract is not connected
to Word rendering.

## Implementation checklist

- [x] Add the one-way `oxml-layout` dependency.
- [x] Add the test-only `oxml-opc` dependency for both 50-deck gates.
- [x] Define the complete owned `ResolvedSlide` type set in `lib.rs`.
- [x] Convert effective transforms and styles into concrete neutral values.
- [x] Preserve unsupported content with bounds and diagnostics.
- [x] Resolve a corpus slide without source-model or theme references.
- [x] Freeze and document the renderer-facing contract.

## Open questions

None. The HLD fixes the owned type set, `MediaId` boundary, unsupported-content
policy, and dependency direction.

## Design deviations

- Remediation pass 1 replaced three in-memory corpus-shaped fixtures with both
  required 50-deck gates. This requires the test-only `oxml-opc` dependency
  described above. No production dependency or runtime package boundary
  changed.
- Remediation pass 1 added a backend-neutral accumulated group transform to
  each resolved shape. This prevents nested group transforms from being lost
  before the frozen renderer boundary and follows the existing HLD group seam.
- Remediation passes 1 and 2 expanded the owned text contract with paragraph
  spacing and complete bullet styling. Character and auto-number bullets both
  retain independently inherited font, colour, size, and choice values.
