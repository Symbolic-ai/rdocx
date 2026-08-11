# Current Sprint, S33

**Milestone**: M13 Bindings and tooling.

**Goal**: Validate the Python handle design against the settled rdocx API before
the same machinery is reused for rpptx. Build the shared path and revision
support, expose the documented rdocx surface through lazy handles, preserve
Python enum and unit behaviour, and prove rendering releases the GIL for real
parallel work.

## Spec references

- `docs/hld/03-architecture.md`, for the `oxml-py-support` ownership boundary,
  dependency direction, facade handle conventions, and non-consuming setter
  twins used by Python properties.
- `docs/hld/10-bindings-spec.md`, for path-based PyO3 handles, revision
  invalidation, lazy collections, tri-state properties, Python API shape,
  exception hierarchy, units, enums, and `allow_threads` rendering.
- `docs/hld/12-testing-strategy.md`, for Python differential coverage, binding
  test placement, and the required exclusions for workspace all-feature gates.
- `docs/hld/13-risks-and-open-questions.md`, for the index-path aliasing risk
  and its revision-counter, lazy-collection, and stale-error mitigations.
- `docs/hld/14-development-backlog.md`, for F-129 through F-133 dependencies
  and their named test gates.
- `docs/hld/15-build-and-toolchain.md`, for binding version alignment and the
  PyO3 link constraints that shape CI commands.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-129 | oxml-py-support | M | in-progress | codex |
| F-130 | rdocx-py core | L | in-progress | codex |
| F-132 | Python enums, units and exceptions | M | in-progress | codex |
| F-131 | rdocx-py formatting and tables | L | pending | - |
| F-133 | rdocx-py rendering with allow_threads | S | pending | - |

## Sequencing note

Rows are listed in dependency order, not by F-ID. F-129 establishes the shared
path, revision, error, and `Length` machinery. F-130 and F-132 can follow it
independently if their approved designs are conflict-free. F-131 and F-133 both
depend on the core document binding from F-130 and can then proceed in parallel
if they do not share binding or integration-test files. F-008 is already done,
so it does not block F-130.

## Definition of done for this sprint

- A stale content path raises `StaleElementError` and reports both the captured
  and current document revisions.
- `PyDocument`, lazy paragraph and run collections, and nested formatting and
  table handles mutate the settled facade without storing Rust borrows across
  Python calls.
- Tri-state formatting preserves `None` when inherited, Python enums retain
  their documented integer values, and one inch equals 914400 EMU.
- Rendering methods release the GIL through `allow_threads`, and four
  independent `to_pdf` calls complete faster in a thread pool than serially.
- Focused Rust and Python binding tests pass, the workspace gate uses the
  required binding-crate exclusions, and no existing document or rendering
  output regresses.
