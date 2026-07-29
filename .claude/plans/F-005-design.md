# F-005, Fix the image counter

**Status**: approved
**Sprint**: S01
**Size**: S
**Depends on**: none

## Problem

`Document::from_package` sets `image_counter` by counting matching part names
at `crates/rdocx/src/document.rs:134`. Sparse names therefore collide with an
existing or lower suffix instead of allocating one past the greatest suffix.

## Spec reference

- `docs/hld/04-opc-and-packaging.md`, "Part naming".
- `docs/hld/13-risks-and-open-questions.md`, "The image counter".

## Approach

Scan package keys under `/word/media/image`, parse the consecutive decimal
digits immediately after `image`, ignore names without a valid positive
numeric suffix, and initialize the counter to the maximum parsed value or zero.
Keep `next_image_number()` unchanged so it returns one greater than that
maximum.

Add the named regression cases to the existing
`crates/rdocx/tests/regression_test.rs` integration binary. Construct sparse
packages in code, reopen them through `Document`, add an image, serialize, and
inspect the resulting package names.

## Rejected alternatives

- Counting parts was rejected because deletion makes the sequence sparse.
- Searching for the first gap was rejected because the specification requires
  monotonic allocation after the maximum existing suffix.
- Adding a generic media-namer abstraction was rejected because there is only
  one implementer in this story.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `next_image_name_uses_the_highest_existing_index_not_the_part_count` | `image1` plus `image5` yields `image6`, while `image1`, `image2`, and `image4` yield `image5`. |
| unit | malformed media-name cases | Unrelated prefixes and nonnumeric suffixes do not affect the maximum. |

The **test gate** is
`next_image_name_uses_the_highest_existing_index_not_the_part_count`.

## HLD impact

- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/13-risks-and-open-questions.md`

## Risk routing

none

## Hash harness

Expected to be unchanged. The existing samples create dense media names from a
new package and do not exercise sparse imported names.

## Implementation checklist

- [ ] Replace the part-count initialization with maximum-suffix parsing.
- [ ] Add both required sparse-name regression cases to the existing test binary.
- [ ] Add malformed-name coverage without adding a new module or test binary.
- [ ] Run the focused rdocx regression test target.

## Open questions

None.
