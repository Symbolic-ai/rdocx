# F-135, python-docx parity suite

**Status**: completed
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
any comparison. Define an explicit seventeen-entry manifest inside one parity
test module for every executable python-docx 1.2.0 documentation example whose
API is in the approved S33 surface. Pin each source URL to the upstream v1.2.0
tag and record the exact local source statements, heading, transformation, and
normalized structural assertions.

Sixteen entries change only the import namespace from `docx` to `rdocx`. The
Quickstart held-row example is the documented exception. Its first cell text
replacement advances the strict global revision and intentionally stales the
held row. Preserve the exact upstream body in the manifest, then make the
minimal public adaptation by re-fetching `document.tables[0].rows[1]` before
the second cell assignment. This keeps strict path and revision semantics while
making the documented operation usable. The line-spacing entry preserves the
tagged source statements without inventing a local import. Its setup supplies
the `Pt` name established earlier in the upstream page.

Cover document construction, open and save, paragraphs, runs, tables, the
approved formatting inventory, units, and enums. Test both producer directions.
Open rdocx output with python-docx, then open python-docx output with rdocx.
Compare public paragraph, run, direct-formatting, table, cell, unit, and enum
records. Preserve relative float line spacing as a distinct normalized value
from absolute `Length` spacing, and author both forms in each producer
direction. Set an explicit table style in both producer directions and compare
it after each saved package is reopened by both readers. Do not compare ZIP or
XML bytes. Pin the exact manifest ID set so an example cannot disappear
silently. Build inputs in code and commit no binary fixtures.

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
| differential, gate | `documented_s33_examples_run_with_declared_transformations` | Every pinned in-scope example runs, the manifest is complete, sixteen bodies use only namespace substitution, and the held-row body uses exactly one public re-fetch |
| differential | `rdocx_and_python_docx_round_trip_the_same_normalized_content` | Both producer directions preserve the approved public structure, distinct relative and absolute line spacing, and table style after save and reopen |
| regression | oracle, source, manifest, re-fetch, line-spacing, and table-style mutations | An oracle upgrade, source drift, missing example, removed held-row re-fetch, collapsed spacing kind, changed relative value, or dropped table style fails before acceptance |

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

- [x] Expand the pinned manifest to seventeen entries with stable v1.2.0
  sources and the exact held-row compatibility adaptation.
- [x] Preserve relative and absolute line spacing distinctly in both producer
  directions.
- [x] Set and compare table style through both saved writers and both readers.
- [x] Assert the exact oracle version and exact manifest ID set.
- [x] Run the parity module with the existing binding suite.
- [x] Prove the gate fails under source, manifest, re-fetch, spacing-kind,
  spacing-value, and table-style mutations.

## Open questions

None. The bounded pinned manifest, sixteen namespace-only substitutions, one
held-row public re-fetch adaptation, new parity test, and exact test-only
oracle pin are explicitly approved.
