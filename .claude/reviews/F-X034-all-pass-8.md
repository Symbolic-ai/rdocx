# F-X034, all, pass 8

**Reviewed**: uncommitted `work/f-x034-codex` diff, 8 files and 1,342 added or deleted lines
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, the HTML visibility pass ignores hidden-content semantics
`scripts/sprint_workflow.py:286`

`VisibleReleaseNoteText.handle_starttag` discards every attribute and hides
only five tag names. A required section containing only
`<div hidden>Invisible release notes</div>` therefore passes even though the
standard `hidden` attribute removes that content from presentation. Containers
whose children are not normally displayed, including `iframe` fallback and
`noscript` content in a scripted browser, also satisfy the check. Direct probes
reproduced all three accepted cases. The current HTML matrix covers empty tags
and five fixed invisible containers but not standard hidden attributes or the
remaining non-presented containers.

## Smells

None.

## Nitpicks

None.

## Not found

The pass 7 Unicode-code defect is remediated. Inline and fenced code now use
the same meaningful-character predicate as prose. Zero-width, directional,
spacing, soft-hyphen, BOM, combining-mark, and variation-selector payloads are
rejected, while real text and pictographs remain accepted with exact output.

No additional defects were found in Markdown links, references, code fences,
inline code, block markers, heading state, tag validation, required heading
order, placeholder rejection, family separation, check and render
immutability, pre-publication validation ordering, exact GitHub body
comparison, final approval, contributor evidence, or post-publication
verification. Generated adapters remain synchronized and explicitly invoked.

The five focused workflow tests passed. The generated-skill drift check,
prose check, and `git diff --check` also passed.
