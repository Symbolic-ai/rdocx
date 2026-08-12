# F-133, all, pass 2

**Reviewed**: working implementation diff from claim base `ddc031a`, 3 files
and 296 changed or newly added lines, with 2 modified tracked files and 1
approved untracked test file
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, The parallel completeness check still accepts truncated PDFs
`crates/rdocx-py/tests/test_rendering_threads.py:246`
`crates/rdocx-py/tests/test_rendering_threads.py:247`

The remediated gate checks the `%%EOF` terminator only on serial outputs.
Parallel outputs pass solely through `pdfinfo` and `pdftotext`. Poppler is
deliberately tolerant of repairable truncation. An independent probe removed
the final 6 bytes and then the final 20 bytes from a generated PDF, and
`_pdf_semantics` accepted both with the original page count and full extracted
text. A parallel render can therefore lose its terminator and still compare
equal, leaving the original corruption case partly unprotected. Apply the
completeness assertion to both output sets before semantic comparison.

### D2, The new Poppler oracle is neither routed nor version-pinned
`crates/rdocx-py/tests/test_rendering_threads.py:61`
`.claude/plans/F-133-design.md:66`

The pass-1 remediation introduced `pdfinfo` and `pdftotext` as external PDF
oracles, but the test invokes whichever binaries happen to be on `PATH`
without checking their versions. The plan still has no external-oracle risk
row and says the timing test records elapsed time only. The local binaries are
both Poppler 26.01.0 today, but a missing installation or package-manager
upgrade changes the gate without a reviewed repository change. Revise the
risk routing to declare the test dependency, fail clearly when either command
is absent, and assert the repository's reviewed Poppler version before using
the tools.

## Smells

None.

## Nitpicks

None.

## Pass-1 re-evaluation

- D1 is partly resolved at
  `crates/rdocx-py/tests/test_rendering_threads.py:232`. Serial and parallel
  batches now use equivalent fresh documents, all semantic checks run outside
  the timed intervals, and page count plus full extracted text are compared.
  The remaining parallel completeness hole is D1 above.
- D2 is resolved at
  `crates/rdocx-py/tests/test_rendering_threads.py:80`. The lock-handoff helper
  proves interpreter progress without sleeps, and named tests cover
  `to_bytes`, single-page PNG, and all-page PNG at lines 196, 204, and 212.
  Removing all three detach calls made each corresponding test fail in the
  worker's recorded mutation check. The `to_pdf` median gate remains sensitive
  to retaining its own detach call.

## Not found

- Detachment boundary defects: all Python arguments become Rust scalars before
  detachment. Detached closures touch only the Rust `Document`, and Python
  bytes, lists, and mapped errors are created after reattachment at
  `crates/rdocx-py/src/document.rs:60` through line 90.
- Mutable aliasing and PyO3 safety defects: detached `to_bytes` retains the
  exclusive pyclass borrow. Rendering retains shared borrows, and independent
  documents can proceed concurrently. No unsafe block, Python object access,
  or interpreter token crosses into a detached closure.
- GIL-progress synchronization defects: the worker announces readiness before
  blocking on a lock held by the caller. A 60-second interpreter switch
  interval prevents an ordinary bytecode timeslice from satisfying the gate,
  and cleanup restores the prior interval and joins the worker. Three
  independent review runs of all seven focused tests passed in 15.94, 14.90,
  and 14.82 seconds.
- Timing stability defects: each sample uses four independent, nontrivial,
  uncached documents. A sacrificial render warms global font discovery, trial
  order alternates, three samples feed each median, and the gate remains a
  strict parallel-less-than-serial comparison on a required multi-core host.
- Signature and result defects: page index and DPI convert before detachment,
  DPI defaults to 150, out-of-range page access returns `None`, and successful
  calls return bytes or a list of bytes as specified.
- Exception defects: layout errors are mapped after reattachment to the public
  `LayoutError`. The malformed-font installed-wheel path and the exact-class
  Rust classifier remain covered.
- Core, layout, HLD, and file-discipline defects beyond D2: no core crate,
  layout crate, HLD file, dependency, feature flag, or Rust module changed.
  The implementation uses only the approved Python test file and retains the
  approved HLD impact of none.
- Artifact defects: no extension module, wheel, Python cache, or compiled
  Python file is present in the worker worktree.
- Focused gate failures: a fresh abi3 wheel passed all 7 rendering-thread tests
  and all 29 binding tests. Independent `cargo check -p rdocx-py --all-targets`
  passed. Worker evidence records green clippy, rdocx WASM,
  no-default-features layout, formatting, prose, skill-drift, diff, and
  unchanged 28-entry hash gates.
