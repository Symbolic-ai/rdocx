# Current Sprint, S46

**Milestone**: M14 Word collaboration layer.

**Goal**: open the collaboration layer at both ends. Comments are the most
requested missing API in this space, content controls are the foundation of
document assembly, and bookmarks provide the targets that later field
evaluation needs.

## Spec references

- `docs/hld/02-scope-and-non-goals.md`, for the post-v1 preservation baseline
  and the roadmap boundary that moves collaboration features into scope.
- `docs/hld/03-architecture.md`, for the `rdocx-oxml` model boundary, the
  `rdocx` facade ownership rule, and the Word-specific pagination layer.
- `docs/hld/04-opc-and-packaging.md`, for relationship handling, deterministic
  part naming, package integrity, and verbatim preservation of unmodelled
  parts.
- `docs/hld/08-rendering-spec.md`, for the Word pagination contract that
  `PAGEREF` must query without creating a second layout path.
- `docs/hld/12-testing-strategy.md`, for round-trip and regression test
  categories, deterministic fixtures, and the byte-preservation rules.
- `docs/hld/14-development-backlog.md`, for the M14 boundary, the five story
  definitions, their dependency pairs, and their exact test gates.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-147 | Comment model and part | M | in-progress | codex |
| F-152 | Content control model | L | in-progress | codex |
| F-154 | Bookmarks and cross-references | M | in-progress | codex |
| F-148 | Comment API | M | in-progress | codex |
| F-153 | Content control binding | M | in-progress | codex |

## Sequencing note

Rows are listed in dependency order, not by F-ID.

F-147, F-152, and F-154 are independent roots and can proceed in parallel.
F-148 follows F-147 because the public comment and reply API needs the comment
part and body anchors. F-153 follows F-152 because value and custom XML binding
must operate on the typed five-level content-control model. F-154 is in this
sprint so F-161 can resolve `REF` and `PAGEREF` two sprints later.

## Definition of done for this sprint

- Three comments, including one spanning two paragraphs, round-trip with their
  anchors in place and byte-identical producer content where unmodelled.
- The public comment API can add a ranged comment, reply to it, resolve it, and
  remove it, and Word opens the resulting thread intact.
- Content controls at block, row, cell, paragraph, and run level survive
  round-trip and report their tag, alias, id, and type.
- Content controls can be read and written by tag or alias, map binding updates
  display text, and a bound custom XML part updates with the same value.
- A bookmark inserted over a range is listed with readable text, and `REF` and
  `PAGEREF` resolve to the correct content and page after pagination.
