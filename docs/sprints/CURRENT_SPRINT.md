# Current Sprint, S41

**Milestone**: X Cross-cutting.

**Goal**: land the parts of the external PR 2 rendering contribution that
current `main` still lacks, rebuilt against the anchor architecture that
superseded the contributor's own. Two footnote placement defects are fixed
first, since they are contained and independent of the drawing work. Anchored
drawings then gain a real wrap and alignment model, and body text learns to flow
around them.

## Spec references

- `docs/hld/03-architecture.md`, for the `rdocx-layout` flow model boundary, and
  for the rule that the `wp:` inline and anchor code in `rdocx-oxml/drawing.rs`
  is Word-only and stays where it is rather than migrating to `oxml-drawing`.
- `docs/hld/05-drawingml-model.md`, "Do not touch the Word path", which is what
  keeps F-X015 and F-X016 inside `rdocx-oxml` and out of the shared DrawingML
  crates.
- `docs/hld/12-testing-strategy.md`, for the test taxonomy each story picks its
  gate from, and for the hash harness rule that an intentional delta lands as
  its own labelled commit with the expected change stated.
- `docs/hld/14-development-backlog.md`, for the F-X013 through F-X016 scope and
  acceptance gates.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-X013a | Footnote line advance | S | done | claude |
| F-X013b | Footnote reservation and splitting | L | done | claude |
| F-X013c | Endnotes at the document end | M | done | claude |
| F-X014 | Kashida justification values | S | done | claude |
| F-X015 | Anchored drawing wrap and alignment model | M | done | claude |
| F-X016 | Floating drawing placement and text wrapping | L | pending | - |

## Sequencing note

Rows are listed in dependency order, not F-ID order.

The three F-X013 children run in order. F-X013a is a contained drawing-position
fix that stands alone. F-X013b builds the shared note height map and the
reservation that consumes it, which is what F-X013c then needs in order to route
one of the two note streams somewhere else. Attempting F-X013c first would mean
separating the streams with no place to put the endnote stream.

F-X014 is independent of everything else in the wave and can land at any point.
It is grouped here because it comes from the same external contribution.

F-X015 must precede F-X016. F-X015 deliberately changes no placement and no
rendering, which is what lets its harness result stay flat and prove the story
is model-only. F-X016 then owns the entire rendering delta for wrapped drawings
in one reviewable commit. Splitting them the other way, or merging them, would
mix a model addition with a layout change and leave a delta nobody can attribute
to a cause.

F-X016 is sized L and touches line breaking, which every paragraph in every
baseline flows through. If it exceeds twice its estimate it splits into F-X016a
for alignment-based placement and F-X016b for text wrapping, per the escalation
rule in `.claude/WORKFLOW.md`.

## Definition of done for this sprint

- A footnote assembled from several runs renders its segments at strictly
  increasing x, and a page whose body fills the text area leaves the reserved
  footnote area clear.
- The three kashida justification values parse to the justified variant, and an
  unknown justification string is still rejected.
- Wrap mode, the four text distances and both alignment axes round-trip through
  `CT_Anchor` and reach `AnchoredDrawing`, with the hash harness unchanged.
- Body text flows around a `wrapSquare` drawing and clears a `wrapTopAndBottom`
  one, while every baseline without a wrapped drawing stays byte-identical.
- Every harness delta in the sprint is stated and justified in the commit that
  causes it. No delta is folded into an unrelated change.
