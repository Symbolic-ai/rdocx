# F-X002, README example correctness

**Status**: completed
**Sprint**: S36
**Size**: S
**Depends on**: none

## Problem

The root README contains six Rust examples, but they are not compiled. Its read
example calls nonexistent `TableRef::rows()` and `TableRowRef::cells()` methods.
Users therefore encounter an API error in the first documentation path while
the normal Rust documentation build remains green.

## Spec reference

- `docs/hld/12-testing-strategy.md`, documentation and CI gates.
- `docs/hld/14-development-backlog.md`, "F-X002, README example correctness".
- `docs/hld/15-build-and-toolchain.md`, CI and full verification.

## Approach

Mark all six README Rust fences `rust,no_run` so rustdoc compiles them without
filesystem side effects. Fix the read example using the existing total indexed
APIs: `row_count` plus `row`, then `cell_count` plus `cell`. Do not add public
iterators merely to preserve incorrect documentation.

Add `scripts/readme_doctests.py` as the one owner of the nontrivial runner. It
builds rdocx with Cargo JSON messages, locates the emitted rlib, and invokes
rustdoc directly against `README.md` with the correct edition, dependency
search path, and `--extern rdocx`. Invoke that runner from the existing CI docs
job and the canonical full verify docs step. Keep README as the sole snippet
source.

## Rejected alternatives

- Copy README snippets into crate docs or tests. The copies would drift.
- Add `rows()` and `cells()` APIs for one example. The indexed facade is
  already total and sufficient.
- Execute the examples against committed sample files. Compilation is the
  stated gate and must not write repository outputs.
- Duplicate artifact-discovery shell in CI and verify. One small runner keeps
  the command stable and testable.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| doctest, gate | `python3 scripts/readme_doctests.py` | All six README examples compile against the actual rdocx rlib in no-run mode |
| regression | bad-table-iterator mutation | Restoring `table.rows()` makes the exact runner fail |
| hygiene | repository output scan | The doctest gate creates no sample or document output in the worktree |

Sensitivity runs the exact runner against a disposable README mutation with
the nonexistent iterator restored, requires a compile failure, then reruns the
tracked README green.

## HLD impact

- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- New file. Obtain explicit approval for `scripts/readme_doctests.py` and keep
  it as the sole runner rather than adding a module or test binary.
- Canonical command and generated adapter. Update the full verify docs step,
  regenerate the verify skill adapter, and pass the drift gate.

## Hash harness

Expected unchanged. README compilation does not generate baseline samples.

## Implementation checklist

- [x] Fix the six README fences and indexed table example.
- [x] Add the approved direct README rustdoc runner.
- [x] Wire the runner into existing CI and full verify.
- [x] Prove the bad iterator mutation fails without worktree output.
- [x] Run docs, prose, skill sync, and hash gates.

## Open questions

None. New tracked path `scripts/readme_doctests.py` is approved as the one
durable runner for CI and full verification.
