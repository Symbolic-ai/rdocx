# F-220, SmartArt layout and rendering

**Status**: completed
**Sprint**: S62
**Size**: L
**Depends on**: F-219

## Problem

`crates/rpptx-layout/src/context.rs` currently resolves every
`GraphicDataPayload::SmartArt` to an unsupported bounds fallback. Once F-219
provides typed data, layout, style, colour, and relationship ownership, the
resolver still needs deterministic placement for the six layout families in
the story and must lower them through existing DrawingML geometry, paint, and
text paths.

The completed F-219 model identifies layout families and exposes flattened
algorithm and constraint names, but its initial render projection loses the
nested instruction ownership used by the six authentic PowerPoint programs.
In particular, flattened records cannot preserve which `layoutNode`,
`forEach`, `choose`, condition, `shape`, or `presOf` instruction owns an
algorithm, constraint, rule, parameter, decorative shape, or text mapping.
Reparsing preserved XML in `rpptx` or `rpptx-layout` would create a second
diagram model and violate the parser boundary. F-220 therefore expands the
existing doc-hidden read-only layout projection and retains the existing
doc-hidden colour projection. The colour projection closes the final F-219
colour-model gap because the ordinary string lists retain colour values but
not the schema-owned choice kind or ordered transforms required for faithful
rendering.

SmartArt producers can encode many algorithm variants and cached drawings. The
supported subset must be explicit and must never make an unsupported algorithm
disappear. Geometry and pixels must be compared with a pinned PowerPoint oracle
at declared tolerances, using deterministic fonts on the Rust side.

## Spec reference

- ECMA-376 Part 1, DrawingML diagram layout algorithms, constraints, rules,
  style labels, colour transforms, and presentation nodes.
- `docs/hld/02-scope-and-non-goals.md`, the SmartArt scope table.
- `docs/hld/03-architecture.md`, resolver and renderer seams.
- `docs/hld/06-presentationml-model.md`, typed SmartArt layout projection and
  raw-preservation boundary.
- `docs/hld/07-inheritance-and-resolution.md`, graphic-frame resources,
  visible unsupported fallbacks, group transforms, and scoped media.
- `docs/hld/08-rendering-spec.md`, "The renderer's input", deterministic
  presentation entry points, text shaping, and SmartArt fallback behavior.
- `docs/hld/10-bindings-spec.md`, additive pre-1.0 low-level Rust surface and
  unchanged native facade and binding policy.
- `docs/hld/12-testing-strategy.md`, "Test taxonomy", "The deck corpus", and
  native presentation differential evidence.
- `docs/hld/14-development-backlog.md`, "F-220, SmartArt layout and
  rendering".

## Approach

Complete and integrate F-219 first, then consume its exact public diagram
types. Do not introduce a second SmartArt object tree or parse XML in the
layout crate.

Extend `CT_DiagramLayoutDefinition` and `CT_DiagramColorsDefinition` with two
doc-hidden read-only projections populated by their existing namespace-aware
parsers. Keep the existing flattened layout fields for compatibility and add
one nested instruction root:

```rust
#[doc(hidden)]
pub struct DiagramRenderProjection {
    pub algorithms: Vec<(String, Vec<(String, String)>)>,
    pub constraints: Vec<Vec<(String, String)>>,
    pub rules: Vec<Vec<(String, String)>>,
    pub root: Option<DiagramRenderInstruction>,
}

#[doc(hidden)]
pub struct DiagramRenderInstruction {
    pub kind: DiagramRenderInstructionKind,
    pub attributes: Vec<(String, String)>,
    pub children: Vec<DiagramRenderInstruction>,
}

#[doc(hidden)]
pub enum DiagramRenderInstructionKind {
    LayoutNode,
    Algorithm,
    Parameter,
    ConstraintList,
    Constraint,
    RuleList,
    Rule,
    ForEach,
    Choose,
    Condition,
    Else,
    Shape,
    PresentationOf,
    VariableList,
    Variable(String),
    AdjustmentList,
    Adjustment,
    Unsupported(String),
}

impl CT_DiagramLayoutDefinition {
    #[doc(hidden)]
    pub fn render_projection(&self) -> &DiagramRenderProjection;
}

#[doc(hidden)]
pub struct DiagramColorRenderProjection {
    pub labels: Vec<DiagramColorRenderLabel>,
}

#[doc(hidden)]
pub struct DiagramColorRenderLabel {
    pub name: String,
    pub fill: Vec<ColorChoice>,
    pub line: Vec<ColorChoice>,
    pub effect: Vec<ColorChoice>,
    pub text_fill: Vec<ColorChoice>,
}

impl CT_DiagramColorsDefinition {
    #[doc(hidden)]
    pub fn render_projection(&self) -> &DiagramColorRenderProjection;
}
```

The nested projection contains only the schema-owned instruction kinds needed
by the six pinned programs. Each node retains ordered decoded attributes and
ordered projected children. `forEach`, `choose`, condition, nested layout
ownership, layout variables, and shape adjustments therefore remain explicit
rather than being inferred from flattened records. Unknown instruction kinds,
attributes, enum values, or unsupported children remain preserved in the
source XML, project only a decoded unsupported local name, and make native
evaluation fail closed. Namespace aliases are accepted, fixed-prefix shadows
and extension lookalikes are excluded, and no raw XML or mutation surface is
exposed. This is an additive low-level pre-1.0 `rpptx-oxml` surface for the
private `rpptx` consumer. The native `rpptx` facade, `rpptx-layout`,
`rpptx-render`, Python, WASM, and CLI APIs remain unchanged. Colour choices use
the existing DrawingML `ColorChoice` model so choice kind, value, and ordered
transforms resolve through the shared colour engine without exposing raw XML.

Add one private diagram module at `crates/rpptx/src/diagram.rs`. The module was
initially placed in `rpptx-layout`, but correctness review showed that doing so
required new public resource carriers and resolver entry points and encouraged
SmartArt-specific text shaping. Relocating the same single private module to
the package facade is the smallest correction because that crate already owns
the scoped OPC resources and can prepare ordinary PresentationML shapes for
the unchanged resolver without publishing a bridge API.

The private module validates the typed F-219 data graph and its presentation
graph separately. `presOf` connections map authoritative data-node text to the
presentation node that owns the rendered shape. `presParOf` and supported
`parOf` connections provide checked presentation-node topology. Missing,
ambiguous, duplicate, cyclic, or out-of-bounds ownership fails closed.

The supported layout-program evaluator is an interpreter for the finite nested
instruction subset exercised by the six exact pinned authentic resources. It
evaluates bounded layout-node ownership, `forEach` axis and point selection,
ordered `choose` branches and conditions, direct shape creation, `presOf`
authoritative data ownership, algorithms and parameters, constraints, and
rules. It also creates the decorative shapes and connectors owned by those
programs. Selection is deterministic by typed point role, connection role,
source and destination ordinals, model id, and the instruction's declared
axis. Iteration, instruction depth, selected points, generated shapes, and
condition evaluation are bounded before any transient shape is allocated.

The evaluator implements only the algorithm, parameter, constraint, rule,
selector, condition, reference, shape, and geometry forms reached by the six
pinned instruction trees. Every record is evaluated under its owning layout
node and iteration context. An unknown attribute, enum, axis, condition
function, reference, geometry, or ownership combination fails closed with the
visible SmartArt bounds fallback. This prevents a partially understood
authentic program from producing plausible wrong output.

For supported input, the module replaces a SmartArt graphic frame only in the
transient cloned slide, layout, or master used for rendering. It inserts one
PresentationML group containing ordinary shapes and connectors whose geometry
comes from the typed data, layout, style, and colour definitions. The group
maps its child coordinate space through the authoritative graphic-frame
offset, extent, rotation, and flips.
Each ordinary shape retains the complete authoritative `CT_TextBody` and the
schema-owner-selected quick-style and colour references. The existing
`ResolveCtx` then performs normal DrawingML style, colour, effect, font, text,
wrapping, anchoring, autofit, clipping, and geometry resolution. This is not
cached-drawing rendering, not mutation of the opened package, and not a second
persistent SmartArt model. Parsing remains exclusively in `rpptx-oxml`.
Cached drawing availability or parse success never gates the supported native
layout path.
`rpptx-render` gains no SmartArt branch, module, dependency, or public API.

Recognize only these exact layout identities and their validated projected
instruction shapes:

- `urn:microsoft.com/office/officeart/2005/8/layout/list1`
- `urn:microsoft.com/office/officeart/2005/8/layout/hierarchy1`
- `urn:microsoft.com/office/officeart/2005/8/layout/cycle1`
- `urn:microsoft.com/office/officeart/2009/3/layout/CircleRelationship`
- `urn:microsoft.com/office/officeart/2005/8/layout/matrix1`
- `urn:microsoft.com/office/officeart/2005/8/layout/pyramid1`

The exact pinned `cycle1` resource has one narrow compatibility exception.
OOXML defines its cycle and radial-connector inputs but leaves the generating
application's diameter packing and connector route unspecified. For the exact
PowerPoint 16.104 resource bytes pinned below, an exact three-node data graph,
and the validated projected instruction tree, the private evaluator may use a
reverse-engineered normalized placement and connector profile from the common
PowerPoint oracle. A different resource hash, identity, instruction shape, or
node count fails closed. This exception does not permit measured geometry for
another family or cycle variant and does not add a public API, dependency,
module, or persistent model.

The generator and external manifest additionally bind the exact layout bytes
to the approved SHA-256 pins below. At runtime an identity match is necessary
but not sufficient. The complete projected instruction tree must validate
against the supported finite subset before evaluation. Other identities,
missing owners, unsupported tree shapes, parameters, duplicate parameters,
or algorithm attributes fail closed. Presentation nodes and connectors are
normalized by typed `srcOrd` and `destOrd`, with stable model-id tie breaking.
Duplicate semantic edges and cycles fail before placement.
Unsupported algorithms, condition functions, impossible graphs, invalid
constraints, non-finite results, or resource failures retain the current
visible bounds fallback and emit a stable diagnostic naming the layout identity
and unsupported reason. A failure in one diagram does not hide sibling shapes.

Style labels choose existing DrawingML shape styles. Colour labels resolve
through the existing theme colour map and transform logic. Node text uses the
F-219 `CT_TextBody` value and the shared DrawingML text engine, including
deterministic font selection, paragraph layout, vertical anchoring, and clipping.
Connectors use existing path and stroke values. The outer graphic-frame
transform remains authoritative, and the complete diagram group is clipped to
its bounds.

Package assembly keeps scoped parsed diagram resources private to `rpptx` and
expands each producing slide, layout, or master clone before constructing the
existing resolver. Layout lookup is restricted to that producing scope. The
existing static, timeline, media, and animation facade paths all retain the
expanded clones and therefore gain SmartArt rendering through their existing
entry points. All new `rpptx-layout` public resource carriers and resolver
entry points are removed.

Expansion also records the authoritative graphic-frame bounds for each
transient child, keyed inside its producing scope. After the unchanged shared
lowerer creates ordinary render groups, the private `rpptx` assembly pass adds
the equivalent child-local bounds clip, including the resolver's accumulated
parent-group scale, and composes it with any existing timeline clip. This is
transient render metadata, not a second object model or a new `rpptx-layout`
or `rpptx-render` public surface.

No new trait, generic, feature, crate, dependency, integration binary, or
binary asset is added. The new module is private and the published API remains
unchanged.

## Rejected alternatives

- Rendering only the cached diagram drawing would not make supported layouts
  respond to F-219 node edits.
- Reimplementing DrawingML text or paint inside a SmartArt renderer would make
  two engines disagree on fonts, colours, and clipping.
- Passing SmartArt XML into `rpptx-render` would reverse the resolver boundary.
- Guessing unsupported layout algorithms from point count would create
  plausible but incorrect output without a diagnostic.
- Recording only screenshot hashes would hide geometry errors behind
  antialiasing noise.
- Comparing authentic PowerPoint resources against a separate synthetic Rust
  program is not a differential. It asks two renderers to evaluate different
  inputs and can neither explain nor fix a mismatch.
- Approximating authentic outputs with family-specific hard-coded geometry
  would discard instruction ownership and fail to respond correctly to bounded
  data-graph changes. The sole exception is the exact SHA-pinned three-node
  `cycle1` compatibility profile documented above because its native diameter
  and connector solver is not specified by OOXML or the resource program.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `authentic_smartart_programs_place_owned_and_decorative_shapes_deterministically` | Exact normalized bounds, order, ownership, connector endpoints, finite transforms, and clipping for every text-bearing and decorative shape generated by the six pinned programs. |
| unit | `authentic_smartart_instructions_constraints_rules_styles_and_colours_resolve_in_owner_order` | Nested iteration and conditions, supported constraints and rules, style labels, theme mapping, colour transforms, and authoritative text ownership use the shared engines in schema order. |
| round-trip | `smartart_render_projection_reads_only_schema_owned_nested_instructions_and_preserves_raw_xml` | Alias prefixes project ordered nested layout nodes, loops, choices, conditions, shapes, `presOf`, algorithms, parameters, constraints, rules, colour choices, and transforms, while namespace shadows and extension lookalikes remain excluded and source bytes remain exact. |
| regression | `unsupported_or_invalid_authentic_smartart_program_remains_visible_and_does_not_hide_siblings` | Stable diagnostic and bounds fallback appear for altered identities or instruction trees, unsupported selectors and conditions, malformed graphs, missing parts, bounds exhaustion, and non-finite geometry. |
| integration | `authentic_smartart_rendering_uses_producing_scope_and_updates_only_authoritative_text_owner` | Colliding relationship ids do not alias, one F-219 text edit changes only shapes whose `presOf` mapping owns that data node, decorative shapes remain stable, and static, timeline, media, and animation paths agree. |
| differential | `supported_smartart_corpus_matches_pinned_powerpoint_geometry_and_ssim` | Both renderers consume the same source deck. Every family records oracle identity, one source hash, render hashes, exact dimensions, text-bearing and decorative geometry, diagnostics, maximum point error, full-image luminance SSIM as a diagnostic, symmetric text-masked non-text SSIM, and ordered text-line ink metrics. |
| regression | `smartart_differential_rejects_geometry_and_pixel_perturbations` | A one-point node displacement and calibrated pixel mutation fail the declared thresholds. |

The exact backlog **test gate is differential**: "The supported corpus renders
within the declared PowerPoint geometry and SSIM thresholds."

Use Microsoft PowerPoint 16.104, Info.plist build 16.104.25121423, and
AppleScript build 1214 as the pinned oracle already used by the presentation
timeline differential. A non-GUI ignored generator writes six schema-valid
one-slide PowerPoint source decks from the standard package template. Each deck
has deterministic data-only document and node points, exact data, layout,
quick-style, and colour roles, and no cached drawing. It embeds authentic
PowerPoint SmartArt layout, style, and colour resources from
`SmartArt.framework/Versions/A/Resources` only after verifying these pins:

- `lo/list1.glo`: `e0e143896ab59ea7bf1e5bcf88151467271e0baad2f998c2e3e663384b985dee`
- `lo/hierarchy1.glo`: `92d8fd7ff62d662670cbe4aa723f0565b98d577e18d3e9a26c3901a64f68bf31`
- `lo/cycle1.glo`: `8a7e35b9099cff9fd646490ab9b36f8349d82e2568c92354f871f90315301461`
- `lo/circlerelationship.glo`: `0de1977f023d64027ebac0bb5ea27522690423b504a9ff577e381a727585bcd6`
- `lo/matrix1.glo`: `19a09a8304e33a88048fe66ba1d9e0488da1c4d6d0ea694f18279879612e1255`
- `lo/pyramid1.glo`: `5cb535131a8ee862690c56455f7c56d75a68ccac7c2f7ee81da638de1af6eba2`
- `qs/simple1.gqs`: `213edbaedea282b8d13dc85a874c9494e66b90546118e40e32614d939a704bd1`
- `cs/accent1_1.gcs`: `dc0a610ca9a665158d3afaff8612e29320e009f151990f74a265197a4e96f9a4`

These copyrighted resource bytes remain external and are not copied into the
repository. Each generated deck is the single common input to PowerPoint and
Rust. The Rust side reopens that exact deck through the normal facade and
evaluates the embedded authentic layout, style, colour, and data parts. No
synthetic `urn:f220:*` layout or separate Rust source is generated or accepted.
The generator does not launch PowerPoint or claim oracle provenance. A human
captures the PowerPoint PDF and normalized PNG for each exact common source
outside the repository.

The external trust anchor `f220-smartart-oracle.tsv` lives beside those
artifacts under the ignored corpus directory. Its six rows pin one common
source, every PDF, and every normalized PNG by SHA-256 and record family, exact
one-based slide, PowerPoint version and both build identities, output
dimensions, normalized text-bearing and decorative shape bounds with explicit
ownership, and exact expected diagnostics. The harness rejects a second source
column, placeholder hashes, omitted decorative geometry, or inconsistent
ownership. No trust anchor is committed until all six PowerPoint artifacts
exist and their provenance is verified.

Render Rust output with bundled deterministic fonts at literal 150 dpi.
Normalize the verified PowerPoint render and matching deterministic Rust slide
to the same dimensions. Cached `dsp:drawing` is not required and remains
outside the render path. The ordinary gate skips with the exact missing
external path. `RDOCX_PPTX_CORPUS_REQUIRED` fails when the trust anchor or any
pinned artifact is absent.

Every manifest-owned text-bearing or decorative shape permits at most 1 point
absolute error per edge. The symmetric text-masked non-text luminance SSIM
must be at least 0.90 for every family. Full-image luminance SSIM remains a
recorded diagnostic and is not a pass threshold because the pinned Calibri
oracle and bundled metric-compatible Carlito renderer have different raster
covariance. Text comparison requires the exact line count and owner order and
permits at most 3 points per ink-box edge. Exact resolved text, ownership,
diagnostics, dimensions, and source, PDF, and PNG provenance remain mandatory.
Line detection runs independently inside each exact text-owner shape in owner
order. Top and bottom ink edges remain in raw slide coordinates. Horizontal
ink edges are recentered on the independently gated owner-shape centre before
comparison so trailing wrap whitespace and Calibri to Carlito advance-width
covariance cannot masquerade as placement error. Line ink-width difference is
also capped at 3 points. Because every owner shape edge and centre is already
gated at 1 point, recentering cannot hide horizontal owner placement.
The ordinary-shape control measures the renderer ceiling, while the negative
test proves that a 1.01-point displacement and calibrated paint and size
mutation fail the applicable thresholds. Missing artifacts fail when
`RDOCX_PPTX_CORPUS_REQUIRED` is set and skip with an explicit reason otherwise.
Classify every divergence as a Rust defect, an oracle limitation, or a
documented ECMA-376 interpretation.

Add unit tests inside the new layout module and integration cases to the
existing `crates/rpptx/tests/integration.rs` binary. Do not add an integration
binary or committed binary fixture.

## HLD impact

- `docs/hld/02-scope-and-non-goals.md`
- `docs/hld/03-architecture.md`
- `docs/hld/06-presentationml-model.md`
- `docs/hld/07-inheritance-and-resolution.md`
- `docs/hld/08-rendering-spec.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- Layout, pagination, line breaking, and text shaping: use bundled
  deterministic fonts for every unit baseline and differential render. Run
  `cargo test -p oxml-layout --no-default-features` in addition to the normal
  gate.
- Any parser or serializer: populate the nested render projection through the
  existing namespace-aware schema-position walk. Add alias-prefix,
  fixed-prefix-shadow, extension-lookalike, nested-owner order,
  structural-reparse, and byte-exact raw preservation coverage in the existing
  `rpptx-oxml` integration binary.
- Crate dependency graph and cross-family uses: keep parsing in `rpptx-oxml`,
  transient expansion in `rpptx`, and `rpptx-render` on the unchanged
  backend-neutral values. Run
  `cargo tree -p rpptx-layout -e normal`, `cargo tree -p rpptx-render -e normal`,
  and the shared-crate dependency-direction test.
- New module or file: explicit approval covers the relocated single private
  `crates/rpptx/src/diagram.rs`. Six related algorithms share checked graph,
  constraint, and transient-shape preparation while remaining separate from
  the already large facade. No second diagram module remains in
  `rpptx-layout`.
- External oracle comparison: follow
  `.claude/skills/differential-testing.md`. Pin PowerPoint 16.104 and both build
  identities, SHA-256-pin the source and rendered artifacts, use literal 150
  dpi, permit at most 1 point geometry error, require symmetric text-masked
  non-text luminance SSIM of at least 0.90, require exact ordered text lines
  with at most 3 points per ink-box edge, retain full-image SSIM as a
  diagnostic, fail required-corpus mode on missing evidence, and classify
  every divergence.
- Public API of a published crate: the existing doc-hidden layout projection
  gains one read-only nested instruction root and doc-hidden instruction
  carrier types in pre-1.0 `rpptx-oxml`, with one private `rpptx` consumer. The
  existing colour projection remains unchanged. Run patched publish dry-runs
  for `rpptx-oxml`, `rpptx-layout`, and `rpptx`, then assert every generated
  archive remains below 10 MiB. No facade, layout, renderer, or binding surface
  is added.

The supported public facade API is unchanged. The expanded doc-hidden layout
projection and existing doc-hidden colour projection earn the published-crate
package rider above.

## Hash harness

Expected unchanged, 49 of 49. The existing hash harness does not contain a
SmartArt presentation render. Any delta is unexplained and blocks integration.

## Implementation checklist

- [x] Complete the F-219 dependency-prefix checkpoint and reconcile its final
  diagram model and resource types.
- [x] Relocate the approved private diagram module to `rpptx`, keep scoped
  resources private, and remove the unapproved `rpptx-layout` public path.
- [x] Implement checked list, hierarchy, cycle, relationship, matrix, and
  pyramid placement.
- [x] Resolve the supported constraint subset, style labels, colours,
  connectors, and text through shared engines, while unsupported rules fail
  closed.
- [x] Preserve visible bounded fallbacks and stable diagnostics for every
  unsupported or invalid case.
- [x] Reuse the same resolved group across static, timeline, media, and
  animation assembly.
- [x] Expand the doc-hidden layout projection with the bounded nested
  instruction tree and parser-preservation regressions.
- [x] Replace synthetic family placement with the finite interpreter for the
  six exact pinned authentic instruction trees, including decorative shapes
  and authoritative `presOf` text ownership. Connector lowering may correct a
  malformed trailing zero in the bundled `circularArrow` preset definition
  before the unchanged shared DrawingML geometry evaluator runs. The repair is
  covered in `oxml-drawing` and does not add a second geometry implementation.
- [x] Make the ignored generator and manifest use one common authentic source
  deck per family and reject the former split-source contract.
- [x] Add deterministic unit, integration, differential, and
  negative-sensitivity regressions to existing targets.
- [x] Run the pinned PowerPoint common-source corpus gate and every routed
  rider.

## Open questions

None. The material revision to the existing private layout module and
doc-hidden OXML projection is approved. Both renderers consume the same six
authentic source decks. The PowerPoint 16.104 differential remains at 150 dpi,
at most 1 point geometry error, symmetric text-masked non-text luminance SSIM
of at least 0.90, and at most 3 points per ordered text-line ink-box edge.
Full-image SSIM remains diagnostic only.
