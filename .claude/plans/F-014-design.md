# F-014, New unit types

**Status**: completed
**Sprint**: S03
**Size**: M
**Depends on**: F-013

## Problem

The shared units planned for DrawingML do not exist. The current unit surface
at `crates/rdocx-oxml/src/units.rs:3` covers Word twips, EMUs, and half-points,
while `Length` at `crates/rdocx/src/length.rs:11` has no millimetre constructor
or accessor.

The new conversions must preserve the repository's pinned truncation rule.
Rounding would shift layout without a compile failure.

## Spec reference

- `docs/hld/01-glossary.md`, "Units".
- `docs/hld/05-drawingml-model.md`, "Colour, the part everyone gets wrong".
- `docs/hld/11-migration-plan.md`, "Preserve behaviour, do not improve it".
- `docs/hld/12-testing-strategy.md`, "New tests the extracted crates need",
  subsection `oxml-core`.

## Approach

Extend `oxml_core::units` with tuple newtypes `Centipoints(i32)`, `Angle(i32)`,
and `Percent1000(i32)`. Add converting constructors and accessors using the
exact storage scales from the glossary. Add `Length::mm` using 36,000 EMUs per
millimetre. Keep float-to-integer conversion as Rust
casts so positive and negative fractional inputs truncate toward zero.

Expose the new types through the existing `oxml_core::units` public module. Do
not add a generic unit trait because there are no two implementations that need
one today.

## Rejected alternatives

- Round converted values. Existing unit behaviour deliberately truncates.
- Use one generic scaled-unit wrapper. It would add a generic parameter and
  make distinct schema units easier to mix accidentally.
- Correct the legacy Word tint and shade path while adding `Percent1000`. That
  behaviour is deliberately frozen for the extraction.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `centipoints_round_trip_points` | `Centipoints::from_pt(18.0).0 == 1800` and converts back |
| unit | `angle_round_trip_degrees` | `Angle::from_degrees(90.0).0 == 5_400_000` and converts back |
| unit | `percent1000_round_trip_percent` | 75 percent stores as 75,000 and converts to fraction 0.75 |
| unit | `new_unit_float_constructors_truncate_toward_zero` | Positive and negative fractional conversions do not round |
| unit | `length_millimetres_round_trip` | Millimetres convert through EMUs without changing existing constructors |

The backlog test gate is the exact `Angle`, `Centipoints`, and `Percent1000`
round-trip assertions.

## HLD impact

None. The glossary and DrawingML model already specify these types and scales.

## Risk routing

- Unit conversion. Preserve truncation, add positive and negative tests, and
  require an unchanged deterministic hash harness.
- Public API of a published family. Treat the additions as semver-compatible,
  run `cargo package -p oxml-core`, and assert the archive is below 10 MiB.

## Hash harness

Expected to remain unchanged. The new types have no rdocx call sites.

## Implementation checklist

- [x] Add the three concrete unit newtypes and exact conversions.
- [x] Add millimetre support to `Length`.
- [x] Add the backlog assertions and truncation-discriminating cases.
- [x] Run focused `oxml-core` tests, packaging, and the hash harness.

## Open questions

None.
