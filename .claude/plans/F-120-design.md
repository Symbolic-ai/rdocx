# F-120, Axes

**Status**: completed
**Sprint**: S30
**Size**: L
**Depends on**: F-118

## Problem

F-118 leaves the plot area as an ordered raw shell at
`crates/rpptx-chart/src/lib.rs:1873`, and F-119 adds only a read-only series
projection at `crates/rpptx-chart/src/lib.rs:1930`. There is no typed model for
category, value, date, or series axes, and no validation that axis identifiers
and `crossAx` references form reciprocal pairs.

The pinned corpus also exposes a producer compatibility requirement that the
nominal unsigned schema type does not reveal. PowerPoint writes negative
lexical axis identifiers in representative bar and line decks. Rejecting those
values would make valid corpus charts unreadable, while treating identifiers
as unvalidated strings would make pairing errors invisible.

## Spec reference

- `docs/hld/04-opc-and-packaging.md`, XML preservation and prefix contract.
- `docs/hld/05-drawingml-model.md`, shape and text property reuse.
- `docs/hld/06-presentationml-model.md`, schema-ordered typed XML behavior.
- `docs/hld/09-charts-spec.md`, "The ChartML model" and the axis contract.
- `docs/hld/12-testing-strategy.md`, "Test taxonomy" and "The deck corpus".
- `docs/hld/13-risks-and-open-questions.md`, "R5, schema child ordering".
- `docs/hld/14-development-backlog.md`, "F-120, Axes".

## Approach

Extend the existing `rpptx-chart` crate root. Add no crate, file, module,
feature, dependency, trait, generic parameter, or forwarding wrapper.

Introduce one axis value with a root-kind discriminator instead of four
near-duplicate structures:

```rust
pub enum AxisKind { Category, Value, Date, Series }

pub struct AxisId(i64);

pub struct Scaling {
    pub log_base: Option<f64>,
    pub orientation: Orientation,
    pub maximum: Option<f64>,
    pub minimum: Option<f64>,
}

pub struct Axis {
    pub kind: AxisKind,
    pub id: AxisId,
    pub scaling: Scaling,
    pub deleted: bool,
    pub position: AxisPosition,
    pub major_gridlines: Option<ChartLines>,
    pub minor_gridlines: Option<ChartLines>,
    pub title: Option<CT_Title>,
    pub number_format: Option<NumberFormat>,
    pub major_tick_mark: TickMark,
    pub minor_tick_mark: TickMark,
    pub tick_label_position: TickLabelPosition,
    pub sp_pr: Option<CT_ShapeProperties>,
    pub tx_pr: Option<CT_TextBody>,
    pub cross_axis: AxisId,
}
```

`AxisId` accepts the observed producer range from `i32::MIN` through
`u32::MAX`. Parsed negative PowerPoint values retain their lexical form when
unchanged, while equality and pairing use one normalized identifier domain.
Values outside that range are errors. The newtype is justified because axis
and cross-axis references are two current uses of the same non-obvious domain.

`Axis::from_xml` accepts `c:catAx`, `c:valAx`, `c:dateAx`, or `c:serAx` under
any prefix bound to ChartML. `Axis::to_xml` writes the fixed root and fixed
`c:`, `a:`, and `r:` prefixes. Require `axId`, `scaling`, `axPos`, and
`crossAx`. Validate enum values, booleans, finite scaling bounds, identifier
ranges, and duplicate modelled children.

Type the common axis sequence through `crossAx`. Preserve `crosses`,
`crossesAt`, type-specific category, value, date, and series children,
extensions, unknown attributes, comments, and whitespace byte-for-byte in
ordered schema slots. `ChartLines` types only optional shape properties.
`NumberFormat` carries the common `formatCode` and `sourceLinked` attributes
and becomes the reviewed concrete value F-123 reuses for data labels.

Add `CT_PlotArea::axes() -> Result<Vec<Axis>>` as a read-only projection over
direct axis children. Validate each nonempty axis set as one graph: ids are
unique, every `crossAx` resolves in the same plot area, and references are
reciprocal. Reject self-crossing, dangling, duplicate-id, and one-way pairs.
An empty set remains valid for axis-free plots. F-121 and F-122 own replacing
preserved plot-area slots with authored plot and axis collections.

## Rejected alternatives

- Parse every id as `u32`. Real PowerPoint corpus charts contain negative
  lexical values that still pair consistently.
- Keep axis ids as strings. That preserves bytes but cannot enforce the story's
  reciprocal pairing gate.
- Add four axis structs. Their implemented common sequence is one concrete
  behavior with a root-kind choice, not four independent abstractions.
- Type plots in this story. F-121 and F-122 own those containers.
- Type every axis tail. Unsupported type-specific children remain raw until a
  story requires their behavior.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit, gate | `axis_id_pairs_are_reciprocal` | Category and value axes accept producer-compatible signed ids and reject duplicate, self, dangling, and one-way references |
| unit | `all_axis_forms_write_fixed_prefixes_in_schema_order` | Category, value, date, and series axes write required fields, scaling, gridlines, ticks, labels, shape, text, and cross references in order |
| negative | `malformed_axis_values_return_errors_without_panicking` | Missing required fields, invalid ranges, nonfinite values, unknown enums, invalid booleans, and duplicates return contextual errors |
| preservation | `axes_preserve_unmodelled_children_byte_for_byte` | Aliased input, type-specific tails, extensions, attributes, comments, and whitespace retain bytes and positions |
| round-trip, gate | `every_corpus_axis_round_trips_structurally` | Every corpus axis serializes and reparses equally, every nonempty set pairs reciprocally, and date plus series axes have non-vacuous inline coverage |

The test gate is: axis id pairing is consistent, and a corpus chart's axes
round-trip.

## HLD impact

- `docs/hld/09-charts-spec.md`

Replace the future axis paragraph with the implemented API, producer-compatible
identifier range, reciprocal-pair invariant, validation and preservation
rules, and observed corpus coverage.

## Risk routing

- Any parser or serialiser. Read HLD 04 and HLD 06. Add alias-prefix,
  fixed-prefix, exact schema-order, malformed-value, producer signed-id,
  byte-preservation, and corpus structural round-trip checks.

No crate graph, published API, binding, external oracle, feature, new file,
version, release, layout, unit-conversion, or baseline rider applies.

## Hash harness

Expected unchanged. The unpublished ChartML model is not consumed by Word
sample generation or rendering. All 28 hashes must match.

## Implementation checklist

- [x] Add the producer-compatible `AxisId` domain and axis enums.
- [x] Parse and write the four axis roots and common schema sequence.
- [x] Preserve unsupported axis content in ordered raw slots.
- [x] Add plot-area axis projection and reciprocal-pair validation.
- [x] Add inline negative, ordering, preservation, and all-form fixtures.
- [x] Add the non-vacuous corpus axis gate and record coverage.
- [x] Update exactly HLD 09.
- [x] Run focused parser, corpus, preservation, microscope, and worker
      preparation checks.

## Open questions

None. The signed identifier range is resolved by real PowerPoint corpus
evidence, and later stories own plot containers and rendering geometry.
