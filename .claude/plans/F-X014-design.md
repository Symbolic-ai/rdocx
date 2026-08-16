# F-X014, Kashida justification values

**Status**: completed
**Sprint**: S41
**Size**: S
**Depends on**: none

## Problem

`ST_Jc::from_str` in `crates/rdocx-oxml/src/shared.rs:19` enumerates `start`,
`left`, `end`, `right`, `center`, `both`, `justify` and `distribute`, and
returns `OxmlError::InvalidValue` for anything else. `ST_Jc` is missing the
three Arabic justification values the schema defines: `lowKashida`,
`mediumKashida` and `highKashida`.

The consequence is larger than a wrong alignment. `CT_PPr::from_xml` at
`properties.rs:154` propagates the rejection with `?`, and that error travels
out through `CT_P::from_xml` and `CT_Document::from_xml` to `Document::open`. A
document carrying one of the three therefore **fails to open at all**. Confirmed
directly:

```
ERROR: invalid value: invalid ST_Jc: lowKashida
```

Kashida justification stretches Arabic text by elongating the connecting stroke
rather than by widening spaces. Rendering that faithfully needs shaping work
this crate does not do. Justified alignment is the correct approximation and is
what the three values mean at the paragraph level.

## Spec reference

- `docs/hld/12-testing-strategy.md`, "Test taxonomy" for the regression
  category, and "The hash harness" for the labelled-delta rule.
- `docs/hld/14-development-backlog.md`, "F-X014, Kashida justification values".

## Approach

Add the three values to the `both | justify` arm:

```rust
"both" | "justify" | "lowKashida" | "mediumKashida" | "highKashida" => Ok(ST_Jc::Both),
```

`to_str` is unchanged. `ST_Jc::Both` already writes back as `both`, so a
round trip normalises a kashida value to plain justification. That is a real
loss of fidelity and it is the right trade for now: preserving the exact value
would mean a variant that renders identically to `Both` everywhere, which adds a
case a reader must consider for no behavioural difference. Recorded here so the
normalisation is a decision rather than an accident.

Nothing else changes. The layout engine already handles `ST_Jc::Both`.

## Rejected alternatives

- **A distinct `Kashida` variant.** A new case in every match over `ST_Jc`, all
  of them behaving exactly as `Both`, to preserve a value nothing reads. Fails
  the test in `CLAUDE.md`: it increases the places a reader must look without
  reducing the cases they must consider.
- **Make `from_str` infallible and default to `Left`.** That is the right answer
  for the general problem and the wrong scope for this story. Filed as F-X018,
  which decides the rule for all nine enumerations rather than changing one in
  passing.
- **Map to `Distribute`.** `distribute` spreads the last line too. Kashida
  justification does not.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `a_document_using_kashida_justification_still_opens` | A document whose paragraph carries each of the three values parses, and the paragraph keeps its justification and its sibling properties |
| unit | `kashida_justification_maps_to_both` | Each of the three values parses to `ST_Jc::Both` |
| unit | `an_unknown_justification_is_still_rejected` | A genuinely unknown string still returns `InvalidValue`, so this story widens the accepted set rather than removing the check |

**Test gate**, from the backlog: the regression, named as a sentence describing
the failure it prevents.

## HLD impact

None. The story adds three accepted spellings to a value the spec set already
describes.

## Risk routing

Matched row: **Any parser or serialiser**.

- Prefix-tolerant on read, fixed prefix on write. Unchanged here: this touches
  an attribute value, not an element name or a namespace.
- The round-trip consequence is that a kashida value is written back as `both`.
  Stated above as a deliberate normalisation and covered by the test plan.

The layout row does not match. No pagination, line breaking or shaping code is
touched, and a paragraph that previously could not be loaded at all has no
previous rendering to change.

## Hash harness

**Expected unchanged.** No corpus document carries a kashida justification
value, and no existing behaviour moves: the affected documents currently fail to
open rather than rendering differently.

## Implementation checklist

- [x] Add the three values to the `both | justify` arm
- [x] Add the regression and the two unit tests
- [x] Confirm the regression fails against the unwidened parser
- [x] `/microscope F-X014 --working`
- [x] `/verify`

## Open questions

None.
