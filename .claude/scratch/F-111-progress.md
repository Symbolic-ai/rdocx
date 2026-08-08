# F-111 progress notes

## Current state

The implementation and all planned tests are complete. The owning facade uses
staged package and media clones so every fallible path completes before the
presentation commits package, relationship, media, or shape-tree mutations.
Microscope pass 1 reports zero defects and zero smells. Completion checks are
green, and the worker is ready for its implementation commit and handoff.

## Changed areas

- `crates/rpptx-oxml/src/picture.rs`
- `crates/rpptx/src/lib.rs`
- `crates/rpptx/tests/integration.rs`
- `.claude/plans/F-111-design.md`
- `.claude/reviews/F-111-all-pass-1.md`
- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/06-presentationml-model.md`
- `docs/hld/14-development-backlog.md`

## Last green check

`cargo test -p rpptx --test integration added_picture_validates_and_opens_without_repair -- --ignored --exact`
passed on 2026-08-08 against Microsoft PowerPoint 16.104, bundle
16.104.25121423. The generated deck opened without repair.

The same final constructor passed the pinned python-pptx 1.0.2 comparison,
which reported `PICTURE` and 12,700 by 12,700 EMU for both implementations.
The non-fast verify gate passed formatting, workspace clippy, affected and
workspace tests, the hash harness with 28 matching entries, prose, skill drift,
no-default-features, wasm, and docs. Workspace tests used the canonical
checkout's read-only pinned corpus through `RDOCX_PPTX_CORPUS_DIR`.

## Blockers

None. The first workspace test run could not find the ignored corpus inside the
worker worktree. The rerun uses the canonical checkout's read-only pinned
corpus through `RDOCX_PPTX_CORPUS_DIR`.

Immediately before completion, the exact gate
`picture_without_explicit_size_uses_native_dimensions`, the python-pptx 1.0.2
helper, and the PowerPoint 16.104.25121423 helper all passed. The gate is absent
at claim Base `ac557a879e38a543a0064877ad20f7d21f35ceab`, which confirms the test
does not pass against the unimplemented claim state.

## Next action

Create the reviewed implementation commit, then write and validate the worker
handoff using that implementation SHA as `Head`.
