# F-248, all, pass 2

**Reviewed**: Working-tree implementation diff from `0dfb0144688b6005db848fb58bbbf1cc182e67cd` on `work/f-248-codex`. All 19 changed implementation and HLD files were reviewed against the approved plan and its eight cited HLD files. The implementation diff contains 4,739 insertions and 211 deletions. The changed files currently contain 114,864 lines. The pass-1 review artifact was excluded from these scope counts.
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Not found

- Correctness: The pass-1 ancestor fallback defect is fixed. Unstarted ancestor placeholders now use the base level start, and the deeper-before-ancestor regression proves marker text and result-local context agree. Concrete-instance counters, starts, overrides, restarts, section and table traversal, `numId` zero, TOC prefixes, and REF projections match the reviewed contract.
- Contract: Atomic style-numbering link and unlink operations validate both graphs, require the exact reciprocal tuple, serialize the staged candidate, and publish once. The public API and intentional shared-definition divergence are documented in the cited HLD files.
- Panics: Added production `expect` and `unreachable` sites are dominated by complete graph validation, checked source identity construction, or closed enum selection. New indexing and arithmetic do not expose an untrusted-input panic path.
- OOXML: The `w:numPr` overlay preserves extended leaves, producer attributes, namespace aliases, revision markers, and schema child order. Synthesized inherited namespace declarations are escaped before raw replay, including ampersands. The style-link transaction retains these carriers while changing or clearing only modeled numbering values.
- Tests: The pinned `three_level_style_linked_numbering_matches_pinned_word` differential passed. The complete `rdocx-layout` suite passed 275 unit tests and one documentation test. Focused validation also passed the replacement-start regression, three numbered REF tests, eight `numPr` tests, and two atomic style-link tests.
- Structure: The implementation remains in the existing document, layout, numbering, and field owners. It adds no crate, module, feature flag, trait, generic abstraction, forwarding wrapper, or speculative extension point.
