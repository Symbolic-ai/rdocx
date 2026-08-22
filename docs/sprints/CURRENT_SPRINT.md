# Current Sprint, S53

**Milestone**: M17 Security and compliance.

**Goal**: finish the security and compliance milestone with documents that an
enterprise or public body can accept. Add standards-compatible package signing,
semantic tagged PDF, PDF/A-2b and PDF/A-3b output, and destructive redaction
that removes sensitive content from every relevant package part.

## Spec references

- `docs/hld/03-architecture.md`, for keeping package security, document
  semantics, PDF generation, and embedded chart workbooks in their existing
  ownership layers.
- `docs/hld/04-opc-and-packaging.md`, for signature relationships, canonical
  package bytes, staged mutation, content types, and preservation of unrelated
  package parts.
- `docs/hld/08-rendering-spec.md`, for carrying Word semantics into marked PDF
  content, structure trees, font resources, metadata, and backend output.
- `docs/hld/09-charts-spec.md`, for removing redacted values from embedded
  chart workbooks without leaving cached or modeled traces.
- `docs/hld/10-bindings-spec.md`, for additive native signing, PDF conformance,
  and redaction APIs with unchanged Python, WASM, and CLI boundaries unless a
  feature design explicitly expands them.
- `docs/hld/12-testing-strategy.md`, for readable regression fixtures,
  round-trip and external-oracle evidence, raw-package scans, accessibility
  structure checks, and full workspace gates.
- `docs/hld/14-development-backlog.md`, for the M17 goal, the four S53 stories,
  their dependencies, and their exact test gates.
- `docs/hld/15-build-and-toolchain.md`, for feature isolation, dependency and
  licence policy, WASM checks, packaging, and publication-safe verification.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-172 | Digital signature creation | M | pending | - |
| F-173 | Tagged PDF structure tree | L | pending | - |
| F-175 | Redaction | M | pending | - |
| F-174 | PDF/A conformance | M | pending | - |

## Sequencing note

Rows are listed in dependency order, not F-ID order.

F-172 builds on the signature verification and authenticated coverage model
completed in F-171. F-173 is an independent rendering root and establishes the
semantic PDF structure that F-174 must retain while adding PDF/A output intent,
metadata, and prohibited-feature checks. F-175 is also ready because F-147 and
F-149 are complete. It may proceed independently, but its body, comments,
revisions, properties, and chart-workbook mutation paths must remain staged and
failure safe.

## Definition of done for this sprint

- A package signed here with a supplied key and certificate verifies through
  the F-171 verifier and in the pinned Microsoft Word oracle.
- Signature creation covers the complete declared package graph, uses strict
  supported algorithms and schema order, and leaves the live package unchanged
  on key, certificate, canonicalization, relationship, or write failure.
- A rendered PDF emits `/StructTreeRoot`, marked content, heading levels, list
  nesting, table headers, and alternate text that match the source semantics.
- Tagged output preserves visible PDF and raster results unless an approved
  design declares and reviews a deterministic baseline change.
- PDF/A-2b and PDF/A-3b output embeds the required output intent and metadata,
  rejects prohibited features, and passes the declared conformance checker.
- PDF/A output retains the F-173 structure tree and continues to satisfy the
  PDF/UA structure check required by the M17 gate.
- Redaction removes selected text and every recoverable trace from body,
  comments, revisions, document metadata, chart caches, and embedded chart
  workbooks rather than covering content visually.
- Redaction failure is atomic, unrelated parts and unmodelled XML remain
  byte-preserved, and a raw ZIP scan cannot find the redacted value.
- New cryptographic, PDF, or conformance dependencies satisfy supply-chain,
  feature-isolation, default-off, WASM, package-size, and licence gates.
- The full workspace gate passes with all deterministic hashes unchanged unless
  an approved design declares a reviewed delta.
- The M17 end gate passes: an encrypted document opens with its password, a
  signed document verifies, and a rendered PDF passes a PDF/UA structure check.
