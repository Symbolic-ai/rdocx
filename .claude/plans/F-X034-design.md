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
contributed. Preparing that record is a release ceremony, not an incidental
script option. The notes must be reviewed with the code and must be the exact
body published by the tag workflow.

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

Add a canonical `.claude/commands/release-notes.md` command with the invocation
`/release-notes TAG`. The repository's existing skill synchronizer generates
`.agents/skills/release-notes/SKILL.md`, so Claude, Codex, and human operators
follow the same ceremony. The user explicitly approved this new custom command
and generated skill surface.

The ceremony reads the target release F-ID and design plan, completed
`AS_BUILT.md` entries since the previous tag in the selected family, the
reviewed commit range, merged pull requests, contributor identities, and the
current migration contract. It separates stable and incubating family changes,
excludes internal-only work from user-facing highlights, cites compatibility
requirements, and refuses to invent contributor credit or release claims that
the repository cannot prove.

Use the existing root `CHANGELOG.md` as the single tracked output. Each release
story adds a second-level heading whose text is the exact tag, such as
`## rpptx-v0.4.0` or `## v0.8.0`, followed by these nonempty sections:

- `### Highlights`
- `### Added`
- `### Fixed`
- `### Compatibility`
- `### Contributors`

Extend the existing `scripts/sprint_workflow.py` command surface with a
deterministic `release-notes TAG --check` and `release-notes TAG --render`
helper used by the ceremony and automation. It validates the exact tag syntax,
locates one exact changelog section, rejects missing or duplicate required
headings, rejects empty text and placeholder tokens, and renders only that
reviewed section body. Check and render modes perform no tracked-file or
network mutation.

Update `publish.yml` to render the notes into the runner temporary directory
and pass that file to `gh release create --notes-file`. Remove
`--generate-notes`. Update `/release` so preflight renders and inspects the
exact notes at the reviewed SHA, and its separate final approval summary names
the notes source. After publication, verify the GitHub release body matches the
rendered reviewed section.

Add mutation-sensitive coverage to the existing workflow test module. Do not
add a script, template, or generated prose. The one new canonical command and
its generated adapter are the explicitly approved ceremony surface.
Human-written notes are the product artifact, and the existing workflow CLI
owns deterministic validation and extraction.

## Rejected alternatives

- Keep `--generate-notes` and prepend a sentence. Generated commit summaries
  do not provide a reviewed user-facing compatibility record.
- Hide the ceremony inside `sprint_workflow.py`. Deterministic validation
  belongs there, but gathering and writing meaningful notes requires a visible
  custom command that agents and humans deliberately invoke.
- Add one release-note file per tag. The existing changelog is already the
  durable source and a second record would drift.
- Generate notes from commit messages or AS_BUILT. Those records are useful
  inputs but cannot decide what users need to know.
- Add a new script. The existing sprint workflow CLI already owns release
  preflight contracts and avoids another file.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| workflow, gate | `/release-notes` command contract | The ceremony reads the release plan, delivery records, commit range, PRs, contributors, and compatibility contract before writing the exact changelog section |
| unit | `test_release_notes_require_complete_reviewed_changelog_sections` | Both tag families check and render exact complete notes and missing, duplicate, empty, or placeholder sections fail |
| regression | publish workflow mutation matrix | `--generate-notes`, missing extraction, a different notes file, or release creation before extraction fails the contract test |
| regression | release command authority contract | `/release` validates notes before final approval and verifies the published body afterward |
| integration | generated skill drift and validation | The `release-notes` adapter points to the canonical command digest and passes repository skill validation |
| prose | changelog fixtures and real release sections | Required headings remain readable and comply with tracked prose rules |

The **test gate** is regression. The custom command prepares complete notes
from the reviewed release record, its generated skill is in sync,
release-note extraction returns the exact versioned changelog section for both
tag families, incomplete notes fail, and the publish workflow can create a
GitHub release only from that reviewed output.

## HLD impact

- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- **Release scripting, version strings**. Read `/release` and HLD 15. Inspect
  the release-command, changelog, workflow, and test diffs. Require full
  verification, a clean sprint review, exact rendered notes, and a separate
  final approval before any tag or publication. Changing
  `.claude/commands/release.md` also requires regenerating the agent skill
  adapters and passing the drift gate.

- **A new module or file**. The user explicitly approved a custom release-note
  command or skill. Add exactly the canonical command file and its generated
  adapter. Do not add a standalone script, reference bundle, or template file.

## Hash harness

Expected unchanged across all 49 entries. Release-note validation and workflow
metadata do not affect generated Office or rendered output.

## Implementation checklist

- [ ] Add the canonical `/release-notes TAG` ceremony.
- [ ] Generate and validate its agent skill adapter.
- [ ] Add deterministic check and render modes to the existing sprint CLI.
- [ ] Require complete versioned changelog sections for both tag families.
- [ ] Publish only the rendered reviewed notes file.
- [ ] Extend `/release` preflight, approval summary, and post-release check.
- [ ] Add mutation-sensitive workflow and command contract tests.
- [ ] Regenerate agent skill adapters after the command edit.
- [ ] Run full verification and the unchanged hash harness.
- [ ] Update exactly the HLD files listed above.

## Open questions

None. The user requires meaningful release notes for every release going
forward and explicitly chose a custom command or skill ceremony.
`CHANGELOG.md` is the reviewed source, and generated GitHub notes are no longer
sufficient.
