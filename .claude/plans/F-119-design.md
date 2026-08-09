# F-119, Series and data references

**Status**: approved
**Sprint**: S29
**Size**: L
**Depends on**: F-118

## Problem

F-118 establishes the ChartML root but deliberately leaves plot children
opaque. The next reusable layer is the `c:ser` payload and its category and
value references. ChartML references are not formula strings alone. Each one
must carry the literal cache most viewers and this repository's renderer use
when they do not open the embedded workbook.

The contract at `docs/hld/09-charts-spec.md:102` makes cache presence a
correctness requirement. A writer that emits a formula without its matching
cache produces an empty chart in common viewers, while independently supplied
formula and cache data can silently diverge.

## Spec reference

- `docs/hld/04-opc-and-packaging.md`, XML preservation and prefix contract.
- `docs/hld/05-drawingml-model.md`, shape-property reuse and preservation.
- `docs/hld/06-presentationml-model.md`, schema-ordered typed XML behavior.
- `docs/hld/09-charts-spec.md`, "The ChartML model" and "Cached values are not
  optional".
- `docs/hld/12-testing-strategy.md`, "Test taxonomy" and "The deck corpus".
- `docs/hld/13-risks-and-open-questions.md`, "R5, schema child ordering".
- `docs/hld/14-development-backlog.md`, "F-119, Series and data references".

## Approach

Extend the existing `rpptx-chart` crate root from F-118. Add no crate, module,
file, feature, dependency, trait, generic parameter, or forwarding wrapper.

Introduce concrete reference and cache values:

```rust
pub struct StringRef {
    pub formula: String,
    pub values: Vec<String>,
}

pub struct NumericData {
    pub formula: String,
    pub format_code: String,
    pub values: Vec<f64>,
}

pub enum AxisData {
    String(StringRef),
    Numeric(NumericData),
}

pub struct Series {
    pub index: u32,
    pub order: u32,
    pub name: Option<StringRef>,
    pub categories: Option<AxisData>,
    pub values: NumericData,
    pub bubble_size: Option<NumericData>,
    pub sp_pr: Option<CT_ShapeProperties>,
}
```

Constructors take each formula and value vector once and derive `ptCount`,
sequential point indexes, and cached `c:v` values during serialization. There
is no separate caller-supplied cache count or point index that could disagree.
Reject empty formulae, nonfinite numeric values, missing required index,
order, or values, duplicate modeled children, malformed point indexes, and a
declared point count that differs from the parsed points.

`Series::from_xml` and `to_xml` model `c:idx`, `c:order`, optional `c:tx`,
optional `c:spPr`, optional `c:cat`, required `c:val`, and optional
`c:bubbleSize`. The typed reference readers support `c:strRef` plus
`c:strCache`, and `c:numRef` plus `c:numCache`. Unsupported series children,
data points, labels, markers, trendlines, and producer extensions stay
byte-preserved in ordered schema slots for their owning later stories.

Add narrow internal helpers on the F-118 plot-area shell to enumerate and
structurally validate every nested `c:ser` from preserved plot children without
claiming a plot kind. Serialization continues to use the preserved plot bytes
until F-121 and F-122 type those containers. Standalone typed series writing is
the authoring seam those plot stories consume.

Corpus coverage extracts every immediate `c:ser` under supported chart plot
containers, parses and reparses it, and compares typed values while retaining
unsupported children. The story gate constructs a series from one in-memory
data vector and asserts that its formula, cache count, point indexes, and
literal values remain consistent after round-trip.

## Rejected alternatives

- Store formulae and caches as unrelated caller inputs. That permits the exact
  divergence the story exists to prevent.
- Parse Excel formula syntax or depend on `oxml-sml`. F-119 depends only on
  F-118, and formula evaluation belongs to neither ChartML nor the minimal
  workbook writer.
- Type plot containers now. Bar and line plots belong to F-121, while the
  remaining plot families belong to F-122.
- Drop unsupported data labels, points, markers, or extensions. They remain
  raw until their owning story and must survive byte-identically.
- Add a generic cache type. String and numeric caches have different required
  children and are already the two concrete instantiations needed today.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit, gate | `series_formula_and_cache_are_consistent_with_one_source` | Formula, `ptCount`, indexes, format code, and cached literals derive from one supplied value vector |
| unit | `string_and_numeric_references_write_fixed_prefixes_in_schema_order` | `c:f`, cache metadata, points, and values use fixed prefixes and required order |
| negative | `malformed_series_and_cache_values_return_errors_without_panicking` | Missing required children, duplicate fields, invalid counts, indexes, and nonfinite values return errors |
| preservation | `series_preserves_unmodelled_children_byte_for_byte` | Markers, labels, points, trendlines, extensions, attributes, and whitespace retain bytes and positions |
| round-trip | `every_corpus_series_round_trips_structurally` | Every corpus `c:ser` under a chart plot reparses with equal references and caches |

The test gate is: a chart written with a cache and a formula reference has both
consistent with one source of data.

## HLD impact

- `docs/hld/09-charts-spec.md`

Document the concrete string and numeric reference API, derived cache
invariants, validation errors, preserved series boundary, and observed corpus
series count.

## Risk routing

- Any parser or serialiser. Read HLD 04 and HLD 06. Add alias-prefix,
  fixed-prefix, schema-order, malformed-value, corpus structural round-trip,
  and byte-preservation checks for series and reference subtrees.

No crate graph, published API, binding, external oracle, feature, new file,
version, release, layout, unit-conversion, or baseline rider applies.

## Hash harness

Expected unchanged. The unpublished ChartML data model is not consumed by Word
sample generation or rendering. All 28 hashes must match.

## Implementation checklist

- [ ] Add concrete string, numeric, category, and series values to the existing
      crate root.
- [ ] Derive cache metadata and points from one supplied value vector.
- [ ] Parse and write the modeled series children in schema order.
- [ ] Preserve unsupported series payloads in their original slots.
- [ ] Add the consistency gate, negative cases, and corpus-wide series
      round-trip coverage.
- [ ] Update exactly HLD 09.
- [ ] Run focused parser, corpus, preservation, microscope, and worker
      preparation checks.

## Open questions

None. F-119 and HLD 09 fix the required reference and cache behavior. Plot
containers, workbook integration, and facade authoring belong to later F-IDs.
