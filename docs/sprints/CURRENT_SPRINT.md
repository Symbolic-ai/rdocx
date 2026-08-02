# Current Sprint, S21

**Milestone**: M9 Inheritance resolver.

**Goal**: Freeze a correct `ResolvedSlide` boundary by flattening slide,
layout, and master content into final draw order and collapsing every inherited
or theme-derived value. Prove the contract on the corpus and publish it as the
documented handoff to the M10 render track without publishing any crate.

## Spec references

- `docs/hld/05-drawingml-model.md`, for the typed colour, theme, fill, shape,
  and text inputs and their concrete resolved forms.
- `docs/hld/06-presentationml-model.md`, for part relationships, document-order
  shape trees, placeholder identity, and latent-placeholder inputs.
- `docs/hld/07-inheritance-and-resolution.md`, for the `ResolvedSlide` output,
  four-pass draw order, suppression rules, and concrete-value contract.
- `docs/hld/08-rendering-spec.md`, for the renderer-facing seam and
  backend-neutral geometry, paint, and content types.
- `docs/hld/12-testing-strategy.md`, for the corpus, exact 40-colour table,
  differential decks, and one-time manual review evidence.
- `docs/hld/14-development-backlog.md`, for the three F-ID dependencies, test
  gates, and M9 end gate.
- `docs/hld/15-build-and-toolchain.md`, for keeping PowerPoint development
  crates at version 0.0.0 with publication disabled while publishing the
  contract only to the render track.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-086 | Draw order and the flattener | L | done | - |
| F-087 | ResolvedSlide contract | M | done | - |
| F-088 | Visual differential tests | M | done | - |

## Sequencing note

F-086 runs first because F-087 depends on F-082 through F-086. F-087 then
freezes the complete renderer-facing output contract. F-088 consumes that
frozen contract and supplies the differential and manual-review evidence for
the milestone gate. These dependencies leave no safe implementation
parallelism across the three F-IDs.

## Definition of done for this sprint

- The flattener emits the background, allowed master non-placeholder shapes,
  allowed layout non-placeholder shapes, and slide shape tree in final document
  order.
- `showMasterSp`, placeholder suppression, and latent date, footer, and slide
  number placeholders behave as documented. No prompt text is drawn, and a
  master logo appears exactly once.
- The public, documented `ResolvedSlide` type set is frozen. It contains no
  PresentationML or DrawingML model types and no unresolved theme references.
- Every corpus slide resolves without unresolved theme references.
- The exact 40-case colour table and differential decks pass and receive their
  required one-time manual review.
- The M9 contract is documented and published to the M10 render track.
- The full workspace gate passes with all 28 deterministic hashes unchanged.
  Every PowerPoint development crate remains version 0.0.0 and unpublished,
  with no crates.io publication.
