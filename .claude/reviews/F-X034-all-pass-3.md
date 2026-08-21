# F-X034, all, pass 3

**Reviewed**: uncommitted `work/f-x034-codex` diff, 8 files and 743 added or deleted lines
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, release headings inside raw HTML blocks are accepted as reviewed Markdown
`scripts/sprint_workflow.py:150`

`markdown_heading_lines` excludes HTML comments and fenced code, but it does
not track CommonMark raw HTML blocks. A complete `## v1.2.3` release section
inside `<script>`, `<pre>`, or `<style>` is therefore accepted and rendered,
even though Markdown renders its headings as raw block content rather than as
the visible versioned changelog section the ceremony requires. The renderer
then publishes the extracted inner text with the unmatched closing HTML tag,
so the reviewed changelog presentation and the GitHub release presentation
are not semantically faithful. Direct probes reproduced acceptance for all
three block types. The parser regression matrix covers comments and variable
length fences, but it does not cover raw HTML blocks.

## Smells

None.

## Nitpicks

None.

## Not found

The pass 2 defect is remediated. The publish job now runs the exact release
note validator before either family's first crates.io command, and the
mutation matrix rejects missing, delayed, conditional, ignored, or altered
pre-publication validation.

No additional defects were found in canonical tag validation, required
section order, empty and placeholder rejection, family separation, check and
render immutability, exact render and byte comparison ordering immediately
before `gh release create`, release-command preflight and final approval, or
post-publication comparison. The generated adapters match their canonical
digests and implicit invocation remains disabled for the ceremony.

The five focused workflow tests passed. The generated-skill drift check,
prose check, and `git diff --check` also passed.
