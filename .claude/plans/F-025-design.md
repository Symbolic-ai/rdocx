# F-025, MediaNamer

**Status**: approved
**Sprint**: S05
**Size**: S
**Depends on**: none

## Problem

The corrected Word image allocator scans numeric suffixes in
`crates/rdocx/src/document.rs:2762`, but the logic remains embedded in a
format-specific document type. `oxml-media` needs the same collision-safe
behavior for both `/word/media` and `/ppt/media` without counting parts or
accepting malformed near-matches.

## Spec reference

- `docs/hld/04-opc-and-packaging.md`, "Media".
- `docs/hld/12-testing-strategy.md`, "oxml-media".
- `docs/hld/14-development-backlog.md`, "F-025, MediaNamer".

## Approach

Add the public concrete `MediaNamer` to `oxml-media/src/lib.rs`. `scan` stores
the normalized directory and stem plus the occupied positive numeric suffixes
that exactly match them. `next_part_name(ext)` allocates after the maximum,
skips occupied values, wraps safely after `usize::MAX`, never emits zero, and
returns a package part name using the caller's extension.

Port the sentence-named F-005 regression cases into the staged crate as unit
tests. Leave the existing rdocx allocator and all consumers unchanged until
F-027.

## Rejected alternatives

- Store only `max + 1`. Keeping occupied suffixes is required for safe wrap at
  `usize::MAX`.
- Count matching parts. Sparse names such as image1 and image5 would collide.
- Rewire `Document` now. F-027 owns the consumer cutover.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `next_image_name_uses_the_highest_existing_index_not_the_part_count` | Sparse suffixes allocate after the maximum. |
| regression | `malformed_media_names_do_not_change_the_highest_image_index` | Wrong directory, stem, zero, sign, and nonnumeric names are ignored. |
| regression | `occupied_max_image_suffix_wraps_to_a_free_low_number` | Maximum suffix wraps without emitting zero or colliding. |
| regression | `max_minus_one_allocates_max_then_wraps_safely` | Sequential allocation crosses the integer boundary safely. |

The test gate is the naming assertions from F-005, now in the shared crate.

## HLD impact

None. The existing media contract already specifies maximum-suffix scanning.

## Risk routing

- Public API of a reserved crate. Add only the concrete planned type and its
  two methods, run the package and archive-size rider, and keep publication
  disabled.
- Generic parameter. The `impl Iterator<Item = &'a str>` input is instantiated
  by both slice iterators and package-key iterators in tests today, which
  justifies the planned generic input without a trait abstraction.

## Hash harness

Expected to remain unchanged. Existing rdocx naming remains the active path.

## Implementation checklist

- [ ] Add exact directory, stem, and positive suffix parsing.
- [ ] Add collision-safe allocation and integer-boundary wrap.
- [ ] Port all F-005 sentence-named regression cases.
- [ ] Confirm no rdocx consumer changes and run the unchanged hash gate.

## Open questions

None.
