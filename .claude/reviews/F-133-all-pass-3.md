# F-133, all, pass 3

**Reviewed**: working implementation diff from claim base `ddc031a`, 3 files
and 338 changed or newly added lines, with 2 modified tracked files and 1
approved untracked test file
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, The Poppler version assertion is a substring check
`crates/rdocx-py/tests/test_rendering_threads.py:71`

The revised plan requires each oracle executable to report the exact reviewed
Poppler 26.01.0 version, but `_poppler_tools` accepts the expected text anywhere
in combined output. An executable reporting `pdfinfo version
26.01.0-unreviewed` or the analogous `pdftotext` string therefore passes. An
independent mocked-executable probe reproduced that acceptance for both tools.
Parse the version line and compare it for equality to the expected complete
line so the test implements the approved exact pin.

## Smells

None.

## Nitpicks

None.

## Earlier-finding re-evaluation

- Pass-1 D1 and pass-2 D1 are resolved at
  `crates/rdocx-py/tests/test_rendering_threads.py:260`. Serial and parallel
  batches use identical seed sets with separate uncached documents. Every
  output from both modes must end in `%%EOF` at lines 274 through 277 before
  pinned-tool semantic comparison. The worker's six-byte truncation mutation
  made the combined assertion fail.
- Pass-1 D2 is resolved at
  `crates/rdocx-py/tests/test_rendering_threads.py:103`. The explicit lock
  handoff and 60-second switch interval prove interpreter progress without
  sleeps for `to_bytes`, single-page PNG, and all-page PNG. The corresponding
  named tests begin at lines 223, 231, and 239, and the worker's detach-removal
  mutations made all three fail. The `to_pdf` path remains protected by the
  four-document median gate at line 248.
- Pass-2 D2 is resolved except for D1 above. The approved plan now routes the
  external comparison through differential-testing at
  `.claude/plans/F-133-design.md:79`. The test checks availability with
  `shutil.which`, retains each resolved absolute executable path, and fails
  clearly for a missing command or a wholly different version.

## Not found

- Detachment boundary defects: all Python arguments are converted to Rust
  scalars before detachment. Detached closures perform only Rust `Document`
  operations, and Python bytes, lists, and mapped errors are created after
  reattachment at `crates/rdocx-py/src/document.rs:60` through line 90.
- Mutable aliasing or PyO3 safety defects: `to_bytes` retains an exclusive
  pyclass borrow during mutable serialization. Rendering retains shared
  borrows. No unsafe block, Python object, or interpreter token enters a
  detached closure.
- PDF completeness and semantic-comparison defects beyond D1: validation runs
  outside timing and compares the page count and complete extracted text for
  each equivalent document. Absolute paths prevent a later `PATH` change from
  swapping the checked tools before semantic extraction.
- Timing stability defects: four independent nontrivial documents are freshly
  constructed for each mode and each trial. A separate sacrificial render
  warms font discovery, order alternates, three samples feed each median, and
  the strict parallel-less-than-serial assertion remains guarded by a required
  multi-core host.
- Signature, result, and error defects: page index and DPI convert before
  detachment, DPI defaults to 150, out-of-range page access returns `None`, and
  successful calls preserve their bytes and list shapes. Layout errors map
  after reattachment to the exact public `LayoutError` class.
- Core, layout, HLD, and file-discipline defects: no core crate, layout crate,
  HLD file, dependency, feature flag, or Rust module changed. The narrow plan
  revision records the external oracle, HLD impact remains none, and the only
  new file is the approved Python integration test.
- Artifact defects: no extension module, wheel, Python cache, or compiled
  Python file is present in the worker worktree.
- Focused gate failures: a dedicated isolated environment passed all 8
  rendering-thread tests and all 30 binding tests. Independent
  `cargo check -p rdocx-py --all-targets` and the exact LayoutError Rust unit
  test passed. Worker evidence records green clippy, rdocx WASM,
  no-default-features layout, formatting, prose, skill-drift, diff, and
  unchanged 28-entry hash gates.
