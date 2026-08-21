# Current Sprint, S51

**Milestone**: M16 Document automation.

**Goal**: close M16 with mail merge, document comparison, and watermarks, the
three automation capabilities that depend on the field, template, revision,
header, and rendering foundations already in place. Keep each operation
package-preserving and bounded to its declared document stories.

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
  F-166, F-167, and F-168 scope, dependencies, and test gates.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-166 | Mail merge | M | pending | - |
| F-167 | Document comparison | L | pending | - |
| F-168 | Watermarks | S | pending | - |

## Sequencing note

Rows are listed in dependency order, not F-ID order.

All three stories have their dependencies satisfied before S51 and have no
dependency on another row in this wave. F-166 comes first because it composes
the existing field and structural-template evaluators. F-167 follows as the
flagship comparison path over body text, tables, and lists. F-168 completes the
milestone through the independent header and renderer path.

## Definition of done for this sprint

- A fixture record set drives `MERGEFIELD` into one document per record and
  one section per record, with absent fields rendered empty.
- Comparing a document with its edited copy produces tracked revisions that,
  when accepted, reproduce the edited copy exactly.
- Formatting-only comparison differences are reported as diagnostics rather
  than revisions.
- Text and image watermarks round-trip through header `w:pict` shapes and render
  behind body text on every page.
- Every operation preserves unrelated package parts, relationships, schema
  order, and the unchanged hash-harness baseline unless a reviewed plan
  declares an intentional delta.
