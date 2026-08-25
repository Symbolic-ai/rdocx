# Current Sprint, S56

**Milestone**: M18 Format breadth, plus cross-cutting contribution and release
work.

**Goal**: close M18 by adding the outbound formats that reuse the document and
layout models already established. Write the supported ODT fidelity boundary,
export reflowable EPUB 3 from document structure, and export searchable SVG
pages from the shared `PageFrame` contract with explicit diagnostics for every
lossy conversion. Audit PRs 47 through 52 against the current reader and
preservation contracts, then publish the stable family at v0.10.0 with reviewed
compatibility notes and authenticated contributor credit.

## Spec references

- `docs/hld/03-architecture.md`, for the single Word document ownership tree,
  format-neutral layout boundary, and dependency direction for new exporters.
- `docs/hld/04-opc-and-packaging.md`, for deterministic archive output, safe
  package paths, media ownership, and the boundary between ODT ZIP and OPC.
- `docs/hld/08-rendering-spec.md`, for the shared `LayoutResult`, `PageFrame`,
  text, font, image, background, and recursive element contracts consumed by
  SVG output.
- `docs/hld/10-bindings-spec.md`, for additive native facade methods and the
  rule that Python, WASM, and CLI surfaces change only when explicitly scoped.
- `docs/hld/12-testing-strategy.md`, for source-built fixtures, structural
  round trips, named regressions, pixel goldens, and external oracle discipline.
- `docs/hld/14-development-backlog.md`, for the M18 fidelity rule and the exact
  ODT writer, EPUB export, and SVG page export scopes and gates.
- `docs/hld/15-build-and-toolchain.md`, for dependency policy, pinned external
  tools, package checks, release-family publication, and the unchanged
  deterministic verification contract.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-180 | ODT writer | L | done | |
| F-181 | EPUB export | M | done | |
| F-182 | SVG page export | M | done | |
| F-X054 | Integrate PRs 47 through 52 | L | done | |
| F-X055 | Tag v0.10.0 | S | in-progress | codex |

## Sequencing note

Rows are listed in dependency order, not F-ID order.

F-180 depends on the F-179 reader completed in S55, so its round-trip boundary
is available at sprint entry. F-181, F-182, and F-X054 have no implementation
dependency on F-180 or on each other. The four stories may proceed in parallel
while keeping exclusive ownership of ODT packaging, EPUB structure, stable SVG
entry points, and the contributed reader surface respectively. F-X055 runs
last, after every implementation and contribution record is complete.

## Definition of done for this sprint

- ODT output preserves the declared text, formatting, table, list, and image
  fidelity when read back through the F-179 importer.
- Unsupported ODT properties and content produce stable diagnostics naming
  what was dropped while supported siblings remain available.
- Generated EPUB 3 output passes the pinned `epubcheck` gate, and its spine and
  navigation order match the document outline.
- SVG output retains searchable text and rasterises to the same pixels as the
  PNG backend at the same dpi within the reviewed tolerance.
- SVG traverses every recursive `PageFrame` element, preserves page geometry,
  fonts, images, backgrounds, links, and clipping at the declared boundary,
  and diagnoses any unsupported effect without dropping supported siblings.
- The M18 end gate proves each format at its declared fidelity level and every
  lossy conversion records a diagnostic naming what it dropped.
- PRs 47 through 52 are audited against current preservation, namespace,
  ordering, error, and public API contracts. Every retained outcome has a named
  regression, a direct pull-request link, and specific authenticated credit to
  `@pedroassumpcao`.
- Stable version selection follows the recorded beta policy. Before 1.0,
  public API additions or incompatibilities and internal-only new functionality
  take a minor release, while repair-only compatible changes take a patch
  release. At and after 1.0, incompatible public API changes take a major
  release, compatible additions take a minor release, and fixes take a patch
  release. S56 therefore prepares v0.10.0 and records the PR 51 source change
  explicitly without claiming the stable 1.0 boundary.
- The v0.10.0 release notes cover only the stable family, name every addition,
  fix, compatibility action, and included external record, and credit
  `@emptinessform` and `@pedroassumpcao` for their specific included outcomes.
- After publication verifies, PRs 47 through 52 receive release-bound
  maintainer comments and close with their direct or hardened-equivalent status
  stated accurately.
- All five story gates, the full workspace gate, package checks, and the
  deterministic hash harness pass without an unexplained baseline change.
