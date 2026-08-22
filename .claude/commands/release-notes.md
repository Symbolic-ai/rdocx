---
description: Prepare and validate meaningful reviewed release notes in CHANGELOG.md for one stable or incubating release tag.
---

# /release-notes {vX.Y.Z | rpptx-vX.Y.Z}

Prepare the human-written release record that `/release` publishes unchanged.
This ceremony edits only `CHANGELOG.md`, including the matching section and
any necessary `Unreleased` cleanup. It does not create a tag, push, publish, or
create a GitHub release.

## Inputs

Choose exactly one release family from the requested tag:

- `vX.Y.Z` is the stable rdocx family.
- `rpptx-vX.Y.Z` is the incubating OOXML and PowerPoint family.

Find the current sprint story whose title is `Tag <requested-tag>`. Read its
design plan and dependencies before collecting claims. Refuse a tag that has
no release story in the active sprint or whose plan is not approved.

## Evidence

Build the notes from reviewed repository evidence, not memory:

1. Find the previous tag in the selected family. Use the repository root when
   the family has no previous tag.
2. Read the release story and design plan, completed `AS_BUILT.md` entries in
   the selected commit range, and the reviewed commits in that range.
3. Inspect merged pull requests in the range. Use the pull request record for
   its author and user-facing purpose. Do not infer contributor identity from
   an unauthenticated display name.
4. Read the current compatibility and migration contract in `CHANGELOG.md`,
   the release plan, and the relevant HLD sections.
5. Separate changes for the selected family from changes for the other family.
   Exclude internal workflow work unless it changes a user-visible release or
   compatibility promise.

Every highlight, addition, fix, compatibility statement, and contributor must
be supported by that evidence. Refuse unverifiable claims or credit instead of
inventing them.

## Write the reviewed section

Add exactly one second-level heading whose text is the requested tag. Keep the
sections below in this order and give every section meaningful text:

```markdown
## v0.8.0

### Highlights

Describe the most important user outcome.

### Added

Describe user-visible capabilities.

### Fixed

Describe user-visible fixes, or state that there are no user-facing fixes.

### Compatibility

State required migration action, supported compatibility, or that no action is
required.

### Contributors

Credit verified contributors and maintainers whose work is in this release.
```

Do not copy the example prose. Do not use placeholders such as `TBD`, `TODO`,
`FIXME`, `CHANGEME`, `PLACEHOLDER`, or `???`. Keep the existing `Unreleased`
material accurate after moving claims into the versioned section.

## Validate and review

Run both deterministic modes:

```bash
python3 scripts/sprint_workflow.py release-notes <requested-tag> --check
python3 scripts/sprint_workflow.py release-notes <requested-tag> --render
```

The rendered output is the exact GitHub release body. Inspect it alongside the
release plan, the commit range, the pull requests, the contributor evidence,
and the `CHANGELOG.md` diff. Run the prose gate and generated-skill drift gate
before handing the release story to review.

Report the selected family, previous tag or repository root, evidence range,
included pull requests and contributors, compatibility conclusion, and the
successful check and render commands.

## Refused situations

- The tag is not exactly `vX.Y.Z` or `rpptx-vX.Y.Z`.
- The active sprint has no approved release story for the exact tag.
- The versioned heading is missing, duplicated, incomplete, empty, or contains
  a placeholder.
- A note belongs only to the other release family.
- A product claim or contributor identity cannot be verified from the reviewed
  record.
- The requested action would tag, push, publish, or create a release. Continue
  through `/release` after review and full sprint verification.
