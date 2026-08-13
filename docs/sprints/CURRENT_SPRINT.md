# Current Sprint, S36

**Milestone**: M13 Bindings and tooling.

**Goal**: Complete the v1 command-line and JavaScript package surfaces, then
close the remaining cross-cutting quality gaps. Add shared CLI plumbing, ship
the presentation CLI including thumbnail and outline commands, establish
installable npm packages for both WASM bindings, and finish the CLI, README,
sample-generator, and concurrent-test hardening work.

## Spec references

- `docs/hld/02-scope-and-non-goals.md`, for the v1 requirement that both
  libraries ship supported CLI and WASM package surfaces.
- `docs/hld/03-architecture.md`, for ownership of `oxml-cli-support` and
  `rpptx-cli`, plus the dependency direction from format-neutral plumbing to
  the presentation facade.
- `docs/hld/08-rendering-spec.md`, for the shared presentation rendering path
  used by the thumbnail command.
- `docs/hld/10-bindings-spec.md`, for the mirrored CLI command set,
  presentation-specific thumbnail and outline commands, shared range and JSON
  contracts, and WASM package names.
- `docs/hld/12-testing-strategy.md`, for CLI exit-status, rendering, package,
  and installation gates.
- `docs/hld/14-development-backlog.md`, for F-143 through F-146 and F-X001
  through F-X004 dependencies and named acceptance gates.
- `docs/hld/15-build-and-toolchain.md`, for CLI publication order and the
  unpublished WASM package boundary that npm packaging closes.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-143 | oxml-cli-support | S | in-progress | codex |
| F-146 | npm publication | S | pending | - |
| F-X001 | rdocx-cli tests | M | pending | - |
| F-X002 | README example correctness | S | pending | - |
| F-X003 | Deduplicate the sample generators | S | in-progress | codex |
| F-X004 | Fix the shared temp path in the test suite | S | in-progress | codex |
| F-144 | rpptx-cli | L | in-progress | codex |
| F-145 | rpptx-cli thumbnail and outline | M | pending | - |

## Sequencing note

Rows are listed in dependency order, not by F-ID. F-143 establishes the shared
CLI contracts required by F-144, and F-145 follows F-144 because it extends
that executable. F-146 can start independently because its dependencies F-140
and F-142 are complete. F-X001 through F-X004 are dependency-independent
hardening stories and can run alongside the first CLI wave, subject to ordinary
file-conflict checks during design.

## Definition of done for this sprint

- `2,4-6` parses to the expected range and the shared JSON envelope carries
  `"schema": 1`.
- `rpptx-cli validate` exits non-zero on a corrupted deck and zero across the
  pinned corpus.
- `rpptx-cli thumbnail` produces a PNG of slide one, and `outline` prints the
  title and bullet tree.
- `npm pack` produces installable `@tensorbee/rdocx-wasm` and
  `@tensorbee/rpptx-wasm` tarballs without publishing either package.
- Every `rdocx-cli` subcommand has an integration test.
- README examples compile as doctests.
- One sample generator produces every artifact required by the hash harness.
- Two concurrent test runs pass without sharing a fixed temporary path.
- The full verification gate, package checks, and all 28 deterministic hashes
  pass without forbidden dependency edges.
