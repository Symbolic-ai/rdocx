# F-005, correctness, pass 2

**Reviewed**: F-005 working diff, 3 files with 138 additions and 7 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, The upper-bound suffix can still overflow or collide

`crates/rdocx/src/document.rs:149`
`crates/rdocx/src/document.rs:539`
`crates/rdocx/src/document.rs:553`

Filtering out `usize::MAX` prevents that value from directly seeding the
counter, but `usize::MAX - 1` still seeds it. If a package contains both
`image{usize::MAX - 1}.png` and `image{usize::MAX}.png`, the next allocation
increments to `usize::MAX` and `set_part` overwrites the existing maximum part.
If the package contains only `image{usize::MAX - 1}.png`, one allocation uses
`usize::MAX` and a second allocation overflows. The upper valid suffix is
therefore not safely usable, and the unallocatable maximum can still cause a
collision. The boundary needs explicit exhaustion or collision handling.

The new regression seeds `usize::MAX` next to `image4`, so it proves the
original panic trigger is fixed but does not exercise either adjacent-boundary
case.

`crates/rdocx/tests/regression_test.rs:99`

## Smells

None.

## Nitpicks

None.

## Not found

- No regression in the two sparse-name cases. The named gate still proves that
  allocation proceeds after the greatest ordinary suffix.
- No regression in malformed-name handling. Missing, signed, zero, unrelated,
  and nonnumeric suffixes remain ignored.
- No failure remains for the exact `usize::MAX` plus `image4` trigger from pass
  1. It allocates `image5`, preserves both existing parts, and does not create
  `image0`.
- All 15 tests in the `rdocx` regression binary pass.
