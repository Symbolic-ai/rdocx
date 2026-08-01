# F-064, DrawingML text model

**Status**: completed
**Sprint**: S14
**Size**: XL
**Depends on**: F-053

## Problem

The original F-064 contract spans the complete DrawingML text hierarchy and is
too large for one reviewable implementation. The repository workflow requires
an XL story to split into natural letter-suffixed children, while retaining the
parent as the gate that closes only after all children close.

## Spec reference

- `docs/hld/05-drawingml-model.md`, "Text body" and "Preservation".
- `docs/hld/08-rendering-spec.md`, "Text in a shape" and "Autofit".
- `docs/hld/12-testing-strategy.md`, "The deck corpus".
- `docs/hld/14-development-backlog.md`, "F-064, DrawingML text model" and
  "F-064a" through "F-064d".

## Approach

Retain F-064 as an umbrella gate and implement its vocabulary through F-064a
to F-064d. The children own body properties and the root shell, paragraphs and
runs, bullets, then nine-level list styles and the complete text-body structural
round-trip. The `text/` module remains a wire model. Layout, inheritance, and
font resolution stay in their later milestones.

The available sprint evidence uses inline schema-valid XML fixtures. The
external deck corpus remains required at the M7 boundary, where it can validate
the completed model without committing binary fixtures.

## Rejected alternatives

- Keep one XL implementation. It would combine several natural schema
  boundaries and exceed the repository's review-size rule.
- Commit a binary deck fixture. The testing strategy reserves real decks for
  the external fetched corpus.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| round-trip | `schema_valid_text_body_fixture_round_trips_structurally` | The integrated child model preserves the complete `a:txBody` hierarchy |
| regression | `leading_and_trailing_text_whitespace_uses_xml_space_preserve` | The parent whitespace gate remains covered by F-064b |
| integration | `all_f064_children_are_completed_before_the_parent_closes` | F-064a through F-064d have durable plans, reviews, tests, and AS_BUILT entries |

The test gate is the integrated `a:txBody` structural round-trip plus the
external corpus run at the M7 boundary.

## HLD impact

- `docs/hld/14-development-backlog.md`

The completion check confirms the four child contracts still describe the
implemented split.

## Risk routing

- Any parser or serialiser: the child plans add the parser and serialiser rider
  and its schema-order, prefix, and raw-preservation checks.
- A new module or file: the child plans request the exact `text/` files before
  implementation.

## Hash harness

Expected to be unchanged. The unpublished DrawingML model has no Word consumer.

## Implementation checklist

- [x] Approve the F-064a through F-064d split in both backlogs and the sprint plan.
- [x] Complete F-064a, F-064b, F-064c, and F-064d with individual evidence.
- [x] Confirm the integrated text-body and whitespace gates pass.
- [x] Close the parent only after every child is complete.

## Open questions

None. The four-child split and inline schema-valid S14 fixtures are approved.
The external deck corpus remains the M7 boundary gate.
