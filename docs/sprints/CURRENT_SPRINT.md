# Current Sprint, S59

**Milestone**: M21 Presentation depth.

**Goal**: deepen the Presentation package and collaboration surfaces before
adding dynamic rendering behavior. The sprint adds editable collaboration and
navigation metadata plus password and signature policy while preserving
unsupported executable payloads without running them.

## Spec references

- `docs/hld/03-architecture.md`, for shared package-security ownership and the
  dependency boundaries that keep cryptographic features out of ordinary
  graphs.
- `docs/hld/04-opc-and-packaging.md`, for transactional Agile encryption,
  digital-signature verification and creation, relationship ownership, and
  mutation invalidation rules.
- `docs/hld/06-presentationml-model.md`, for PresentationML relationship
  ownership, latent footer placeholders, ordered raw preservation, and the
  boundary between typed and opaque presentation content.
- `docs/hld/10-bindings-spec.md`, for facade-level password and signature APIs
  and the native-only feature boundary.
- `docs/hld/12-testing-strategy.md`, for pinned PowerPoint differentials,
  package round trips, trusted certificate fixtures, and mutation gates.
- `docs/hld/14-development-backlog.md`, for the F-217 and F-221 acceptance
  contracts, dependencies, and test gates.
- `docs/hld/15-build-and-toolchain.md`, for default-off encryption and
  signature features plus their dependency and portability policy.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-217 | Presentation collaboration and navigation model | L | done | - |
| F-221 | Presentation encryption and signatures | M | done | - |

## Sequencing note

Rows are listed in dependency order, not F-ID order.

F-217 is an independent PresentationML model and package root. F-221 depends
on the completed shared security stories F-169 through F-172 and exposes that
implementation through the Presentation facade. The two S59 stories have no
dependency on each other and may proceed in parallel isolated worktrees, with
their package and facade interaction reconciled by integrated verification.

## Definition of done for this sprint

- Comments, replies, sections, slide numbers, dates, footers, notes headers,
  and handout settings survive ordered mutation, save, and reopen with their
  relationships intact.
- Pinned PowerPoint opens password-protected output, signature verification
  matches the trusted certificate fixtures, and every relevant mutation
  invalidates rather than falsely preserving signature validity.
- Unsupported executable content remains preserved and inspectable but is
  never executed, and binary `.ppt` remains out of scope.
- Full verification passes with every deterministic hash explained and all
  package, portability, documentation, and supply-chain gates green.
