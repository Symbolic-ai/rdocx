# F-133, all, pass 4

**Reviewed**: working implementation diff from claim base `ddc031a`, 3 files
and 353 changed or newly added lines, with 2 modified tracked files and 1
approved untracked test file
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Earlier-finding re-evaluation

- Pass-1 D1 and pass-2 D1 are resolved at
  `crates/rdocx-py/tests/test_rendering_threads.py:274`. Equivalent separate
  uncached document sets feed serial and parallel modes. Every output from both
  modes must end in `%%EOF` before full Poppler page-count and extracted-text
  comparison. The recorded parallel truncation mutation fails this assertion.
- Pass-1 D2 is resolved at
  `crates/rdocx-py/tests/test_rendering_threads.py:110`. The lock handoff and
  60-second interpreter switch interval prove Python progress during
  `to_bytes`, single-page PNG, and all-page PNG without sleeps. The named tests
  at lines 238, 246, and 254 are sensitive to removing their detach calls, and
  the four-document median gate at line 263 protects `to_pdf`.
- Pass-2 D2 is resolved at
  `.claude/plans/F-133-design.md:79` and
  `crates/rdocx-py/tests/test_rendering_threads.py:69`. The approved plan routes
  the external comparison through differential-testing. The helper checks both
  tools for availability, invokes the resolved absolute executable paths, and
  requires the reviewed Poppler 26.01.0 version.
- Pass-3 D1 is resolved at
  `crates/rdocx-py/tests/test_rendering_threads.py:59`. Version output is parsed
  into tool-specific version lines and must equal the single complete expected
  line. The regression at line 230 rejects a `26.01.0-unreviewed` suffix for
  both `pdfinfo` and `pdftotext`, and the worker's substring-comparison mutation
  makes that regression fail.

## Not found

- Detachment boundary defects: all Python arguments become Rust scalars before
  detachment. Detached closures perform only Rust `Document` work. Python
  bytes, lists, and mapped errors are created after reattachment at
  `crates/rdocx-py/src/document.rs:60` through line 90.
- Mutable aliasing and PyO3 safety defects: mutable serialization retains the
  exclusive pyclass borrow while detached, rendering retains shared borrows,
  and independent documents proceed concurrently. No unsafe block, Python
  object, or interpreter token enters a detached closure.
- Poppler availability, version, completeness, and semantic defects: the test
  resolves and pins both executables before use, checks complete serial and
  parallel PDF terminators, and compares page count plus complete extracted
  text outside the timing intervals.
- GIL gate and timing stability defects: the three cooperating-thread gates
  use explicit synchronization. The PDF gate uses four independent nontrivial
  documents per mode, a separate font warmup, three alternating trials, median
  comparison, and a required multi-core host. It does not measure a reused
  document cache.
- Signature, result, and exception defects: page index and DPI convert before
  detachment, DPI defaults to 150, an out-of-range page returns `None`, and
  successful methods preserve bytes and list shapes. Layout failures map after
  reattachment to the exact public `LayoutError` class.
- Core, layout, HLD, and file-discipline defects: no core crate, layout crate,
  HLD file, dependency, feature flag, or Rust module changed. The approved
  external-oracle plan revision retains HLD impact none, and the only new file
  is the approved Python integration test.
- Artifact defects: no extension module, wheel, Python cache, or compiled
  Python file is present in the worker worktree.
- Focused gate failures: a dedicated isolated environment passed all 9
  rendering-thread tests and all 31 binding tests. Independent
  `cargo check -p rdocx-py --all-targets` passed. Worker evidence records green
  exact LayoutError, clippy, rdocx WASM, no-default-features layout, formatting,
  prose, skill-drift, diff, and unchanged 28-entry hash gates.
