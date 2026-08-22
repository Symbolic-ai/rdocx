# F-X034, all, pass 4

**Reviewed**: uncommitted `work/f-x034-codex` diff, 8 files and 790 added or deleted lines
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, the raw HTML remediation covers only one CommonMark block class
`scripts/sprint_workflow.py:201`

The scanner now excludes type-1 raw HTML blocks for `script`, `pre`, `style`,
and `textarea`, but headings remain accepted inside the other non-rendered
CommonMark HTML block forms. A complete release section with no blank lines
inside `<div>...</div>`, `<?...?>`, `<!HIDDEN...>`, or
`<![CDATA[...]]>` passes validation and renders with its orphan closing marker.
In each case the changelog presents the content as a raw HTML block rather
than the visible Markdown headings the ceremony requires. Direct probes
reproduced acceptance for all four forms. The new regression matrix covers
only the four type-1 tag names, so it remains green for these triggers.

## Smells

None.

## Nitpicks

None.

## Not found

The exact pass 3 examples are remediated. Mixed-case type-1 openings, opening
attributes, matching close tags, hidden whole releases, and hidden duplicate
headings behave as intended for `script`, `pre`, `style`, and `textarea`.

No additional defects were found in tag validation, required section order,
empty and placeholder rejection, check and render immutability, family
separation, pre-publication validation ordering, exact byte comparison before
GitHub release creation, release final approval, or post-publication body
comparison. Generated adapters remain synchronized and the ceremony remains
explicitly invoked.

The five focused workflow tests passed. The generated-skill drift check,
prose check, and `git diff --check` also passed.
