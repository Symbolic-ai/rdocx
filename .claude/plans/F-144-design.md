# F-144, rpptx-cli

**Status**: completed
**Sprint**: S36
**Size**: L
**Depends on**: F-143, F-116, F-104

## Problem

The presentation facade can open, inspect, mutate, validate, and render decks,
but there is no `rpptx` executable. HLD10 requires the same seven-command shape
as `rdocx-cli`, with presentation-specific behavior and shared range, output,
and JSON contracts.

A complete formatting-preserving replace command also needs one additive
facade operation. Implementing replacement by rewriting whole shape text from
the CLI would discard run formatting and create package logic outside `rpptx`.

## Spec reference

- `docs/hld/03-architecture.md`, ownership and dependency direction.
- `docs/hld/06-presentationml-model.md`, presentation text and package preservation.
- `docs/hld/10-bindings-spec.md`, "CLI".
- `docs/hld/12-testing-strategy.md`, CLI and pinned corpus gates.
- `docs/hld/14-development-backlog.md`, "F-144, rpptx-cli".
- `docs/hld/15-build-and-toolchain.md`, package and publication order.

## Approach

Create published `rpptx-cli` 0.1.2 with one binary and one integration-test
entrypoint. Implement exactly `inspect`, `text`, `convert`, `diff`, `replace`,
`validate`, and `render`.

- `inspect [--json]` reports file, slide and layout counts, slide size, core
  metadata, and per-slide identity, hidden state, and shape count. JSON uses
  the shared schema-1 envelope.
- `text` prints slide text in document order.
- `convert --to pdf|png [-o]` uses deterministic facade rendering and shared
  output defaults. Multi-slide PNG names use one-based suffixes.
- `diff` reports a slide-text longest-common-subsequence comparison.
- `replace` delegates to new additive
  `Presentation::replace_text(&mut self, &str, &str) -> usize`, which traverses
  nested groups and tables and preserves formatting across same-run and
  cross-run matches.
- `validate` exits zero only when facade validation reports no issue.
- `render [-o dir] [--dpi] [--slide range]` uses deterministic fonts and the
  shared one-based range grammar.

F-145 retains sole ownership of thumbnail and outline. The CLI does not read or
mutate raw PresentationML and adds no command trait or wrapper library.

## Rejected alternatives

- Copy shared CLI helpers. F-143 is their format-neutral owner.
- Replace whole shape text in the binary. That loses run formatting.
- Reach into `rpptx-oxml`. Package and text mutation belong to the facade.
- Add thumbnail and outline early. F-145 owns those commands and tests.
- Split each subcommand into another module or test binary. Three source files
  and one integration entrypoint keep the new crate readable.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| integration, gate | `validate_rejects_corruption_and_accepts_the_pinned_corpus` | A deck with a dangling relationship exits nonzero and all 50 verified corpus decks exit zero |
| integration | one case per command | Inspect schema, text order, PDF and PNG signatures/default paths, diff, replacement, and selected render behavior are correct |
| round-trip | `replacement_preserves_formatting_and_opaque_parts` | Literal replacement reopens with run formatting and unmodelled package content intact |
| unit | facade replacement matrix | Same-run, cross-run, nested-group, and table-cell matches return the exact replacement count |

The corpus test requires the verified pinned corpus and never silently skips.
Sensitivity corrupts one relationship and proves the exact validate command
changes exit status. A replacement mutation that rewrites whole shape text must
fail the formatting-preservation gate before byte-identical restoration.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/06-presentationml-model.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- New crate, module, and files. Obtain explicit approval for
  `crates/rpptx-cli/Cargo.toml`, `src/main.rs`, `src/commands.rs`, and
  `tests/integration.rs`.
- Crate dependency graph. Prove `rpptx-cli` depends inward on `rpptx` and
  `oxml-cli-support`, with no reverse or forbidden shared-family edge.
- Public API of published `rpptx`. The replacement method is additive. Run
  publication dry run and archive-size checks and preserve opaque subtrees.
- Layout and rendering. Use deterministic fonts for every rendering assertion
  and require the hash and golden gates to remain unchanged.
- Version and release metadata. Inspect manifests, lockfile, exact incubating
  membership, publication allowlist, tag template, and workflow regressions.

## Hash harness

Expected unchanged. CLI operations and the additive facade replacement do not
alter the canonical sample generator or rendering defaults.

## Implementation checklist

- [x] Create the approved CLI crate and workspace/release wiring.
- [x] Add the formatting-preserving facade replacement seam.
- [x] Implement the seven bounded commands through public facades.
- [x] Add the single integration entrypoint and complete command matrix.
- [x] Run corpus, deterministic render, dependency, publication, and hash riders.

## Open questions

None. The four exact new crate paths, future published incubating 0.1.2
metadata, release-contract expansion from 12 to 14 incubating packages and 19
to 21 workspace publication dry runs, and additive
`Presentation::replace_text` are approved. This does not tag or publish a
package.
