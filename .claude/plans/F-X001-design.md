# F-X001, rdocx-cli tests

**Status**: approved
**Sprint**: S36
**Size**: M
**Depends on**: none

## Problem

The published `rdocx` binary has seven subcommands and no automated tests. Its
exit-status behavior, output files, JSON, text extraction, replacement, diff,
and rendering can regress without any focused command-level gate.

F-143 changes two shared-contract call sites in this executable. This test
story therefore runs after F-143 so its coverage locks the final schema and
output behavior rather than a transient pre-migration state.

## Spec reference

- `docs/hld/10-bindings-spec.md`, "CLIs" and the existing rdocx command surface.
- `docs/hld/12-testing-strategy.md`, "Gaps being closed".
- `docs/hld/14-development-backlog.md`, "F-X001, rdocx-cli tests".

## Approach

Add one integration entrypoint at `crates/rdocx-cli/tests/integration.rs`.
Construct all DOCX and corrupt-package inputs in code and invoke the compiled
binary with `std::process::Command` plus `CARGO_BIN_EXE_rdocx`. Use standard
library assertions and temporary paths derived from the test process ID, with
no `assert_cmd`, snapshot crate, binary fixture, helper module, or second test
binary.

Add at least one command-level case for each of the exact seven commands:
`inspect`, `text`, `convert`, `diff`, `replace`, `validate`, and `render`.
Assert observable output, files, exit status, and reopenability rather than
implementation details. Cover schema 1 and shared default output after F-143.

## Rejected alternatives

- Unit-test command functions only. That misses clap parsing, stdout, stderr,
  file paths, and process exit status.
- Add one test file per command. Each file creates another binary to link.
- Add `assert_cmd` and snapshot dependencies. Standard process and string
  assertions are sufficient for this bounded surface.
- Use checked-in DOCX fixtures. Repository policy constructs fixtures in code.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| integration, gate | one named integration test per subcommand | All seven compiled command paths execute and their public outputs and exit statuses are correct |
| integration | `inspect_json_uses_the_shared_schema` | JSON parses with top-level schema 1 and expected counts |
| integration | `validate_exit_status_is_a_verdict` | Valid input exits zero and a dangling internal relationship exits nonzero |
| round-trip | convert and replace cases | Produced DOCX, PDF, HTML, Markdown, PNG, and replacement outputs are valid and reopen where applicable |
| concurrency | test temp paths | Command tests do not share fixed outputs across concurrent processes |

Sensitivity changes one expected subcommand name or forces validate to report
success. The exact integration suite must fail before byte-identical restoration
and a green rerun.

## HLD impact

- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- New file. Obtain explicit approval for
  `crates/rdocx-cli/tests/integration.rs`. It is the only new test binary.
- Layout and rendering. Command render assertions use deterministic fonts and
  require unchanged hash and golden baselines.

## Hash harness

Expected unchanged. Tests exercise existing commands and do not change product
behavior or sample generation.

## Implementation checklist

- [ ] Add the approved single integration entrypoint.
- [ ] Construct valid and corrupt DOCX inputs entirely in code.
- [ ] Add at least one end-to-end case for each of seven subcommands.
- [ ] Prove command and exit-status sensitivity.
- [ ] Run full rdocx-cli, workspace, render, and hash gates.

## Open questions

None. New tracked path `crates/rdocx-cli/tests/integration.rs` is approved as
the sole integration test binary for all seven subcommands.
