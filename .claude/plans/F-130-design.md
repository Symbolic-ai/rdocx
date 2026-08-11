# F-130, rdocx-py core

**Status**: completed
**Sprint**: S33
**Size**: L
**Depends on**: F-129, F-008

## Problem

No `rdocx-py` crate or Python package exists in the workspace
(`Cargo.toml:3`). The Rust `Document` owns its package, model, and thread-safe
layout caches (`crates/rdocx/src/document.rs:34`), while its paragraph access
allocates a `Vec` (`crates/rdocx/src/document.rs:374`) and its mutable paragraph
API exposes no existing-run lookup (`crates/rdocx/src/paragraph.rs:134`). A
Python object therefore cannot retain a Rust borrow or safely resolve a held
run after document mutation.

## Spec reference

- `docs/hld/03-architecture.md`, "Three families, one workspace" and "Facade conventions".
- `docs/hld/10-bindings-spec.md`, "The PyO3 lifetime problem", "The chosen design", "The invalidation problem, handled loudly", "Two supporting decisions", and "Packaging".
- `docs/hld/13-risks-and-open-questions.md`, "R9, index-path aliasing in the Python bindings".
- `docs/hld/14-development-backlog.md`, "F-130, rdocx-py core".

## Approach

Create the unpublished mixed-layout `rdocx-py` crate. `PyDocument` owns an
`rdocx::Document` and a revision counter. `PyParagraph` and `PyRun` own only a
`Py<PyDocument>` plus an F-129 `ContentPath`. Lazy collection classes hold the
same lightweight state and implement length, integer and negative indexing,
slices, and iteration without storing facade borrows.

Give the new unpublished workspace member the existing `workspace` release
family metadata. Update the release-safety regression from nine to ten
workspace-version packages and keep the exact seven-package published stable
family unchanged.

Expose the core constructor with an optional input path so both `Document()`
and `Document("in.docx")` work, plus the explicit open/from-bytes path and
save/to-bytes path,
paragraph collection, add/remove content, paragraph text, run collection,
add-run, and run text. Check the captured revision before every handle
operation. Increment only after successful operations that add, remove, or
reorder path-addressed content.

Add the smallest facade accessors required for direct re-resolution:
`Document::paragraph`, `Paragraph::run_count`, `Paragraph::run`, and
`Paragraph::run_mut`, plus immutable `ParagraphRef::run_count` and
`ParagraphRef::run`. Read-only run resolution uses only the immutable facade
and cannot clear layout caches. Within run handle and collection resolution,
mutable resolution is reserved for the run setter. Structural mutations such
as `add_run` retain their required mutable access. Keep every existing method.

F-130 maps only the shared stale-domain error through a minimal temporary
named `StaleElementError` bridge required by its gate. Serialized F-132
replaces that bridge with the final pure-Python exception hierarchy and
completes all package error mapping. F-130 and F-132 must not run in parallel.

## Rejected alternatives

- Store `Paragraph<'a>` or `Run<'a>` in a pyclass. Their lifetimes cannot be
  static and later structural mutations can reallocate their vectors.
- Call `Document::paragraphs()` for every property. It allocates snapshots and
  contradicts the lazy-collection contract.
- Reach through private OOXML fields from the binding. The facade remains the
  sole owner of package mutation.
- Add a generic resolver trait or a mirrored document model. Neither has two
  present implementations and both increase indirection.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| integration, gate | `stale_paragraph_after_structural_removal_raises_named_error` | Holding `doc.paragraphs[3]` across `remove_content(1)` raises `StaleElementError` with both revisions |
| integration | `lazy_collections_support_index_slice_and_iteration` | Paragraph and run collections are lazy and Python-index compatible |
| integration | `failed_removal_does_not_stale_live_handles` | An out-of-range removal leaves the revision unchanged |
| round-trip | `core_text_mutations_survive_bytes_round_trip` | Added paragraphs and runs survive save and reopen |
| integration | `constructor_accepts_an_optional_input_path` | `Document()` creates a document and `Document(path)` opens one |
| regression | `immutable_run_accessors_are_total` | Read-only run count and lookup use `ParagraphRef` and return `None` rather than panic |
| regression | `direct_facade_accessors_are_total` | New Rust accessors return `None` rather than panic |

The first integration test is the verbatim backlog gate. Focused commands are
`cargo check -p rdocx-py --all-targets`, `cargo test -p rdocx`, a non-extension
module Rust check for `rdocx-py`, and the approved maturin plus pytest command.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/15-build-and-toolchain.md`

The current "zero change to the Rust API" statement must describe the small
additive index accessors that make path re-resolution possible. The build spec
must count the new unpublished workspace-version package without changing the
published stable family.

## Risk routing

- Crate dependency graph. Run `cargo tree -p rdocx-py --edges normal` and
  verify the binding depends inward on `rdocx` and `oxml-py-support` only.
- WASM or PyO3 bindings. Keep `extension-module` off by default, retain the
  workspace binding exclusions, and run the existing rdocx WASM target check.
- New feature flag. The `extension-module` feature has the current maturin
  build as its named consumer. Run the no-default layout gate.
- New crate, module, or file. Obtain explicit approval for the mixed-layout
  crate, class modules, package initializer, and one Python core test file.
- Public API of published `rdocx`. State that the direct accessors are additive,
  run the workspace publication dry run, and enforce the archive size limit.
- Release scripting and version strings. Add the existing `workspace` release
  family metadata to `crates/rdocx-py/Cargo.toml`, update
  `scripts/test_sprint_workflow.py` from nine to ten workspace-version
  packages, and keep the seven-package published stable allowlist unchanged.

## Hash harness

Expected unchanged. The binding is not used by sample generation, and the new
facade accessors do not alter mutation or serialization behavior.

## Implementation checklist

- [x] Create the mixed-layout `rdocx-py` crate and package skeleton.
- [x] Add direct paragraph and run facade accessors with Rust regressions.
- [x] Implement owned document, path-only handles, and lazy collections.
- [x] Implement successful structural revision bumps and stale validation.
- [x] Provide only the temporary named stale-error bridge required by F-130.
- [x] Accept an optional input path in the Python `Document` constructor.
- [x] Resolve every read-only run operation through immutable facade access.
- [x] Run focused checks and every risk rider.

## Open questions

None. The new mixed-layout crate and minimal files, additive facade accessors,
path semantics, and successful-structural-mutation revision rule were approved
together.
