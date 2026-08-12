# F-135, python-docx parity suite

**Status**: approved
**Sprint**: S34
**Size**: M
**Depends on**: F-131

## Problem

The binding has focused API tests but no two-way compatibility gate against
the library it ports. The literal phrase "every documented example" is larger
than the completed S33 surface. Even the python-docx Quickstart uses headings,
page breaks, paragraph insertion, pictures, row addition, columns, and styles
that are not part of F-131.

## Spec reference

- `docs/hld/02-scope-and-non-goals.md`, "Python binding compatibility".
- `docs/hld/10-bindings-spec.md`, "Python API shape" and "CI".
- `docs/hld/12-testing-strategy.md`, "Binding tests".
- `docs/hld/14-development-backlog.md`, "F-135, python-docx parity suite".

## Approach

Pin `python-docx==1.2.0` and assert the resolved distribution version before
any comparison. Define an explicit manifest inside one parity test module for
every executable python-docx 1.2.0 documentation example whose API is in the
approved S33 surface. Record the upstream page and heading, the sole namespace
substitution from `docx` to `rdocx`, and the normalized structural assertions.
Keep the example body unchanged after that import substitution.

Cover document construction, open and save, paragraphs, runs, tables, the
approved formatting inventory, units, and enums. Test both producer directions.
Open rdocx output with python-docx, then open python-docx output with rdocx.
Compare public paragraph, run, direct-formatting, table, cell, unit, and enum
records. Do not compare ZIP or XML bytes. Pin the exact manifest ID set so an
example cannot disappear silently. Build inputs in code and commit no binary
fixtures.

## Rejected alternatives

- Follow the live documentation site. Upstream edits would silently redefine
  the sprint gate.
- Claim the entire 1.2.0 user guide. That is a separate implementation program,
  not an M-sized parity story over the completed binding.
- Ship a top-level `docx` alias. It would collide with the oracle package in the
  same environment.
- Compare package bytes. Prefixes, attribute order, and whitespace are not the
  compatibility contract.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| differential, gate | `documented_s33_examples_run_with_only_namespace_substitution` | Every pinned in-scope example body runs and the manifest is complete |
| differential | `rdocx_and_python_docx_round_trip_the_same_normalized_content` | Both producer directions preserve the approved public structure |
| regression | oracle version and manifest ID mutations | An oracle upgrade or missing example fails before comparison |

The test gate is the bounded backlog contract above, with structural two-way
round trips through pinned python-docx 1.2.0.

## HLD impact

- `docs/hld/02-scope-and-non-goals.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- External oracle comparison. Pin `python-docx==1.2.0`, assert the runtime
  version, and compare normalized public trees rather than bytes.
- WASM or PyO3 bindings. Retain binding exclusions and check `rdocx-wasm`.
- New file. Obtain explicit approval for
  `crates/rdocx-py/tests/test_python_docx_parity.py`.

## Hash harness

Expected unchanged. The story adds test infrastructure and does not alter the
document writer.

## Implementation checklist

- [ ] Add the pinned in-scope documentation example manifest.
- [ ] Add both normalized producer directions.
- [ ] Assert the exact oracle version and exact manifest ID set.
- [ ] Run the parity module with the existing binding suite.
- [ ] Prove the gate fails under oracle, manifest, and value mutations.

## Open questions

None. The bounded pinned manifest, sole namespace substitution, new parity
test, and exact test-only oracle pin are explicitly approved.
