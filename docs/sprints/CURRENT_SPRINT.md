# Current Sprint, S55

**Milestone**: M18 Format breadth.

**Goal**: add the two remaining inbound formats in M18 while closing the
residual v0.9 editor work reported by the community. Import the practical HTML
and CSS subset produced by browser copy-paste and CMS exports, read
OpenDocument Text through a structural boundary pinned against LibreOffice,
restore the reviewed interactive relayout budget, and document the recursive
layout traversal required by tagged content.

## Spec references

- `docs/hld/03-architecture.md`, for the Word facade's single typed document
  ownership tree and format-boundary dependency rules.
- `docs/hld/04-opc-and-packaging.md`, for bounded archive handling, normalized
  part paths, media ownership, and deterministic package behavior.
- `docs/hld/08-rendering-spec.md`, for exact reusable layout context, bounded
  transactional caches, restart-safe pagination, semantic marked content, and
  warm versus cold equality.
- `docs/hld/10-bindings-spec.md`, for additive native Word entry points,
  conversion results with diagnostics, and unchanged binding boundaries.
- `docs/hld/12-testing-strategy.md`, for named regression fixtures,
  differential oracle discipline, structural comparison, and source-built
  test inputs.
- `docs/hld/14-development-backlog.md`, for the M18 fidelity rule and the exact
  HTML import and ODT reader scopes and gates.
- `docs/hld/15-build-and-toolchain.md`, for dependency policy, package checks,
  and the unchanged deterministic verification contract.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-178 | HTML import | L | in-progress | codex |
| F-179 | ODT reader | L | in-progress | codex |
| F-X052 | Restore interactive relayout performance | L | in-progress | codex |
| F-X053 | Complete layout migration and contribution records | S | pending | - |

## Sequencing note

Rows are listed in dependency order, not F-ID order.

F-178, F-179, and F-X052 are independent and may proceed in parallel. The two
format stories project into the existing Word document tree, but HTML owns
browser and CMS markup plus CSS diagnostics, while ODT owns its archive and
LibreOffice differential boundary. Neither story depends on the outbound ODT
writer planned for S56. F-X052 owns only the existing reusable layout path and
must preserve its correctness and memory contracts. F-X053 runs after F-X052
because the final Issue 46 response needs both the performance implementation
and the migration correction.

## Definition of done for this sprint

- HTML import projects the supported copy-paste and CMS subset into paragraphs,
  runs, tables, and lists in source order.
- Unsupported CSS produces stable diagnostics naming what was dropped while
  supported sibling content remains available.
- ODT input projects its supported text, formatting, tables, lists, and images
  into the normal Word document tree without a second facade model.
- A source-built ODT converted here matches the pinned LibreOffice conversion
  structurally at the declared differential boundary.
- Malformed markup, unsafe archive paths, unsupported content, and retained
  conversion output are bounded and fail or diagnose without partial state.
- A body-only edit and a checked restored-document transfer avoid
  whole-document debug serialization and deep copies of unchanged pages while
  retaining exact warm versus cold output, invalidation, and memory bounds.
- Interleaved release measurements for the Issue 46 load, typing, undo, and
  table-mutation workload are within 1.25 times the reviewed reference on the
  same machine for native and bundled-fallback paths.
- The v0.9.0 compatibility record tells external layout backends to recurse
  through `MarkedContent::children` or use `oxml_layout::walk`. Issue 44,
  Issue 46, and PR 45 close with implementation evidence and retained
  contributor credit, while Issues 39 and 42 remain closed without duplicate
  scope.
- All four story gates, the full workspace gate, package checks, and the
  deterministic hash harness pass without an unexplained baseline change.
