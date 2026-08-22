# Current Sprint, S52

**Milestone**: M17 Security and compliance.

**Goal**: open the files that currently cannot be opened at all. Add
standards-compatible read and write support for OOXML agile encryption, then
verify digital signatures over their complete declared part sets so callers
can distinguish trusted packages from altered or partially covered ones.

## Spec references

- `docs/hld/03-architecture.md`, for keeping package security ownership below
  the native Word facade and preserving the existing document model boundary.
- `docs/hld/04-opc-and-packaging.md`, for package relationships, part naming,
  integrity validation, and staged failure-safe package replacement.
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
| F-169 | Agile encryption, read | L | pending | - |
| F-171 | Digital signature verification | L | pending | - |
| F-170 | Agile encryption, write | M | pending | - |

## Sequencing note

Rows are listed in dependency order, not F-ID order.

F-169 and F-171 are independent roots. Encryption reading starts first because
it removes the current hard-open failure and establishes the parameter parsing,
key derivation, and authenticated package boundary that F-170 must reuse.
Signature verification can proceed independently against the package
relationship graph. F-170 follows F-169 so files written here use the same
validated agile-encryption model that the reader accepts. M17 continues in S53
with signature creation, tagged PDF, PDF/A, and redaction.

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
- Encryption and verification preserve unrelated parts, content types,
  relationships, and unmodelled XML through their declared round trips.
- New cryptographic dependencies and feature edges satisfy the repository
  supply-chain, default-off, WASM, packaging, and licence gates.
- The full workspace gate passes with all 49 deterministic hashes unchanged
  unless an approved design declares a reviewed delta.
