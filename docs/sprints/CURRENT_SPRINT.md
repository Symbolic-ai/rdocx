# Current Sprint, S52

**Milestone**: M17 Security and compliance.

**Goal**: open the files that currently cannot be opened at all, and close the
new community-reported rendering and interactive-layout gaps. Add
standards-compatible read and write support for OOXML agile encryption, verify
digital signatures over their complete declared part sets, remove duplicated
glyphs, prove headers and footers through PDF output, and extend bounded
relayout reuse without weakening correctness.

## Spec references

- `docs/hld/03-architecture.md`, for keeping package security ownership below
  the native Word facade and preserving the existing document model boundary.
- `docs/hld/04-opc-and-packaging.md`, for package relationships, part naming,
  integrity validation, and staged failure-safe package replacement.
- `docs/hld/08-rendering-spec.md`, for shaping ownership, header and footer
  placement, result payloads, bounded caches, and pagination state.
- `docs/hld/10-bindings-spec.md`, for additive native Word APIs and the
  unchanged Python, WASM, and CLI compatibility boundaries.
- `docs/hld/12-testing-strategy.md`, for readable regression fixtures,
  round-trip evidence, external reference metadata, and full workspace gates.
- `docs/hld/14-development-backlog.md`, for the M17 security goal and the exact
  encryption and signature verification stories, dependencies, and gates.
- `docs/hld/15-build-and-toolchain.md`, for dependency policy, supply-chain
  review, feature graphs, packaging, and publication-safe verification.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-169 | Agile encryption, read | L | in-progress | codex |
| F-171 | Digital signature verification | L | in-progress | codex |
| F-X039 | Share layout payloads and transfer reusable engines | M | in-progress | codex |
| F-X041 | Remove duplicated glyphs at break opportunities | M | in-progress | codex |
| F-X042 | Prove headers and footers in PDF output | S | in-progress | codex |
| F-170 | Agile encryption, write | M | in-progress | codex |
| F-X040 | Restart pagination and cache table blocks | L | in-progress | codex |
| F-X043 | Reuse bundled-fallback caller-font layouts | M | in-progress | codex |
| F-X044 | Scale paragraph-cache lookup for editors | M | pending | - |
| F-X045 | Cache headers and footers transactionally | M | pending | - |
| F-X046 | Reuse substituted pages exactly | S | pending | - |
| F-X047 | Attribute empty Word paragraphs | S | pending | - |

## Sequencing note

Rows are listed in dependency order, not F-ID order.

F-169 and F-171 are independent roots. Encryption reading starts first because
it removes the current hard-open failure and establishes the parameter parsing,
key derivation, and authenticated package boundary that F-170 must reuse.
Signature verification can proceed independently against the package
relationship graph. F-X039, F-X041, and F-X042 are independent rendering
roots. F-170 follows F-169 so files written here use the same validated
agile-encryption model that the reader accepts. F-X040 follows F-X039 because
restartable pagination needs shared page ownership and the checked reusable
engine boundary. F-X043 through F-X047 close the remaining useful behavior in
PRs 40 and 41 without importing their unchecked engine access, hash-authority,
or unbounded cache designs. They run serially because they share the reusable
layout engine. M17 continues in S53 with signature creation, tagged PDF,
PDF/A, and redaction.

## Definition of done for this sprint

- A password-protected document produced by Word opens with the correct
  password and fails cleanly with the wrong password.
- Wrong-password, malformed-parameter, authentication, and decryption failures
  leave the caller and package state unchanged.
- A document encrypted here decrypts here, and parameters fixed by the
  specification match the reviewed Word reference bytes.
- A validly signed document verifies every declared part, while a modified
  document fails with the changed or uncovered part named.
- Signature verification refuses missing, duplicate, external, or partially
  covered package relationships rather than reporting a weaker success.
- Shared font and page payloads avoid deep copies, and an editor can transfer
  reusable normal-layout work without serving a stale context.
- A warm edit rebuilds only its bounded pagination region and equals a fresh
  layout in pages, fonts, diagnostics, provenance, numbering, notes, fields,
  and outlines.
- Text at spaces, hyphens, ligatures, and other Unicode break opportunities
  emits every source scalar and shaped glyph exactly once in `PageFrame` and
  both built-in backends.
- Authored and reopened default, first, even, inherited, and multi-section
  headers and footers appear on the correct pages in deterministic PDF output.
- Encryption and verification preserve unrelated parts, content types,
  relationships, and unmodelled XML through their declared round trips.
- New cryptographic dependencies and feature edges satisfy the repository
  supply-chain, default-off, WASM, packaging, and licence gates.
- Caller-font editor layouts fall back only to bundled fonts, reuse work through
  checked exact-context transfer, and never observe the system-font snapshot.
- Editor-scale paragraph lookup, header and footer reuse, and substituted-page
  reuse remain exact, transactional, and inside declared retained-memory bounds.
- Empty Word paragraphs expose a zero-width attributed caret segment across
  body, table, header, footer, footnote, and endnote stories without visible
  rendering drift.
- The full workspace gate passes with all 49 deterministic hashes unchanged
  unless an approved design declares a reviewed delta.
