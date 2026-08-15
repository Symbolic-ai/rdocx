# F-X001, all aspects, pass 2

**Reviewed**: working-tree implementation, 3 files, 403 insertions and 30 deletions
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Not found

- Pass 1 D1. The text fixture now places a table between two paragraphs at
  `crates/rdocx-cli/tests/integration.rs:124` and requires the exact facade
  body-order representation. The command delegates directly to
  `Document::text()` at `crates/rdocx-cli/src/commands.rs:96`. The recorded
  grouped-traversal mutation fails this regression.
- Pass 1 D2. The visible-text fixture requests a system-specific face and
  computes bundled-font-only expected bytes at
  `crates/rdocx-cli/tests/integration.rs:311`. Both the selected-page and
  all-page command outputs must equal those bytes. The production branches use
  `render_page_to_png_deterministic` at
  `crates/rdocx-cli/src/commands.rs:299` and
  `crates/rdocx-cli/src/commands.rs:311`. The recorded ordinary-facade mutation
  of either branch fails the focused gate.
- Contract. The revised approved plan explicitly authorizes the bounded text
  order and deterministic render corrections, adds their named regressions,
  and lists exactly HLD 10, 12, and 14 for completion. No command, option,
  dependency, or public library API was added.
- Exact command surface and structure. One approved integration entrypoint
  invokes each of `inspect`, `text`, `convert`, `diff`, `replace`, `validate`,
  and `render` through the compiled binary. There is no second test binary,
  helper module, binary fixture, trait, generic, wrapper, or feature flag.
- Tests and sensitivity. The focused crate run passes 2 unit and 7 integration
  tests. Warning-denied all-target, all-feature clippy also passes. Recorded
  mutations cover a misspelled command, a false validate verdict, restored
  grouped text traversal, and each render branch returning to ordinary font
  discovery.
- Rendering behavior. Selected-page output preserves the zero-based public
  page option and one-based filename. The all-page loop advances until the
  cached deterministic layout returns no page, writes each page once, and
  reports the exact count. The visible text makes the font-path assertion
  materially sensitive rather than comparing blank rasters.
- Resources and temporary paths. Process ID plus an atomic counter isolates
  concurrent tests. Every output stays beneath its owning temporary workspace,
  and cleanup targets only that exact directory.
- Fixtures and outputs. DOCX and corrupt-package inputs are constructed in
  code. Inspect binds schema 1, convert checks its shared default path and four
  formats, diff binds the non-verdict exit status, replace reopens both output
  and unchanged input, and validate checks a real dangling internal
  relationship.
- Panics and OOXML. Production changes use existing total facade paths and
  propagate render errors. They do not change parsing, serialization, schema
  child order, namespace handling, or unmodelled XML preservation.
