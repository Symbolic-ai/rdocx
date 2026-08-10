# F-118, all, pass 3

**Reviewed**: full remediated working-tree diff against claim base `38aec895c0060ee3da0823bd2d70b6d900b76227`, 8 files, 1,892 changed lines. This includes 85 tracked changed lines and the 1,807 untracked lines in `crates/rpptx-chart/Cargo.toml` and `crates/rpptx-chart/src/lib.rs`. The worker-local `corpus` symlink is excluded.
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Not found

No findings in correctness, contract, panic safety, OOXML schema order,
namespace handling, unmodeled XML preservation, tests, or structure. The
pass-2 nested DrawingML defect is closed by comparing every foreign descendant
before and after delegated serialization. Nested foreign `x:r` and
`x:srgbClr` inputs are rejected, while unknown foreign subtrees that remain
byte-stable are accepted and preserved. The corpus gate now compares ordered
records containing parent path, schema boundary, sibling order, and exact
bytes. It no longer sorts away position evidence. The shared tail slot retains
producer order for opaque `showDLblsOverMax` and `extLst` siblings. All six
pass-1 defects and both pass-2 defects are resolved without expanding F-118
beyond its approved core boundary.
