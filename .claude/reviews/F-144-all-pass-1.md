# F-144, all, pass 1

**Reviewed**: uncommitted `work/f-144-codex` implementation, 14 files and 904
changed lines, including the four approved new CLI crate paths
**Verdict**: 3 defects, 0 smells, 0 nitpicks

## Defects

### D1, the required corpus gate fails in the clean CI test job

`.github/workflows/ci.yml:26`

The workspace test job runs the new `rpptx-cli` integration binary without
fetching the ignored corpus first. A clean checkout therefore reaches the
unconditional corpus assertion at `crates/rpptx-cli/tests/integration.rs:88`
and fails before validating any of the 50 decks. The only corpus fetch remains
in the later presentation-fidelity job, whose filesystem is isolated from the
test job. Fetch and verify the pinned corpus in every job that runs this gate,
then retain the test's no-skip behavior. A fresh review run reproduced the
failure with all 50 manifest entries missing.

### D2, positive finite DPI still permits unsafe raster allocation

`crates/rpptx-cli/src/commands.rs:245`

The CLI accepts every positive finite `f64` as DPI. A value such as 100,000 is
then multiplied by the slide dimensions and passed to `tiny_skia::Pixmap`,
which allocates a zero-filled RGBA vector for the complete raster. A standard
slide requests hundreds of billions of bytes, so user input can abort or hang
the process instead of returning a bounded command error. Validate the derived
pixel dimensions or total pixel budget before either convert or render reaches
the rasterizer, and cover the rejected boundary.

### D3, plain inspect omits core metadata

`crates/rpptx-cli/src/commands.rs:52`

The approved inspect contract reports core metadata with or without `--json`,
but the plain branch prints only file, counts, size, and slide details. A deck
with title, creator, subject, or other core properties therefore loses that
required inspect output unless the caller requests JSON. Print the same core
metadata fields in the plain path and add a command regression with populated
properties.

## Smells

None.

## Nitpicks

None.

## Not found

- **Public replacement semantics**: literal matches are snapshotted before
  mutation, processed in reverse byte order, and never recurse into replacement
  text. Same-run and cross-run replacement retains the first run's formatting,
  keeps later suffix formatting, and leaves breaks, fields, and alternate
  content as boundaries. Nested groups and table cells are traversed.
- **Package preservation**: replacement mutates typed run text in place and
  saves through the facade staging path. The round-trip regression preserves
  both run property sets and an opaque package part.
- **CLI paths**: exactly the seven approved commands exist. Shared range,
  output-path, and JSON helpers are reused, slide selection is one-based,
  multi-slide PNG suffixes are one-based, and thumbnail and outline remain
  absent.
- **Correctness and panics**: apart from D2, no indexing, LCS backtracking,
  range conversion, rendering selection, replacement offset, or validation exit
  defect was found. Internal `expect` and `unreachable` sites are protected by
  immediately established invariants.
- **Tests**: four of the five CLI integration tests passed independently. The
  validation test failed only at the clean-worktree corpus condition in D1.
  The focused facade matrix passed for same-run, group, and table replacement,
  while the CLI round-trip proves a cross-run match.
- **OOXML**: no raw PresentationML is read or mutated by the CLI. Typed edits
  preserve schema order and leave selected alternate-content branches
  untouched.
- **Structure and dependencies**: the implementation uses the four explicitly
  approved new paths, one command module, one test entrypoint, no command trait,
  no forwarding library, and only inward facade and shared-helper edges.
- **Publication and harness**: version pins, release family, workflow allowlist,
  and 21-package dry-run inventory agree. No release action is performed, and
  no sample generator, rendering default, or hash baseline changes.
- **Hygiene**: diff hygiene, prose validation, and generated-skill drift checks
  passed during review.
