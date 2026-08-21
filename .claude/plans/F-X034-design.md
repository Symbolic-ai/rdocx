# F-X034, Reviewed release notes for every release

**Status**: approved
**Sprint**: S51
**Size**: S
**Depends on**: F-X025

## Problem

The publish workflow creates GitHub releases with `--generate-notes`. That
produces a commit-list summary, but it does not guarantee meaningful product
notes, migration guidance, or contributor credit. The release command also
does not verify notes at the reviewed SHA before asking for final publication
approval.

Every release going forward needs a human-written record of what users gain,
what changed or was fixed, what compatibility action is required, and who
contributed. The notes must be reviewed with the code and must be the exact body
published by the tag workflow.

## Spec reference

- `docs/hld/12-testing-strategy.md`, "Test taxonomy" and workflow contract
  tests.
- `docs/hld/14-development-backlog.md`, "F-X034, Reviewed release notes for
  every release".
- `docs/hld/15-build-and-toolchain.md`, "The two release families" and
  "Release tags".
- `.claude/commands/release.md`, family contract, preconditions, final
  approval, and release sequence.

## Approach

Use the existing root `CHANGELOG.md` as the single tracked source. Each release
story adds a second-level heading whose text is the exact tag, such as
`## rpptx-v0.4.0` or `## v0.8.0`, followed by these nonempty sections:

- `### Highlights`
- `### Added`
- `### Fixed`
- `### Compatibility`
- `### Contributors`

Extend the existing `scripts/sprint_workflow.py` command surface with
`release-notes TAG`. It validates the exact tag syntax, locates one exact
changelog section, rejects missing or duplicate required headings, rejects
empty text and placeholder tokens, and writes only that reviewed section body
to stdout. It performs no network or filesystem mutation.

Update `publish.yml` to render the notes into the runner temporary directory
and pass that file to `gh release create --notes-file`. Remove
`--generate-notes`. Update `/release` so preflight renders and inspects the
exact notes at the reviewed SHA, and its separate final approval summary names
the notes source. After publication, verify the GitHub release body matches the
rendered reviewed section.

Add mutation-sensitive coverage to the existing workflow test module. Do not
add a script, module, template file, or release-note generator. Human-written
notes are the product artifact, and the existing workflow CLI owns their
validation and extraction.

## Rejected alternatives

- Keep `--generate-notes` and prepend a sentence. Generated commit summaries
  do not provide a reviewed user-facing compatibility record.
- Add one release-note file per tag. The existing changelog is already the
  durable source and a second record would drift.
- Generate notes from commit messages or AS_BUILT. Those records are useful
  inputs but cannot decide what users need to know.
- Add a new script. The existing sprint workflow CLI already owns release
  preflight contracts and avoids another file.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit, gate | `test_release_notes_require_complete_reviewed_changelog_sections` | Both tag families extract exact complete notes and missing, duplicate, empty, or placeholder sections fail |
| regression | publish workflow mutation matrix | `--generate-notes`, missing extraction, a different notes file, or release creation before extraction fails the contract test |
| regression | release command authority contract | `/release` validates notes before final approval and verifies the published body afterward |
| prose | changelog fixtures and real release sections | Required headings remain readable and comply with tracked prose rules |

The **test gate** is regression. Release-note extraction returns the exact
versioned changelog section for both tag families, rejects missing or
incomplete notes, and the publish workflow can create a GitHub release only
from that reviewed output.

## HLD impact

- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- **Release scripting, version strings**. Read `/release` and HLD 15. Inspect
  the release-command, changelog, workflow, and test diffs. Require full
  verification, a clean sprint review, exact rendered notes, and a separate
  final approval before any tag or publication.

Changing `.claude/commands/release.md` also requires regenerating the agent
skill adapters and passing the drift gate.

## Hash harness

Expected unchanged across all 49 entries. Release-note validation and workflow
metadata do not affect generated Office or rendered output.

## Implementation checklist

- [ ] Add deterministic validation and extraction to the existing sprint CLI.
- [ ] Require complete versioned changelog sections for both tag families.
- [ ] Publish only the rendered reviewed notes file.
- [ ] Extend `/release` preflight, approval summary, and post-release check.
- [ ] Add mutation-sensitive workflow and command contract tests.
- [ ] Regenerate agent skill adapters after the command edit.
- [ ] Run full verification and the unchanged hash harness.
- [ ] Update exactly the HLD files listed above.

## Open questions

None. The user requires meaningful release notes for every release going
forward. `CHANGELOG.md` is the reviewed source, and generated GitHub notes are
no longer sufficient.
