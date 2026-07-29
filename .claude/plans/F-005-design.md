# F-005, Fix the image counter

**Status**: completed
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
numeric suffix, and initialize the counter to the true maximum parsed value or
zero. This includes `usize::MAX`. For each allocation, use checked increment
and wrap from `usize::MAX` to 1. Skip every occupied parsed image number until a
free positive suffix is found. Ordinary packages still allocate one greater
than their maximum existing suffix.

Add the named regression cases to the existing
`crates/rdocx/tests/regression_test.rs` integration binary. Construct sparse
packages in code, reopen them through `Document`, add an image, serialize, and
inspect the resulting package names.

## Rejected alternatives

- Counting parts was rejected because deletion makes the sequence sparse.
- Starting from the first gap for every allocation was rejected because
  ordinary packages require monotonic allocation after the maximum existing
  suffix. Low gaps are considered only after the numeric suffix space wraps.
- Adding a generic media-namer abstraction was rejected because there is only
  one implementer in this story.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `next_image_name_uses_the_highest_existing_index_not_the_part_count` | `image1` plus `image5` yields `image6`, while `image1`, `image2`, and `image4` yield `image5`. |
| regression | `malformed_media_names_do_not_change_the_highest_image_index` | Unrelated prefixes and missing, signed, zero, or nonnumeric suffixes do not affect the maximum. |
| regression | `occupied_max_image_suffix_wraps_to_a_free_low_number` | Existing `usize::MAX - 1` and `usize::MAX` parts are preserved, occupied low suffixes are skipped, and allocation uses a free positive low suffix without creating `image0`. |
| regression | `max_minus_one_allocates_max_then_wraps_safely` | An existing `usize::MAX - 1` part is preserved, the first allocation uses `usize::MAX`, and the second wraps safely to a free positive suffix. |

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

- [x] Replace the part-count initialization with maximum-suffix parsing.
- [x] Add both required sparse-name regression cases to the existing test binary.
- [x] Add malformed-name coverage without adding a new module or test binary.
- [x] Add collision-safe checked wrap at the finite suffix boundary.
- [x] Add both upper-boundary regression cases to the existing test binary.
- [x] Run the focused rdocx regression test target.

## Open questions

None.
