# Current Sprint, S51

**Milestone**: M16 Document automation.

**Goal**: close M16 with mail merge, document comparison, and watermarks, then
ship the completed milestone with the community-requested ordered-body and
complete-layout reader surfaces, including traceable glyph provenance for
viewer and editor integrations. Keep each operation package-preserving and
bounded to its declared document stories. Establish reviewed release notes as
the permanent publication contract before the incubating 0.4.0 and stable
0.8.0 releases through a reusable `/release-notes` ceremony.

## Spec references

- `docs/hld/03-architecture.md`, for WordprocessingML ownership, field and
  template evaluation boundaries, typed story traversal, and atomic facade
  mutation.
- `docs/hld/04-opc-and-packaging.md`, for staged package updates, relationship
  integrity, media allocation, raw XML preservation, and fail-closed commits.
- `docs/hld/08-rendering-spec.md`, for accepted and tracked revision views,
  deterministic pagination, headers, and page-level visual output.
- `docs/hld/10-bindings-spec.md`, for the native Rust revision surface and the
  unchanged Python, WASM, and CLI compatibility boundaries.
- `docs/hld/12-testing-strategy.md`, for readable in-code fixtures, regression
  tests, deterministic golden rendering, and the hash-harness gate.
- `docs/hld/14-development-backlog.md`, for the M16 end gate and the exact
  S51 story scope, dependencies, community contribution, and release gates.
- `docs/hld/15-build-and-toolchain.md`, for the two release families, version
  carriers, publication allowlists, and reviewed release SHA.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-166 | Mail merge | M | in-progress | codex |
| F-167 | Document comparison | L | in-progress | codex |
| F-168 | Watermarks | S | in-progress | codex |
| F-X032 | Expose complete Word layout results | S | pending | - |
| F-X033 | Integrate PR 36 ordered body items | S | pending | - |
| F-X034 | Reviewed release notes for every release | S | pending | - |
| F-X035 | Tag rpptx-v0.4.0 | S | pending | - |
| F-X036 | Tag v0.8.0 | S | pending | - |
| F-X037 | Trace Word glyphs to source paragraphs | M | pending | - |

## Sequencing note

Rows are listed in dependency order, not F-ID order.

F-166 comes first because it composes the existing field and
structural-template evaluators. F-167 follows as the flagship comparison path
over body text, tables, and lists. F-168 completes the milestone through the
independent header and renderer path. F-X037 then carries exact Word source
provenance through the layout engine. F-X032 opens that combined layout result
to external renderers without shipping an intermediate public signature.
F-X033 integrates PR 36 only after the product wave is present on its sprint
base, so current GitHub CI sees the real result and Pedro Assumpcao's commit
remains in the merge record. F-X034 establishes reviewed release notes before
either release tag. F-X035 publishes the missing incubating chart dependency
and low-level provenance types at 0.4.0. F-X036 can prepare and publish stable
0.8.0 only after that registry dependency is verified.

## Definition of done for this sprint

- A fixture record set drives `MERGEFIELD` into one document per record and
  one section per record, with absent fields rendered empty.
- Comparing a document with its edited copy produces tracked revisions that,
  when accepted, reproduce the edited copy exactly.
- Formatting-only comparison differences are reported as diagnostics rather
  than revisions.
- Text and image watermarks round-trip through header `w:pict` shapes and render
  behind body text on every page.
- Third-party renderers can obtain positioned pages together with every font
  used by shaping, including caller-provided fonts on WASM.
- Every attributed glyph run resolves to one exact Word story path and Unicode
  scalar range, while generated text remains truthfully unattributed.
- Direct body paragraphs, tables, content controls, and unsupported XML are
  readable once each in exact source order through the PR 36 API.
- Every release uses reviewed notes with highlights, user-visible changes,
  compatibility guidance, and contributor credit.
- The incubating 0.4.0 and stable 0.8.0 families publish only after their
  separate reviewed-SHA approvals and registry verification.
- Every operation preserves unrelated package parts, relationships, schema
  order, and the unchanged hash-harness baseline unless a reviewed plan
  declares an intentional delta.
