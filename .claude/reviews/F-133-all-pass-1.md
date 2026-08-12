# F-133, all, pass 1

**Reviewed**: working implementation diff from claim base `ddc031a`, 3 files
and 201 changed or newly added lines, with 2 modified tracked files and 1
approved untracked test file
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, The concurrency gate does not compare equivalent render results
`crates/rdocx-py/tests/test_rendering_threads.py:147`
`crates/rdocx-py/tests/test_rendering_threads.py:49`

Each serial batch and parallel batch uses different document seeds, and the
timing helper validates only the five-byte PDF signature. The gate therefore
cannot show that parallel execution performed the same valid work as serial
execution. A concurrency defect that corrupts or truncates a PDF after its
header could still satisfy every assertion. Construct equivalent independent
uncached document sets for the two modes and compare the complete serial and
parallel outputs outside the timed intervals.

### D2, Three detached methods have no GIL-release-sensitive regression
`crates/rdocx-py/src/document.rs:60`
`crates/rdocx-py/src/document.rs:73`
`crates/rdocx-py/src/document.rs:85`
`crates/rdocx-py/tests/test_rendering_threads.py:136`

The only concurrency-sensitive test calls `to_pdf`. The tests for `to_bytes`,
`render_page_to_png`, and `render_all_pages` check result shapes while running
on one Python thread. Removing `detach` from any of those three methods leaves
the complete test file green, despite violating the approved four-method
detachment contract. Add a reliable release-sensitive gate for each named
method. It need not use the expensive four-document timing policy when a
deterministic cooperating-thread check proves interpreter progress.

## Smells

None.

## Nitpicks

None.

## Not found

- Detachment boundary defects in the implementation: `to_pdf`, `to_bytes`,
  single-page PNG, and all-page PNG convert Python arguments before
  detachment. Their detached closures perform only Rust document operations,
  and Python bytes, lists, and mapped exceptions are created after
  reattachment at `crates/rdocx-py/src/document.rs:60` through line 90.
- Mutable aliasing or GIL misuse: detached `to_bytes` retains PyO3's exclusive
  mutable pyclass borrow, while read-only rendering retains shared borrows.
  PyO3's borrow checker prevents conflicting access to the same document, and
  independent documents can proceed concurrently. No unsafe block or Python
  object access occurs inside a detached closure.
- Signature and result-shape defects: `render_page_to_png` takes a zero-based
  page index and defaults to 150 DPI, while `render_all_pages` defaults to 150
  DPI. An out-of-range page maps to Python `None`, and successful results map
  to Python bytes or a list of bytes.
- Exception classification defects: layout failures are mapped only after
  reattachment. The installed-wheel malformed-font test raises the public
  `LayoutError`, and the exact-class Rust classifier at
  `crates/rdocx-py/src/lib.rs:78` passed independently.
- Timing-policy defects beyond D1: the gate uses four independent, nontrivial,
  uncached documents per mode, a sacrificial font warmup, three samples,
  alternating order, and median comparison. It requires a supported
  multi-core environment without skipping or weakening the assertion.
- Cache-timing defects: every measured document is newly constructed and used
  once. The sacrificial document is separate, so the comparison does not time
  repeated access to one document's populated layout cache.
- Core, layout, scope, and HLD defects: no core crate, layout crate, HLD file,
  dependency, feature flag, module, or public Rust API changed. The approved
  HLD impact remains none, and the only new file is the explicitly approved
  Python integration test.
- Artifact defects: no extension module, wheel, Python cache, or compiled
  Python file is present in the worker worktree.
- Focused gate failures: independent `cargo check -p rdocx-py --all-targets`
  passed. A fresh abi3 wheel passed the 4 focused rendering tests and all 26
  binding tests from an isolated temporary directory. The exact LayoutError
  Rust unit test also passed. Worker evidence records green workspace clippy,
  rdocx WASM, no-default-features layout, formatting, prose, skill-drift, diff,
  and unchanged 28-entry hash gates.
