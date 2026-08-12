# F-130, all, pass 2

**Reviewed**: working implementation diff from claim base `ccbeb7a`, 18 files,
956 changed lines, with 940 additions and 16 deletions
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Pass 1 findings re-evaluated

- D1 is resolved. The constructor now accepts an optional `PathBuf` at
  `crates/rdocx-py/src/document.rs:32`, and the regression covers both the empty
  constructor and an input path at `crates/rdocx-py/tests/test_core.py:73`. A
  freshly built wheel also opened the same document from both a string and a
  `PathLike` value.
- D2 is resolved. The run getter resolves through the immutable document facade
  at `crates/rdocx-py/src/run.rs:74`, and collection length does the same at
  `crates/rdocx-py/src/run.rs:129`. Mutable paragraph lookup remains confined to
  the run setter at `crates/rdocx-py/src/run.rs:87` and structural `add_run` at
  `crates/rdocx-py/src/paragraph.rs:80`. The direct immutable lookup is at
  `crates/rdocx/src/document.rs:383`, its total run accessors are at
  `crates/rdocx/src/paragraph.rs:547`, and the real cache reuse regression is at
  `crates/rdocx/src/document.rs:3156`.
- D3 is resolved by explicit serialization rather than by expanding F-130. The
  approved plan limits this story to its gate-only stale bridge and schedules
  F-132 to replace it at `.claude/plans/F-130-design.md:54`. The approved F-132
  plan starts after the mixed package skeleton is integrated and owns the final
  hierarchy and complete mappings at `.claude/plans/F-132-design.md:27`. The
  temporary class at `crates/rdocx-py/python/rdocx/__init__.py:4` and mapper at
  `crates/rdocx-py/src/lib.rs:17` therefore match the revised ownership split.

## Not found

- Correctness and revision handling: successful paragraph addition and removal
  bump the counter at `crates/rdocx-py/src/document.rs:73` and
  `crates/rdocx-py/src/document.rs:88`. Successful run addition bumps it at
  `crates/rdocx-py/src/paragraph.rs:80`. Failed removal and value-only run text
  mutation leave live handles current, as exercised at
  `crates/rdocx-py/tests/test_core.py:48` and
  `crates/rdocx-py/tests/test_core.py:59`.
- PyO3 ownership and borrowing: document ownership remains in `PyDocument` at
  `crates/rdocx-py/src/document.rs:11`. Paragraph and run handles retain only a
  `Py<PyDocument>` and owned path at `crates/rdocx-py/src/paragraph.rs:34` and
  `crates/rdocx-py/src/run.rs:47`. No facade borrow escapes a method call, and no
  unsafe ownership conversion was found.
- Lazy indexing, slicing, and iteration: paragraph collection operations are at
  `crates/rdocx-py/src/paragraph.rs:125`, and run collection operations are at
  `crates/rdocx-py/src/run.rs:153`. Positive and negative indexes, forward and
  reverse slices, zero-step rejection, and both iterators behaved correctly
  against the current wheel.
- Stale behavior: every paragraph, run, and run-collection operation validates
  its captured revision at `crates/rdocx-py/src/paragraph.rs:45`,
  `crates/rdocx-py/src/run.rs:58`, and `crates/rdocx-py/src/run.rs:117`. The
  named Python error retained both revisions and its concrete recovery hint.
- Packaging and dependency direction: the crate is unpublished, keeps
  `extension-module` off by default, and carries the workspace release family
  at `crates/rdocx-py/Cargo.toml:1`. The mixed package configuration is at
  `crates/rdocx-py/pyproject.toml:12`. The dependency tree contains only the
  approved inward workspace edges to `rdocx` and `oxml-py-support`.
- Release metadata: the new workspace member and pin are at `Cargo.toml:23` and
  `Cargo.toml:69`. The regression requires ten workspace-version packages at
  `scripts/test_sprint_workflow.py:363`, while the HLD retains the exact seven
  published stable packages at `docs/hld/15-build-and-toolchain.md:183`.
- HLD scope: the additive immutable facade access is recorded at
  `docs/hld/03-architecture.md:166` and the binding resolution rule is recorded
  at `docs/hld/10-bindings-spec.md:39`. All HLD edits are limited to the three
  approved files.
- Contract and scope: no rendering API from F-133, formatting or table surface
  from F-131, final exception hierarchy from F-132, new trait, generic, dynamic
  dispatch, or unapproved file was found. No wheel, extension binary, or cache
  artifact is part of the working diff.
- Panics and OOXML: no reachable panic, unchecked Python index, invalid slice
  loop, parser change, serializer ordering change, namespace change, or raw XML
  preservation change was found.
- Tests and gates: `cargo check -p rdocx-py --all-targets`, binding clippy, all
  63 rdocx unit tests, 74 integration tests, 17 regression tests, and one doc
  test passed. A current `cp39-abi3` wheel built and all five focused Python
  tests passed. The no-default layout gate passed 62 unit and two doc tests. The
  rdocx WASM check, ten-package release-family regression, rustfmt, diff check,
  dependency inspection, and all 28 hash entries also passed.
