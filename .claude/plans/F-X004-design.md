# F-X004, Fix the shared temp path in the test suite

**Status**: completed
**Sprint**: S36
**Size**: S
**Depends on**: none

## Problem

`crates/rdocx/tests/integration_test.rs` writes `save_and_load_file` to the
fixed process-global path `rdocx_test_output.docx` under the operating-system
temporary directory. Two concurrent integration-test processes can therefore
overwrite or remove each other's file even though the document code is
correct.

## Spec reference

- `docs/hld/12-testing-strategy.md`, "Test taxonomy".
- `docs/hld/14-development-backlog.md`, "F-X004, Fix the shared temp path in the test suite".

## Approach

Keep the test in the existing `integration_test` binary and derive its output
name from `std::process::id()`. Add a direct assertion that the exercised path
contains the current test-process identifier, retain best-effort cleanup, and
add no dependency or helper module. Concurrent cargo invocations use distinct
test-process IDs, while repeated operations inside this one test retain one
clear path.

## Rejected alternatives

- Add `tempfile` only for this test. A process-unique standard-library path is
  sufficient and avoids a new dependency and lockfile change.
- Use a timestamp or random suffix. Process identity is deterministic and
  directly matches the concurrent-process failure mode.
- Add another integration test file. Repository policy keeps tests in the
  existing binary to avoid another link target.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| integration | `save_and_load_file` | The exercised path is process-unique and the saved document reopens correctly |
| concurrency, gate | two concurrent exact `cargo test -p rdocx --test integration_test save_and_load_file -- --exact` processes | Both runs complete successfully without sharing or deleting one path |

Sensitivity temporarily restores the fixed filename, proves the exact path
assertion fails, restores byte-identically, and reruns both concurrent commands
green.

## HLD impact

None. This is a test-isolation correction and does not change the documented
product or harness mechanism.

## Risk routing

None. The diff changes one integration test and introduces no dependency,
public API, parser, serializer, binding, feature, or external oracle.

## Hash harness

Expected unchanged. The modified test does not generate baseline samples.

## Implementation checklist

- [x] Make the existing save-and-load test path process-unique.
- [x] Assert the exercised path includes the current process identifier.
- [x] Run the exact test twice concurrently and require both exits to pass.
- [x] Prove the fixed-name mutation fails and restore byte-identically.
- [x] Run the full rdocx suite, hash harness, format, prose, and diff checks.

## Open questions

None.
