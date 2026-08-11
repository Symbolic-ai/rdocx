# F-129, oxml-py-support

**Status**: approved
**Sprint**: S33
**Size**: M
**Depends on**: none

## Problem

The workspace has no shared Python binding support crate or PyO3 dependency
(`Cargo.toml:3`, `Cargo.toml:46`). The canonical `Length` remains a private-EMU
Rust value whose floating constructors deliberately truncate toward zero
(`crates/oxml-core/src/length.rs:5`, `crates/oxml-core/src/length.rs:11`), and
there is no reusable content path, revision counter, or stale-handle error.
Without those pieces, a Python handle cannot safely re-resolve a paragraph or
run after later document mutations.

## Spec reference

- `docs/hld/03-architecture.md`, "Three families, one workspace" and "The dependency rule".
- `docs/hld/10-bindings-spec.md`, "The chosen design", "The invalidation problem, handled loudly", "Python API shape", and "Packaging".
- `docs/hld/13-risks-and-open-questions.md`, "R9, index-path aliasing in the Python bindings".
- `docs/hld/14-development-backlog.md`, "F-129, oxml-py-support".

## Approach

Create the story-authorized, unpublished `crates/oxml-py-support` workspace
crate with one `src/lib.rs`. Define concrete `PathSeg`, `ContentPath`,
`RevisionCounter`, and stale-domain error types. A path captures the current
revision, preserves nested segment order, and rejects every operation when its
captured revision differs from the document revision. Successful structural
mutations bump the counter, while failed or value-only operations do not.

Keep the crate independent of `rdocx-*` and `rpptx-*`. Delegate every unit
conversion to `oxml_core::Length`, preserving its pinned truncation. Under the
recommended ownership resolution, F-129 supplies Rust conversion and stale
classification support, while F-132 owns the concrete pure-Python
`Length(int)` class and package exception hierarchy required by `abi3-py39`.
Only Word path variants used by S33 are added now. Presentation variants wait
for the existing F-136 consumer.

## Rejected alternatives

- Hold Rust borrows in Python objects. A `#[pyclass]` must be static and later
  vector reallocations would invalidate the borrow.
- Add `Rc<RefCell<_>>` or `Arc<Mutex<_>>` to the core facades. That would
  rewrite the Rust API and weaken the existing `Send + Sync` contract.
- Add arena IDs, snapshots, or an owned mirror API. They exceed the approved
  v0.1 revision-counter design.
- Duplicate or round unit conversions. The existing truncation is intentional
  and pinned.
- Implement `Length(int)` as a native-base PyO3 class. Native-base inheritance
  is unavailable for the required Python 3.9 to 3.11 limited ABI.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit, gate | `stale_path_reports_both_revisions` | A stale path produces the named stale error with captured and current revisions |
| unit | `matching_revision_accepts_content_path` | A current path resolves without error |
| unit | `revision_counter_bumps_after_successful_structure_change` | New paths capture the incremented revision |
| unit | `word_path_segments_preserve_nested_order` | Body, row, cell, paragraph, and run segments remain ordered |
| regression | `python_length_helpers_preserve_rust_truncation` | Positive and negative fractional conversions truncate toward zero |

The backlog test gate is that a stale path raises the named error with both
revisions in the message. Focused commands are
`cargo check -p oxml-py-support --all-targets` and
`cargo test -p oxml-py-support`.

## HLD impact

- `docs/hld/10-bindings-spec.md`
- `docs/hld/14-development-backlog.md`

These files must make the viable F-129 and F-132 ownership split explicit and
defer presentation-only path variants to their named consumer.

## Risk routing

- Unit conversion. Preserve `as i64` truncation and run positive and negative
  conversion regressions.
- Crate dependency graph. Run
  `cargo tree -p oxml-py-support --edges normal` and prove no dependency on a
  format-specific crate.
- WASM or PyO3 bindings. Keep `extension-module` off in the support crate,
  retain the workspace binding exclusions, and run
  `cargo check --target wasm32-unknown-unknown -p rdocx-wasm`.
- New crate, module, or file. Obtain explicit approval for the crate manifest
  and its single source file before implementation.
- Version strings. Inspect the root manifest, new manifest, lockfile, and
  release allowlists. Keep the crate unpublished and create no tag.

## Hash harness

Expected unchanged. The isolated binding support does not participate in
sample generation, document serialization, layout, or rendering.

## Implementation checklist

- [ ] Add the unpublished workspace crate and dependencies.
- [ ] Implement Word path segments, content paths, and the revision counter.
- [ ] Implement the stale-domain error and concrete revision validation.
- [ ] Delegate conversion helpers to the canonical shared `Length`.
- [ ] Add inline unit and regression tests.
- [ ] Run focused checks and every risk rider.

## Open questions

None. The new crate and minimal files, the F-129 and F-132 ownership split, and
deferral of presentation-only path variants were approved together.
