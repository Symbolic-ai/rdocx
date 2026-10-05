# Current Sprint, S89

**Milestone**: M24, modern DOCX authoring completeness.

**Goal**: complete note separators and numbering policy, cross-story range
markers, deterministic fragment dependency remapping, and public glossary
creation. These four stories establish the shared story and package semantics
needed by later fields, templates, forms, and collaboration work.
Include PR 269's fontdb feature-unification repair for downstream SVG users.

## Spec references

- `docs/hld/02-scope-and-non-goals.md`, for the DOCX-036 and DOCX-041 through
  DOCX-044 capability gaps and their owners.
- `docs/hld/03-architecture.md`, for story ownership, note policy, staged
  fragment import, and the glossary root model.
- `docs/hld/04-opc-and-packaging.md`, for package integrity and relationship
  validation after remapping or glossary insertion.
- `docs/hld/12-testing-strategy.md`, for differential, round-trip, and
  regression verification of the changed behavior.
- `docs/hld/14-development-backlog.md`, F-274 through F-277, for each story's
  contract, dependencies, and test gate.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-274 | Note separators, markers, and restart policy | L | done | - |
| F-275 | Cross-story bookmarks, ranges, and annotations | L | done | - |
| F-276 | Complete fragment conflict and dependency policy | L | done | - |
| F-277 | Glossary and building-block creation | L | done | - |
| F-X177 | Accept unified fontdb source features | S | done | - |

## Sequencing note

Rows are listed in dependency order, not F-ID order. F-274 and F-275 can start
independently after their completed prerequisites. F-276 follows both because
fragment import must remap their note and paired-range dependencies. F-277
then uses the F-276 transaction for building-block insertion. The hash harness
baseline has one owner at a time.
F-X177 has no feature dependency. Its documentation and archive evidence
share files with the authoring workers, so it uses a separate implementation
wave after F-276 and before F-277.

## Definition of done for this sprint

- Note separators, custom markers, and section restart rules match the pinned
  placement and numbering checks after save and reopen.
- Paired ranges work across supported stories and nested containers. Invalid
  crossing ranges fail atomically.
- Full-story fragment import remaps declared dependencies deterministically,
  leaves no dangling IDs, and preserves untouched XML.
- Public glossary and building-block creation and insertion survive save and
  reopen with relationships and unsupported siblings intact.
- The font source match compiles with downstream `fontdb/memmap` feature
  unification, including the no-default-feature path.
- The integrated verification gate and sprint review pass on the final tree,
  with every intentional hash harness delta stated and reviewed.
