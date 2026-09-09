# F-248, all, pass 1

**Reviewed**: Working-tree diff from `0dfb0144688b6005db848fb58bbbf1cc182e67cd` on `work/f-248-codex`. All 19 changed files were inspected against the approved plan and its eight cited HLD files. The diff contains 4,733 insertions and 211 deletions. The changed files currently contain 114,858 lines.
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

1. `crates/rdocx-layout/src/style_resolver.rs:635` takes an unstarted ancestor placeholder's fallback start from the effective replacement level. The current-level counter at line 395 and the result-local context at line 445 correctly take that value from the base level, matching the documented contract that replacement levels supply formatting while base levels supply starts. For example, with base level 0 start `3`, a replacement level 0 start `9`, and a first level 1 paragraph whose text is `%1.%2.`, marker rendering substitutes `9` for `%1` while `number_context` records `3`. This emits the wrong visible marker and can propagate the same wrong value into TOC and REF output. The existing `replacement_level_start_is_ignored_like_word` test covers only the replacement level as the current level, so it does not reach this ancestor path. Use the base level's start in `format_lvl_text` and add a regression where a deeper level is visited before its replaced ancestor.

## Smells

None.

## Nitpicks

None.

## Not found

- Correctness: The earlier raw-only instruction and note-only REF preflight failures are fixed. Focused numbered REF tests passed for raw-only instructions, revision-wrapped forward references, and note-only references.
- Contract: Atomic style-numbering link and unlink staging, reciprocal graph validation, exact tuple checks, and publication after serialization match the plan apart from the counter defect above.
- Panics: New production assertions and unreachable branches are guarded by preceding validation or closed enum selection.
- OOXML: The `w:numPr` overlay retains producer attributes, namespaces, extended leaves, revision markers, and schema child order. The latest inherited namespace fix escapes synthesized declarations before raw replay. Eight focused `numPr` tests passed.
- Tests: The prior full `rdocx-layout` run passed 273 unit tests and one documentation test. Focused `rdocx` style-link tests passed. The pinned Word source and render oracle records are deterministic and documented. The missing ancestor replacement-start case is identified in the defect above.
- Structure: The change uses the existing document, layout, numbering, and field owners. It adds no crate, module, feature flag, trait, generic abstraction, or forwarding wrapper.
