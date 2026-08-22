# F-X034, all, pass 1

**Reviewed**: uncommitted `work/f-x034-codex` diff, 7 files and 523 changed lines
**Verdict**: 3 defects, 0 smells, 0 nitpicks

## Defects

### D1, non-rendered Markdown headings can be accepted as the release section
`scripts/sprint_workflow.py:151`
`scripts/sprint_workflow.py:159`

`markdown_heading_lines` ignores only a simplified three-character fence
state and does not ignore HTML comments. A complete `## v1.2.3` section inside
`<!-- ... -->` is therefore accepted and rendered even though the changelog
has no visible version heading. The returned release body includes the orphan
comment terminator. A four-backtick code fence is another trigger. An inner
three-backtick example incorrectly closes the tracked fence, so a tag and all
five required headings that remain inside the outer code block are accepted as
live notes. Both cases let `--check` approve a different semantic section from
the one a Markdown reviewer sees, which breaks the exact reviewed-body
contract. The parser test covers ordinary headings and mutations but neither
Markdown context.

### D2, the exact tag validator accepts noncanonical semantic versions
`scripts/sprint_workflow.py:61`

The version components use unrestricted digit repetitions, so tags such as
`v01.2.3`, `v1.02.3`, and `rpptx-v1.2.03` pass validation. Numeric semantic
version components cannot contain leading zeroes. These spellings also cannot
match the canonical package version that `/release` checks, so the ceremony
can create and validate a changelog section for a tag that the release family
contract must refuse. The invalid-tag test matrix covers missing components,
foreign prefixes, and suffixes but not leading-zero forms.

### D3, the mutation tests do not prove that the rendered file reaches GitHub unchanged
`scripts/test_sprint_workflow.py:3764`
`scripts/test_sprint_workflow.py:4371`

The publish contract asserts only that the render step appears somewhere
before release creation. Inserting a step between them that overwrites
`$RUNNER_TEMP/release-notes.md` leaves every assertion green while GitHub
receives different text. The release-command contract is similarly satisfied
if the executable prefix is changed from `python3 scripts/sprint_workflow.py`
to `echo`, because its check begins at the `release-notes` substring. The
included mutation matrices do not exercise either boundary. The approved
regression gate is specifically mutation-sensitive and must prove that the
validator actually executes and that no workflow step can replace its output
before `gh release create` consumes it.

## Smells

None.

## Nitpicks

None.

## Not found

No additional defects were found in the ceremony's evidence requirements,
family separation, contributor refusal rules, changelog section order,
placeholder and empty-section checks, read-only check and render modes,
current publish command, release preflight, separate final approval, or
post-publication byte comparison. The generated `release-notes` adapter and
updated `release` adapter match their canonical command digests, and implicit
invocation is disabled for the new ceremony.

The five focused workflow tests passed. The generated-skill drift check,
prose check, and `git diff --check` also passed. Direct read-only probes
confirmed that the current renderer accepts both the commented-section and
four-backtick-fence triggers described above.
