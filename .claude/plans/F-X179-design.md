# F-X179, Correct multi-paragraph comment threads from PR 271

**Status**: completed
**Sprint**: S90
**Size**: S
**Depends on**: none

## Problem

`crates/rdocx/src/comments.rs:975` selects a comment extension by the first
paragraph id, while commentEx owns the last paragraph id. Multiline text is
written as one paragraph at `crates/rdocx/src/comments.rs:1312`.
Issue 270 reports lost reply parents, resolution state and paragraph structure.

## Spec reference

- `docs/hld/03-architecture.md`, "Facade conventions", Word comment mutation.
- `docs/hld/04-opc-and-packaging.md`, "Relationship types", comments relationship and preservation contract.
- `docs/hld/12-testing-strategy.md`, regression and round-trip categories.

## Approach

Adopt PR 271 from hadim at e22641a8f20a1c31d905a8c1b83171f548d2e230.
Use last_para_id for a comment's own extension and all paragraph ids for
legacy parent lookup, descendant removal and fragment closure. Split authored
text on newline, accepting CRLF as one separator and retaining empty lines.
Allocate unique ids to every authored paragraph and missing final paragraph ids.
Add focused regressions to the existing comments test module where contributed
coverage does not prove the complete issue. No new public signatures or modules.
Refresh archive observations after the final code and test inventory is known.

## Rejected alternatives

Direct merge into main would bypass sprint closure. First-paragraph fallback
for a comment's own metadata would retain the reported nonstandard semantics.

## Test plan

**Test gate**: regression, threads_and_resolved_state_read_through_the_last_paragraph.

| Category | Test | Asserts |
|---|---|---|
| regression | threads_and_resolved_state_read_through_the_last_paragraph | Each reported read failure and single-paragraph controls |
| regression | reply_and_resolve_write_the_parent_last_paragraph | Last-paragraph parent and done rows survive reopen |
| regression | each_line_of_a_comment_text_becomes_one_paragraph | Paragraph count, unique ids and reply text round trip |
| regression | removing_a_parent_of_several_paragraphs_removes_its_reply | Descendant closure |
| regression | a_windows_line_ending_starts_a_paragraph_without_a_carriage_return | CRLF normalization |
| round-trip | Additional focused tests in comments::tests | All authoring entry points, empty lines, missing final ids, fragment closure and producer XML |

## HLD impact

- `docs/hld/03-architecture.md`

## Risk routing

- Serialization behavior: read packaging and preservation references. Check last paragraph ordering, fixed prefixes and verbatim unsupported subtree preservation during mutation and reopen.
- Published public behavior: signatures unchanged, corrective pre-1.0 behavior. Run local patched cargo publish dry runs for rdocx and rdocx-cli and verify their archive size observations.
- New workflow records: user explicitly approved design, review and progress files. No new source file, trait, generic, crate or module.

## Hash harness

Unchanged, all 49 deterministic entries. Comment metadata and comment paragraphs
do not alter the existing render or document baseline cases.

## Implementation checklist

- [x] Prove the regression gate fails before implementation.
- [x] Apply and inspect the contributed implementation.
- [x] Prove every Issue 270 criterion and the declared edge cases.
- [x] Pass scoped verification, risk riders and zero-finding microscope.
- [x] Update HLD and delivery records with contributor and closure evidence.

## Open questions

None. User approved new workflow records. Issue 264 is excluded.
