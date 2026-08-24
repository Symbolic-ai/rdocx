# F-X053, Complete layout migration and contribution records

**Status**: completed
**Sprint**: S55
**Size**: S
**Depends on**: F-X051, F-X052

## Problem

The v0.9.0 compatibility section says that `PositionedElement` is
non-exhaustive, but it predates the semantic wrapper added by F-173 and does
not tell external backends that visible page content is now nested below
`PositionedElement::MarkedContent` (`CHANGELOG.md:44` and `CHANGELOG.md:255`).
An exhaustive-looking traversal with a wildcard keeps compiling and can render
an empty page because it never visits the wrapper's children.

Issue 44 and PR 45 remain open even though F-X051 implemented their font-alias
behavior as a hardened equivalent. Issue 46 remains open for the measured
relayout regression. Its contributor also confirmed on Issues 39 and 42 that
note operations recovered and F-X048 replaces the dense-form patches, so those
closed records need no new implementation scope.

## Spec reference

- `docs/hld/03-architecture.md`, "Why these seams" and the backend-neutral
  semantic structure boundary.
- `docs/hld/08-rendering-spec.md`, "The recursion hazard".
- `docs/hld/10-bindings-spec.md`, "Native Word facade stability".
- `docs/hld/14-development-backlog.md`, "F-X053, Complete layout migration
  and contribution records".
- GitHub Issue 44, PR 45, and Issue 46, including the authenticated
  `@emptinessform` reports and measurements.

## Approach

Amend the tracked v0.9.0 compatibility section and the `oxml-layout` package
README. State that page consumers must recurse through
`MarkedContent::children` or use `oxml_layout::walk` when visiting
`PageFrame::elements`. Keep the low-level enum non-exhaustive and make no code
or public API change.

Render the amended v0.9.0 changelog section through the existing release-notes
workflow and replace only the published GitHub release body. Verify exact body
equality after the edit. Do not move the tag, publish a package, or alter any
release asset.

After F-X052 is integrated and measured, post record-specific maintainer
comments. Close Issue 44 and PR 45 as addressed by F-X051 rather than merged.
Close Issue 46 with the F-X052 implementation SHA, the measured A/B result,
and the migration correction. Add the three links and authenticated
`@emptinessform` credit to the tracked next-stable contribution inventory.
Leave Issues 39 and 42 closed and do not post duplicate completion comments.

## Rejected alternatives

- Reopening Issues 39 or 42 would create duplicate scope after the reporter
  supplied positive acceptance evidence.
- Describing PR 45 as merged would misstate how F-X051 landed.
- Editing only the GitHub release body would leave the tracked reviewed source
  different from the published record.
- Adding recursive traversal code to every backend is outside this migration
  and record-only story.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `release_notes_v0_9_0_include_marked_content_migration` | The tracked compatibility section names the non-exhaustive enum, `MarkedContent::children`, and `oxml_layout::walk`. |
| package docs | `oxml_layout_readme_names_recursive_page_traversal` | The published crate README gives both supported traversal choices. |
| integration | `v0_9_0_tracked_and_published_release_bodies_match` | The rendered tracked section and GitHub release body are byte-identical after the correction. |
| external records | authenticated GitHub state check | Issue 44, PR 45, and Issue 46 are closed with the exact implementation evidence and contributor credit, while Issues 39 and 42 remain closed. |

The **test gate** is integration. The tracked v0.9.0 changelog section and
published release body are byte-identical after the compatibility correction,
the note names both supported recursive traversal choices, Issue 44 and PR 45
cite the F-X051 implementation, Issue 46 cites F-X052 and the migration
correction, Issues 39 and 42 remain closed, and the next stable contribution
inventory retains the authenticated reporter and contributor credit.

## HLD impact

- `docs/hld/10-bindings-spec.md`

## Risk routing

none. This story corrects tracked and published documentation plus GitHub
record state. It changes no parser, renderer, public type, package graph,
binding, version, tag, or release script. The external-state checks in the
test plan are additional story evidence.

## Hash harness

Expected unchanged, 49 of 49. This story changes documentation and GitHub
records only. Any output delta is unrelated and blocks the sprint. Do not edit
`scripts/hash_baseline.json`.

## Implementation checklist

- [x] Add the recursive `MarkedContent` migration note to the tracked v0.9.0
      section and package README.
- [x] Update the current HLD API guidance without adding change-history prose.
- [x] Extend the next-stable contribution inventory for Issues 44 and 46 and
      PR 45 with authenticated credit.
- [x] Render and verify the amended tracked release body.
- [x] Replace only the published v0.9.0 GitHub release body and prove exact
      equality.
- [x] Close Issue 44 and PR 45 as addressed by F-X051 with its implementation
      SHA.
- [x] Close Issue 46 with the F-X052 SHA, A/B measurements, and migration note.
- [x] Confirm Issues 39 and 42 remain closed without duplicate comments.
- [x] Run prose, release-notes, package-doc, GitHub-state, and unchanged-harness
      checks.

## Open questions

None.
