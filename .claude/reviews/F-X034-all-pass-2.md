# F-X034, all, pass 2

**Reviewed**: uncommitted `work/f-x034-codex` diff, 7 files and 655 changed lines
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, reviewed notes are not validated before irreversible crates.io publication
`.github/workflows/publish.yml:52`
`.github/workflows/publish.yml:104`

Both real family allowlists publish first. The only workflow invocation of the
release-note validator is in the later `release` job, which declares
`needs: publish`. If a matching tag points at a commit with a missing, invalid,
or unrenderable changelog section, every selected crate can be published
successfully before the notes check fails. crates.io versions are immutable,
so the workflow cannot roll that release back or produce the reviewed body
from the tagged SHA afterward. `/release` performs the intended local
preflight, but the tag-triggered workflow remains the external mutation
boundary and accepts any matching pushed tag. The notes check must gate the
real publish commands, not only GitHub release creation. The workflow contract
test currently inspects only the later `release` job and therefore does not
exercise this ordering boundary.

## Smells

None.

## Nitpicks

None.

## Not found

All three pass 1 findings are remediated. HTML-comment headings and headings
inside arbitrary-length backtick or tilde fences are ignored, leading-zero
semantic version components are rejected, and the GitHub release step runs the
exact check, render, byte comparison, and `gh release create` commands in one
uninterrupted shell step. The strengthened mutation matrices reject an altered
validator executable, an overwritten artifact, an intervening workflow step,
and release creation before comparison.

No additional defects were found in visible changelog heading selection,
required section order, empty and placeholder rejection, family separation,
command evidence requirements, read-only check and render modes, exact body
consumption, final approval, post-publication comparison, generated adapter
digests, or implicit-invocation policy.

The five focused workflow tests passed. Direct probes confirmed the two pass 1
Markdown-context triggers are now rejected. The generated-skill drift check,
prose check, and `git diff --check` also passed.
