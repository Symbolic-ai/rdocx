# F-005, correctness, pass 3

**Reviewed**: F-005 implementation working diff, 3 files with 209 additions
and 15 deletions
**Verdict**: 0 defects, 0 smells, 1 nitpick

## Defects

None.

## Resolved

### D1, The upper-bound suffix wraps without overflow or collision

`crates/rdocx/src/document.rs:526`
`crates/rdocx/src/document.rs:534`

The shared parser retains the true maximum representable positive suffix,
including `usize::MAX`. Allocation uses checked increment, wraps the maximum to
1, and advances again for every occupied parsed suffix before updating the
counter. The two boundary regressions prove that an occupied maximum is
preserved, `usize::MAX - 1` can allocate the maximum once, later allocation
wraps safely, occupied low numbers are skipped, and `image0` is never created.

## Smells

None.

## Nitpicks

- `crates/rdocx/src/document.rs:2473`, the doc comment describing the
  lower-cased file extension is attached to `image_number_from_part_name()`
  instead of `image_extension()`.

## Not found

No maximum-parsing defect remains. `Document::from_package` computes the
maximum over every valid positive suffix, and the shared parser handles
`usize::MAX` rather than filtering it out.

No loop-progress defect was found. Each occupied candidate advances through
checked addition or wraps from `usize::MAX` to 1, and the loop stops at the
first unoccupied positive suffix.

No gap was found in the upper-bound evidence. The occupied-maximum test covers
the collision case from pass 2 and low-suffix skipping. The max-minus-one test
covers allocating `usize::MAX` followed by a second safe wrapped allocation.
Both preserve the pre-existing parts and pass independently.

No regression was found in the original sparse or malformed-name gates. Both
required sparse layouts allocate one greater than the ordinary maximum, while
missing, signed, zero, unrelated and nonnumeric suffixes remain ignored. All
four focused image-counter regressions pass.
