# Current Sprint, S04

**Milestone**: M2 Shared infrastructure extraction.

**Goal**: Make the OPC package layer format-neutral and prove it against a
code-built PowerPoint package. Defer the guarded Word facade moves until the
real shared crates may be published after PowerPoint development, preserve
existing rdocx behaviour, and keep every `oxml-*` and `rpptx*` development
crate unpublished.

## Spec references

- `docs/hld/03-architecture.md`, for format-family dependency direction,
  versioning, and the facade boundary retained by the published Word crates.
- `docs/hld/04-opc-and-packaging.md`, for generic package constructors,
  relationship and content-type constants, part-name resolution, deterministic
  saves, and package integrity.
- `docs/hld/11-migration-plan.md`, for the zero-call-site facade pattern and
  the required order of the `oxml-core`, `oxml-opc`, and compatibility-shim
  steps.
- `docs/hld/12-testing-strategy.md`, for the PowerPoint-shaped OPC fixture,
  constant-table assertions, zip-slip cases, and unchanged hash gate.
- `docs/hld/14-development-backlog.md`, for the F-015, F-016, and F-018 through
  F-022 contracts, dependencies, and story test gates.
- `docs/hld/15-build-and-toolchain.md`, for package verification and the rule
  that development crates stay at 0.0.0 and unpublished until PowerPoint work
  is complete.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-018 | Create oxml-opc | M | in-progress | codex |
| F-019 | PresentationML relationship and content types | S | in-progress | codex |
| F-020 | oxml-opc reads a pptx | M | pending | - |
| F-021 | Zip-slip hardening tests | S | pending | - |
| F-022 | rdocx-opc deprecation shim | S | pending | - |
| F-015 | rdocx-oxml becomes a facade | S | pending | - |
| F-016 | Length re-export | S | pending | - |

## Sequencing note

Rows are listed in dependency order, not F-ID order.

F-018 establishes `oxml-opc`. F-019 and F-021 depend on that crate, while F-020
additionally depends on F-019. F-015, F-016, and F-022 are deferred cutover
stories. They must not begin until PowerPoint development is complete and the
real shared crates have an approved publication path. The rdocx 0.5.0 boundary
alone cannot satisfy later package dry-runs because the registry holds only
dependency-free 0.0.0 placeholders.

The recorded three-sprint velocity variance requires replanning the remaining
milestones before implementation begins. Opening S04 creates the delivery
boundary and does not waive that escalation.

## Definition of done for this sprint

- `oxml-opc` owns the format-neutral package implementation and its eleven
  moved tests, with generic main-part and minimal-content-type constructors.
- Relationship and content-type constants cover the package, shared document
  properties, and PresentationML cases with uniqueness and shape tests.
- A code-built PowerPoint package resolves `/ppt/presentation.xml` and a slide
  layout target through the required parent-directory traversal.
- Root-escaping and absolute zip entries are normalized or rejected by direct
  hardening tests.
- F-015, F-016, and F-022 are carried into the deferred shared-crate cutover,
  with their approved zero-churn and compatibility contracts intact.
- The full workspace, package, supply-chain, and hash gates pass with all
  existing hash entries unchanged.
- No `oxml-*` or `rpptx*` development crate is published.
