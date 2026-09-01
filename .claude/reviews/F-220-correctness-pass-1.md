# F-220, correctness, pass 1

**Reviewed**: claim-base `3d87d827351508b38ffb22103f26351c3f18c0ca` through exact source head `5567051e4e9ff1983a6ca5005151b8504f608a3f`, 8 files and 1,930 changed lines (1,886 additions, 44 deletions), plus the approved revised plan, cited HLD sections, F-219 final contract and review, progress record, and full tracked and untracked state
**Verdict**: 11 defects, 0 smells, 0 nitpicks

## Defects

### D1, unsupported algorithm trees and conditional branches render as if they were supported
`crates/rpptx-layout/src/diagram.rs:66`

The renderer dispatches solely on the coarse `layout.family`. It never validates the ordered algorithm identities in `render_projection`, and unknown constraint or rule types are silently ignored at `crates/rpptx-layout/src/diagram.rs:479`. The projection also recursively flattens `choose`, `if`, and `else` children into one unconditional record stream at `crates/rpptx-oxml/src/diagram.rs:1201`, without retaining or evaluating the condition function. A layout whose identity contains `list`, for example, but whose tree contains an unsupported algorithm or mutually exclusive branches is rendered as a plausible list instead of the required labelled fallback and diagnostic.

### D2, the renderer lays out data nodes instead of the presentation graph and silently drops impossible edges
`crates/rpptx-layout/src/diagram.rs:38`

The node set includes only `Node` and `Assistant` points and excludes `Presentation` points. `graph_edges` then retains a supported relationship only when both ends happen to be in that reduced set at `crates/rpptx-layout/src/diagram.rs:182`. A normal presentation mapping such as a `presOf` connection from a data node to a presentation point disappears, and a `parOf` connection with a missing endpoint also disappears. The resulting graph can render with the wrong node count and no connectors instead of using presentation-node ownership or rejecting an impossible graph.

### D3, the declared six-family geometry contract is not implemented
`crates/rpptx-layout/src/diagram.rs:338`

Relationship accepts every positive node count, uses a generic cycle for one to three nodes, and switches to a grid above three. It neither enforces the approved two-to-six bound nor places nodes around a declared shared centre. Hierarchy sends assistant points through the ordinary level layout because node kind is no longer consulted after collection at `crates/rpptx-layout/src/diagram.rs:285`. Cycle reads start angle and direction but ignores projected `spanAng` at `crates/rpptx-layout/src/diagram.rs:315`. Matrix has only a column count and no title-band path at `crates/rpptx-layout/src/diagram.rs:346`. Pyramid assigns every tier the same height rather than consuming level weights at `crates/rpptx-layout/src/diagram.rs:372`. The unit test checks only three positive finite rectangles at `crates/rpptx-layout/src/context.rs:4079`, so all of these contract violations pass.

### D4, style and colour labels are flattened instead of being selected by their schema owner
`crates/rpptx-layout/src/diagram.rs:83`

All fill and line lists from every named colour label are concatenated and cycled by global node index. The first line reference from any quick-style label is converted into an invented stroke-width formula at `crates/rpptx-layout/src/diagram.rs:95`, while fill, effect, and font references are ignored. Every node is then emitted as the same hard-coded rounded rectangle at `crates/rpptx-layout/src/diagram.rs:129`. A document with different labels for parent, child, assistant, or connector shapes therefore applies unrelated colours and does not use the existing DrawingML shape-style engine.

### D5, SmartArt text bypasses the shared DrawingML text engine
`crates/rpptx-layout/src/diagram.rs:134`

The complete `CT_TextBody` is reduced to `plain_text`, then `shape_label` emits one black, non-bold, non-italic `GlyphRun` in hard-coded Carlito at `crates/rpptx-layout/src/diagram.rs:526`. Paragraphs, runs, list style, body properties, theme font and colour, wrapping, vertical direction, anchoring, insets, autofit, and text clipping are all discarded. A supported node with formatted or multi-paragraph text renders differently from an ordinary DrawingML shape, contrary to the shared-engine contract.

### D6, positive sub-point frame bounds can panic in constraint application
`crates/rpptx-layout/src/diagram.rs:503`

The entry check accepts every finite positive width and height at `crates/rpptx-layout/src/diagram.rs:31`. `resize_axis` then forces the resized extent to at least 1 point. When the frame extent is less than 1 point, the upper clamp bound becomes smaller than the lower bound at `crates/rpptx-layout/src/diagram.rs:504`, and `f64::clamp` panics. A supported layout with a positive width or height constraint inside a sub-point graphic frame can therefore abort the process instead of returning a visible fallback.

### D7, accepted spacing and dense-list geometry can move nodes outside the authoritative bounds
`crates/rpptx-layout/src/diagram.rs:475`

Every positive `sp` or `sibSp` value shifts each successive rectangle without a containment check. Dense list layouts similarly force each cell to at least 1 point while retaining an 8-point gap at `crates/rpptx-layout/src/diagram.rs:217`. The postcondition checks only finiteness and positive extents at `crates/rpptx-layout/src/diagram.rs:79`, so large spacing or enough nodes produces valid-looking rectangles outside the frame. The group clip then hides those nodes rather than treating the constraint or geometry as impossible and falling back visibly.

### D8, recursive layout projection has no depth bound
`crates/rpptx-oxml/src/diagram.rs:1149`

`collect_render_projection` recursively descends every nested `layoutNode`, `forEach`, `choose`, `if`, and `else` at `crates/rpptx-oxml/src/diagram.rs:1201` without a counter or iterative stack. The renderer's `MAX_DEPTH` protects only hierarchy graph traversal, after parsing. A deeply nested but well-formed layout definition can overflow the process stack before the renderer can emit an unsupported diagnostic.

### D9, the implementation adds unapproved public `rpptx-layout` API
`crates/rpptx-layout/src/lib.rs:91`

`DiagramResources` and all of its fields, plus `ScopedDiagramResources` and its maps, are public. Two new public resolver entry points are also exposed at `crates/rpptx-layout/src/context.rs:434` and `crates/rpptx-layout/src/context.rs:490`. `#[doc(hidden)]` removes documentation visibility, not Rust API visibility. The approved API impact permits exactly the two doc-hidden `rpptx-oxml` projections and says the other published API remains unchanged, so this is a larger low-level public surface than reviewed.

### D10, the mandatory PowerPoint gate lacks a tracked trust anchor and cannot identify the promised slide or diagnostics
`crates/rpptx/tests/integration.rs:170`

The only manifest is loaded from the same external directory as the mutable source and oracle artifacts, and no textual manifest is tracked at this head. Source bytes, oracle bytes, and their adjacent hashes can therefore be replaced together without a reviewed repository change. The row schema omits both exact slide and expected diagnostics at `crates/rpptx/tests/integration.rs:202`, selects the first supported group across every slide at `crates/rpptx/tests/integration.rs:223`, then always rasterizes page zero at `crates/rpptx/tests/integration.rs:268`. A multi-slide source can compare geometry from one slide with pixels from another. The progress record confirms that the six-family manifest is absent at `.claude/scratch/F-220-progress.md:67`, so ordinary mode returns success without differential evidence and required mode currently fails before comparing any family.

### D11, the negative-sensitivity test does not exercise the differential gate
`crates/rpptx/tests/integration.rs:340`

The geometry half creates a copied `Rect`, adds 1.01, and asserts that 1.01 is greater than 1.0. It does not perturb rendered SmartArt or feed the result through the manifest comparator. The pixel half compares two synthetic scalar arrays at `crates/rpptx/tests/integration.rs:349` through a local SSIM implementation, not the PNG decode and Python helper invoked by the real gate. The test continues to pass if slide selection, bounds extraction, PNG generation, manifest parsing, or the real SSIM invocation stops detecting the approved perturbations.

## Smells

None.

## Nitpicks

None.

## Not found

- **OOXML namespaces, schema positions, and raw preservation**: The render projections use expanded-name checks at schema-owned positions, accept namespace aliases, reject fixed-prefix shadows and foreign lookalikes, retain transform order, and leave `to_xml` byte exact. The focused `rpptx-oxml` projection regression passed.
- **Shared colour transforms**: Once a colour choice has been selected, `resolve_color` applies the existing theme map and ordered DrawingML transforms at `crates/rpptx-layout/src/diagram.rs:567`. No separate colour-transform implementation was found.
- **Producing-scope assembly**: Slide, layout, and master resource maps remain separate at `crates/rpptx/src/lib.rs:6792`, and lookup uses the flattened producing source. No relationship-id alias across those three scopes was found.
- **Static, timeline, and media assembly**: All three public deterministic paths prepare the same scoped diagram resources. The focused facade regression observed the edited node in static, timeline, and media output. No separate SmartArt renderer branch was found in `rpptx-render`.
- **F-219 preservation contract**: The F-220 delta does not weaken F-219's checked edit, transfer, raw-byte preservation, relationship-role, or atomic-publication behavior.
- **Structure**: The new private diagram module was explicitly approved. No new trait, generic, feature, crate, dependency, dynamic dispatch, or integration binary was added.
- **Validation evidence**: `cargo test -p rpptx-layout --lib smartart_` passed 3 tests. The focused `rpptx-oxml` projection test passed. `cargo test -p rpptx --test integration smartart_` passed 16 selected tests in ordinary mode, including the skipped-on-missing-oracle differential case. Re-running that case with `RDOCX_PPTX_CORPUS_REQUIRED=1` failed at the documented missing manifest before any comparison. `git diff --check` passed.
