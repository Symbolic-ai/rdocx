# Current Sprint, S90

**Milestone**: M24, modern DOCX authoring completeness.

**Goal**: provide a complete field construction surface and deterministic
field results across stories, then compose captions, cross-references, indexes,
citations, and numbering-aware navigation. Build on S89's note and range
contracts while preserving producer XML and saved field caches.
Issue 264 and F-X178 are excluded and carried for separate work. F-282 covers
the full pinned Word bibliography source, style and locale catalogue.

## Spec references

- `docs/hld/02-scope-and-non-goals.md`, for DOCX-045 through DOCX-050 and the remaining field and navigation capability owners.
- `docs/hld/03-architecture.md`, for recursive field grammar, immutable evaluation, physical story ownership and the pagination boundary.
- `docs/hld/04-opc-and-packaging.md`, for relationship closure and preservation when authoring fields and bibliography parts.
- `docs/hld/08-rendering-spec.md`, for deterministic pagination, page targets and field result placement.
- `docs/hld/12-testing-strategy.md`, for round-trip, regression and pinned Word differential gates.
- `docs/hld/14-development-backlog.md`, F-278 through F-283, for the six contracts, dependencies and test gates.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-278 | General simple and complex field builder | L | done | - |
| F-279 | Pagination field materialization across stories | L | done | - |
| F-280 | Captions, sequences, and complete cross-references | M | done | - |
| F-281 | Indexes and tables of figures and authorities | L | done | - |
| F-282 | Citations and bibliography authoring | L | in-progress | codex |
| F-283 | Complete numbering-aware navigation fields | L | pending | - |
| F-X178 | Clearable direct run formatting setters | S | pending | - |
| F-X179 | Correct multi-paragraph comment threads from PR 271 | S | done | - |
| F-X180 | Correct cell nil and none border precedence | S | in-progress | codex |
| F-X181 | Ignore page and column breaks inside table cells | S | in-progress | codex |
| F-X182 | Honor direct table alignment | M | pending | - |
| F-X183 | Correct table margins and legacy positioning | M | pending | - |

## Sequencing note

Rows are listed in dependency order, not implementation waves. F-278 supplies
the field substrate. F-279 consumes the completed S89 note policy and F-280
consumes S89 range markers. Both depend on F-278. F-281 follows F-278 through
F-280, while F-282 needs only F-278. F-283 follows F-279 through F-282 and the
completed numbering foundation. Shared source and test files determine which
otherwise independent stories can run together after design.
F-X178 remains pending in the backlog and carried in the S90 run state.
It has no implementation wave in this sprint, honoring the Issue 264 exclusion.

F-X179 adopts PR 271 against all of Issue 270, including multiline text and
last-paragraph thread metadata. It has no feature dependency. Issue 264 and
F-X178 remain outside this intake. GitHub closure waits for sprint close.

The user added Issues 272 and 273 with PRs 274 and 275 during S90.
F-X180 and F-X181 validate each full issue before accepting its contribution.
Their source and regression files overlap F-282 and F-283. Pause F-282 at a
saved checkpoint, run these independent fixes in separate waves, then resume
F-282 before F-283. Source and Cargo ownership never overlap. Reconcile both
approved contracts at integration and check table geometry and bibliography
output. Issue 264 remains excluded.


Newly opened Issues 277 and 276 add F-X182 and F-X183. Run direct alignment
before legacy positioning, with F-X182 completed at a scoped dependency
checkpoint before F-X183 starts. Keep F-282 paused through these exclusive
source and hash-baseline waves, then resume it before F-283. Each rendering
delta is separately declared and reviewed. Issue 264 remains excluded.
PR 279 supplies F-X182. PR 280 is stacked on it and supplies F-X183, including new Issue 278 for unset side margins. Validate both complete issue sets before contributor disposition.

## Definition of done for this sprint

- Simple, complex and nested fields reopen with identical instruction semantics and ordered cached content.
- Pagination field caches across body, headers, footers, notes and text boxes match the pinned Word page and section values.
- Captions and cross-references match Word before and after insertion and renumbering.
- Index, figure and authority tables retain the ordered entries and page targets produced by a pinned Word update.
- Citation and bibliography identifiers, ordering, display text and package round trips match the pinned Word oracle.
- One source-built multilevel numbered document stays consistent across visible markers, navigation, references and saved caches.
- The integrated full verification and sprint review pass, including the source-built document's field caches, page targets and numbering checks. Any intentional hash delta is declared and reviewed.

- Cell border precedence and table-cell break handling satisfy Issues 272 and 273 against the pinned Word oracle.

- Direct table alignment and compatibility-mode positioning satisfy Issues 277 and 276 against pinned Word controls.
