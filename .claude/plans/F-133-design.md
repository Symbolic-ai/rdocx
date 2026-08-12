# F-133, rdocx-py rendering with allow_threads

**Status**: completed
**Sprint**: S33
**Size**: S
**Depends on**: F-130

## Problem

The Rust document already owns thread-safe normal and deterministic layout
caches (`crates/rdocx/src/document.rs:34`) and exposes PDF, PNG, and all-page
rendering (`crates/rdocx/src/document.rs:2169`,
`crates/rdocx/src/document.rs:2292`). No Python binding exists to detach from
the interpreter while that Rust-only work runs. Holding the interpreter lock
would serialize independent document renders and fail the concurrency gate.

## Spec reference

- `docs/hld/08-rendering-spec.md`, "Facade cache and invalidation contract".
- `docs/hld/10-bindings-spec.md`, "Threading" and "Python API shape".
- `docs/hld/14-development-backlog.md`, "F-133, rdocx-py rendering with allow_threads".

## Approach

Extend F-130's `PyDocument` with `to_pdf`, `to_bytes`,
`render_page_to_png`, and `render_all_pages`. Convert Python arguments before
detaching, run only Rust-owned document work inside PyO3's current detach API,
then reattach to build Python bytes or lists and map errors. This preserves the
specified `allow_threads` behavior without using a deprecated spelling.

Keep the Rust `Document`, caches, and rendering backends unchanged. The timing
gate uses four independent, nontrivial uncached documents. It warms global font
discovery with a sacrificial document, compares repeated serial and parallel
medians, and requires the parallel median to be lower on the sprint's supported
multi-core test environment. Outside timed intervals, validate every complete
serial and parallel PDF through pinned Poppler 26.01.0 `pdfinfo` and
`pdftotext`, comparing page counts and full extracted text. The test fails
clearly unless both tools exist and each reports the reviewed exact version.

## Rejected alternatives

- Render while attached to the interpreter. Independent Python threads would
  remain serialized.
- Spawn ad hoc Rust threads or clone document models. PyO3 detachment already
  permits the caller's thread pool to run.
- Time one warmed document four times. That measures cache locking rather than
  independent layout parallelism.
- Use sleeps or test hooks. They do not prove real rendering concurrency.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| integration, gate | `four_concurrent_to_pdf_calls_are_faster_than_serial` | Equivalent independent uncached renders have a lower parallel median, complete structure, and equal full Poppler semantics |
| integration | `to_pdf_returns_pdf_bytes` | The detached call returns a PDF signature |
| integration | `render_methods_return_png_bytes_and_page_lists` | Page and all-page results keep their Python shapes and signatures |
| integration | `render_errors_reacquire_and_map_cleanly` | Rust errors become the approved Python exceptions after reattachment |
| integration | `poppler_pdf_oracle_is_available_at_reviewed_version` | Both required tools exist and report Poppler 26.01.0 exactly |
| regression | existing `Document: Send + Sync` compile gate | Core thread safety remains intact |

The timing test is the verbatim backlog gate. Focused checks use the
non-extension binding check and the approved maturin plus rendering-thread
pytest command.

## HLD impact

None. The threading, cache, and Python rendering contracts already describe
the intended behavior.

## Risk routing

- Layout and rendering. Record no system-font baseline. Functional byte checks
  use deterministic rendering where applicable, and the timing test records
  elapsed time only.
- WASM or PyO3 bindings. Retain workspace binding exclusions and run the
  existing rdocx WASM target check.
- New file. Obtain explicit approval for one dedicated Python rendering-thread
  integration test file. Add no forwarding-only Rust module.
- External oracle comparison. Follow `.claude/skills/differential-testing.md`.
  Require `pdfinfo` and `pdftotext`, assert that both report Poppler 26.01.0,
  then compare complete serial and parallel PDF semantics outside timing.

## Hash harness

Expected unchanged. The new methods wrap existing read-only render calls and
do not change core layout or output.

## Implementation checklist

- [x] Add detached PDF, bytes, page, and all-page methods to `PyDocument`.
- [x] Reattach only for Python result construction and error mapping.
- [x] Add functional PDF and PNG integration tests.
- [x] Add the independent-document serial versus parallel timing gate.
- [x] Run focused checks and every risk rider.

## Open questions

None. The full HLD-named detached surface, multi-core timing policy, and
dedicated Python test file were approved together.
