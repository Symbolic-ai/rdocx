# Current Sprint, S43

**Milestone**: X Cross-cutting.

**Goal**: clear the follow-ups S41 and S42 filed. Three are defects a real
document can reach, and two close gaps in the gates that let those defects
survive as long as they did.

## Spec references

- `docs/hld/03-architecture.md`, for the note placement and reflow the two
  layout follow-ups extend.
- `docs/hld/12-testing-strategy.md`, "The hash harness", which F-X021 changes
  the shape of, and "Test taxonomy" for the categories each story picks from.
- `docs/hld/15-build-and-toolchain.md`, for the publication gate F-X025 brings
  under `/verify`.
- `docs/hld/14-development-backlog.md`, for the five story definitions.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-X018 | Unknown enumerated values must not fail a document open | M | done | - |
| F-X017 | Notes broken to their own section's width | S | in-progress | claude |
| F-X019 | Paragraph-relative later drawings should wrap | M | pending | - |
| F-X021 | The hash harness should cover PDF output | M | in-progress | claude |
| F-X025 | /verify must run the release regressions | S | pending | - |

## Sequencing note

Rows are listed in the order they should land, not by F-ID.

F-X018 leads because it is the only story here where a document fails to open
rather than rendering imperfectly. F-X014 fixed three kashida values because a
real contribution reached them, and eight more parsers have the same shape.

F-X017 and F-X019 are the two limitations S41 recorded rather than hid. Both are
narrow, both are reachable by a real document, and both were written up with the
fix already described.

F-X021 and F-X025 are the gates, and they land last on purpose. Neither blocks
the three defect fixes, and putting a gate improvement ahead of work a user can
actually hit would be optimising for the process rather than the product. They
are in the sprint because this is the moment their absence is freshest: F-X021
exists because a dependency refresh changed every sample PDF while the harness
reported green, and F-X025 because a version bump passed the whole local gate
while leaving the publication preflight stale.

## Definition of done for this sprint

- A document carrying an unmodelled value for any of the nine enumerations
  opens, keeps every sibling property, and renders the default for the
  unmodelled one.
- A note is broken to the width of the section that references it.
- A wrapping drawing anchored to a later paragraph pushes earlier text aside
  even when it is positioned relative to its own paragraph.
- The harness records a stable fingerprint for PDF output, and a deliberate
  change to the PDF writer moves it while leaving the PNG entries untouched.
- `/verify --full` fails on a stale version literal in the release regressions
  or the workflow files.
- Every harness delta in the sprint is stated and justified in the commit that
  causes it, including the deliberate re-record F-X021 requires.
