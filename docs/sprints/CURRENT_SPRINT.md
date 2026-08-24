# Current Sprint, S55

**Milestone**: M18 Format breadth.

**Goal**: add the two remaining inbound formats in M18. Import the practical
HTML and CSS subset produced by browser copy-paste and CMS exports, and read
OpenDocument Text through a structural boundary pinned against LibreOffice.

## Spec references

- `docs/hld/03-architecture.md`, for the Word facade's single typed document
  ownership tree and format-boundary dependency rules.
- `docs/hld/04-opc-and-packaging.md`, for bounded archive handling, normalized
  part paths, media ownership, and deterministic package behavior.
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
| F-178 | HTML import | L | pending | - |
| F-179 | ODT reader | L | pending | - |

## Sequencing note

Rows are listed in dependency order, not F-ID order.

F-178 and F-179 are independent and may proceed in parallel. Both project into
the existing Word document tree, but HTML owns browser and CMS markup plus CSS
diagnostics, while ODT owns its archive and LibreOffice differential boundary.
Neither story depends on the outbound ODT writer planned for S56.

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
- Both story gates, the full workspace gate, package checks, and the
  deterministic hash harness pass without an unexplained baseline change.
