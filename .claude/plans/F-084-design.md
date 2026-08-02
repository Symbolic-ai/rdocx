# F-084, Format scheme reference resolution

**Status**: completed
**Sprint**: S20
**Size**: M
**Depends on**: F-063, F-065

## Problem

DrawingML already models fill, line, effect, and font references at
`crates/oxml-drawing/src/style_ref.rs:82`, plus theme format lists at
`crates/oxml-drawing/src/theme.rs:558`. Ordinary `CT_Shape`, however, retains
`p:style` as opaque bytes at `crates/rpptx-oxml/src/shape_tree.rs:488`. A parsed
shape therefore cannot select a theme entry, substitute `phClr`, or layer its
explicit shape properties on top.

The current HLD also describes all four `p:style` references as numeric
one-based indices at `docs/hld/07-inheritance-and-resolution.md:113`.
`a:fontRef@idx` is instead `major`, `minor`, or `none`, which the existing
`FontCollectionIndex` correctly models at
`crates/oxml-drawing/src/style_ref.rs:122`.

## Spec reference

- `docs/hld/05-drawingml-model.md`, "Theme", "Colour, the part everyone gets
  wrong", and "Style references".
- `docs/hld/06-presentationml-model.md`, "The shape tree" and
  "Preservation strategy".
- `docs/hld/07-inheritance-and-resolution.md`, "Style references" and
  "Colour".
- `docs/hld/14-development-backlog.md`, "F-084, Format scheme reference
  resolution".

## Approach

Extend the existing ordinary-shape model with an optional typed
`CT_ShapeStyle` containing concrete line, fill, effect, and font reference
fields. Parse `p:style` prefix-tolerantly after `p:spPr`, write fixed prefixes
in `xsd:sequence`, and preserve unmodelled attributes and children through the
existing ordered raw-child machinery. Build on F-082's typed
`CT_ShapeProperties` rather than introducing a second representation.

Add `style.rs` to `rpptx-layout`. Resolve line, fill, and effect numeric
references with one-based indexing. Index zero means no referenced theme
style. A positive out-of-range index returns `ResolveError::StyleIndexOutOfRange`
rather than panicking or silently choosing a value. Fill indices above 1000
select `background_fill_styles[(idx - 1000) - 1]`.

Clone the chosen format entry and substitute every modelled
`schemeClr val="phClr"` with the reference colour. Keep reference transforms
first and then the placeholder colour's transforms so document order remains
deterministic. Fill choices and typed effects are atomic overlays. Explicit
line properties merge per property over the referenced line so omitted shape
properties remain inherited. Explicit `noFill` replaces the theme fill.
Unsupported raw effect children remain preserved and are not claimed as
resolved. If raw XML contains an unresolved `phClr`, return a specific resolver
error so a later renderer cannot consume a falsely concrete value.
Resolve opaque `effectDag` and nested `schemeClr` names against their XML
namespace before classifying them. Same-named producer extensions remain
opaque and cannot replace a DrawingML effect or trigger a placeholder-colour
error.

Keep font collection selection typed as `major`, `minor`, or `none`. F-085
owns typeface-token lookup inside the selected collection.

The first normal-stack corpus run also exposed an integrated F-082 regression.
Adding the typed `CT_ShapeProperties` inline had expanded `CT_Shape` enough for
the five-level group tree in `themes.pptx` slide master 3 to overflow the test
thread stack. Store the required public shape properties as
`Box<CT_ShapeProperties>`. Ordinary field access remains available through
`Box` dereferencing, while recursive parsing no longer carries the large
property value inline. `rpptx-oxml` is unpublished at version `0.0.0`, so this
public storage-type correction has no published semver impact.

## Rejected alternatives

- Treat `a:fontRef@idx` as a number. That contradicts OOXML and the existing
  correct parser model.
- Rewrite raw effect XML to replace `phClr`. Byte rewriting would bypass the
  typed parser and could corrupt namespace or transform semantics.
- Silently clamp or ignore an out-of-range numeric index. Malformed input must
  not select an unrelated style.
- Set `RUST_MIN_STACK` for the corpus test. That hides a library stack-footprint
  regression and leaves ordinary callers exposed. The required gate must pass
  with the normal test-thread stack.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `shape_with_style_resolves_theme_fill_with_reference_colour_substituted` | The backlog shape resolves its theme fill with the reference colour |
| unit | `fill_ref_1001_selects_first_background_fill` | The background-fill rule uses one-based indexing |
| unit | `style_matrix_zero_is_none_and_out_of_range_is_error` | Zero and malformed positive indices follow distinct policies |
| unit | `placeholder_colour_substitution_keeps_transform_order` | Reference and placeholder transforms remain in the declared order |
| unit | `explicit_shape_properties_overlay_the_theme_style` | Fill and effect replacement plus line property overlay follow precedence |
| unit | `unmodelled_effect_with_placeholder_colour_is_rejected` | The resolver never reports an opaque unresolved colour as concrete |
| unit | `foreign_namespace_effect_dag_does_not_replace_theme_effect` | A same-named producer extension cannot suppress a referenced DrawingML effect |
| unit | `foreign_namespace_scheme_color_is_not_a_placeholder_color` | A foreign `schemeClr` cannot trigger DrawingML placeholder-colour rejection |
| round-trip | `ordinary_shape_style_round_trips_in_schema_order` | Alternate prefixes parse, fixed prefixes write, and raw siblings survive exactly |
| regression | `ordinary_shape_properties_round_trip_in_schema_order` | Boxed public shape properties retain ordinary field access and structural round trips |
| corpus | `all_corpus_modelled_parts_reparse_structurally` | All 50 decks pass on the normal test-thread stack after typing `p:style` and boxing shape properties |

The backlog test gate is named explicitly:
`shape_with_style_resolves_theme_fill_with_reference_colour_substituted`.

## HLD impact

- `docs/hld/06-presentationml-model.md`
- `docs/hld/07-inheritance-and-resolution.md`

## Risk routing

- Theme colour, tint, shade, and colour mapping. Preserve transform order and
  do not modify `rdocx_oxml::theme::apply_tint_shade`. Run focused exact-colour
  tests and require all 28 hashes unchanged.
- Any parser or serialiser. Recheck `p:style` schema order,
  prefix-tolerant reads, fixed-prefix writes, and byte preservation of
  unmodelled children. Require namespace-aware inspection of opaque effects and
  run the required corpus structural round-trip gate.
- A new module or file. `src/style.rs` is justified by the current F-084
  implementation and requires the shared explicit approval recorded in F-081.
- Public storage type in an unpublished crate. `CT_Shape::shape_properties`
  changes to `Box<CT_ShapeProperties>` in unpublished `rpptx-oxml 0.0.0`, with
  no published semver impact. Run the normal-stack 50-deck corpus gate and
  retain direct field-access coverage.

## Hash harness

Expected to be unchanged. PowerPoint theme resolution does not change the Word
rendering baseline.

## Implementation checklist

- [x] Type ordinary-shape `p:style` in the existing schema position.
- [x] Resolve normal and background format-list indices without clamping.
- [x] Substitute modelled `phClr` values while preserving transform order.
- [x] Overlay explicit fill, line, and effect properties.
- [x] Keep same-named foreign effect extensions opaque during inspection.
- [x] Keep font collection selection separate from F-085 typeface lookup.
- [x] Bound recursive parser stack use without a larger-stack test workaround.
- [x] Add focused parser, resolver, and corpus regressions.
- [x] Correct the font-reference wording in the inheritance HLD.

## Open questions

None. Numeric zero is no style, malformed positive indices are errors, and
opaque unresolved placeholder colours are rejected rather than misreported.
