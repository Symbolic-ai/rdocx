# Current Sprint, S70

**Milestone**: M23 From-scratch business documents.

**Goal**: establish the measured public-API completeness contract for modern
business documents, turn the five private client documents into a non-committed
conformance corpus, make the public capability surface visible, and close the
three independent incremental-layout gaps reported in Issue 69.

## Spec references

- `docs/hld/02-scope-and-non-goals.md`, for the M23 and M24 authoring boundary
  and the permanent execution non-goals.
- `docs/hld/03-architecture.md`, for facade ownership, dependency direction,
  and preservation of unmodeled XML.
- `docs/hld/04-opc-and-packaging.md`, for deterministic OPC ownership,
  relationships, content types, and loss-free retention.
- `docs/hld/08-rendering-spec.md`, for incremental layout, cache reuse,
  source provenance, and deterministic rendering.
- `docs/hld/10-bindings-spec.md`, for the public Rust API and binding parity
  that the completeness audit must classify.
- `docs/hld/12-testing-strategy.md`, for the private corpus, conformance,
  differential, regression, and deterministic-font gates.
- `docs/hld/13-risks-and-open-questions.md`, for corpus confidentiality,
  completeness scope, and facade-divergence controls.
- `docs/hld/14-development-backlog.md`, for the F-240 through F-242 and
  F-X083 through F-X086 acceptance contracts.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-240 | Modern DOCX completeness audit and private corpus matrix | L | done | - |
| F-241 | Public authoring conformance harness | L | in-progress | codex |
| F-242 | Root README product and capability overview | M | pending | - |
| F-X083 | Close confirmed Issue 67 and intake Issue 69 | S | done | - |
| F-X084 | Narrow note-part paragraph cache invalidation | M | in-progress | codex |
| F-X085 | Memoize restart body identities once per layout | M | pending | - |
| F-X086 | Provenance-safe restart after body-length changes | L | pending | - |

## Sequencing note

F-X083 lands first so Issue 67 is closed with the reporter's confirmation and
Issue 69 is recorded as three independently reviewable changes with contributor
credit. F-X084 through F-X086 may then proceed independently, but each remains
its own behavioural commit and hash-harness review. F-240 is the planning
authority for M23 and M24. It may revise provisional later stories and must
update the shared sprint and HLD records as an explicit audit output. F-241 and
F-242 begin after the audit fixes the capability vocabulary and corpus policy.

## Definition of done for this sprint

- The public `rdocx` surface is audited against the full authoring properties
  reachable in the modeled WordprocessingML layer, including numbering
  authoring, story scope, package support, layout, bindings, and determinism.
- Every capability is classified as complete, partial, preserve-only,
  unsupported, or a permanent non-goal, with a cited owning story for every
  incomplete in-scope row.
- The audit updates `BACKLOG.md`, `SPRINT_PLAN.md`, `CURRENT_SPRINT.md`, and the
  affected HLD files when evidence changes the provisional plan.
- The five client documents remain in an ignored private directory. Their
  fingerprints, required-part inventories, structural assertions, and visual
  expectations can drive local conformance without committing customer data.
- A public synthetic conformance suite proves that from-scratch fixtures use
  `Document::new()` and public `rdocx` APIs only. Raw XML injection and base
  templates fail this gate.
- The root README work has a stable capability vocabulary, comparison format,
  and roadmap source that can be kept current without duplicating the backlog.
- Issue 67 is closed only after recording the reporter's confirmation that
  F-X075 in v0.12.0 fixed it. Issue 69 retains the reporter's attribution and
  links each offered commit to its matching story.
- Note-part invalidation no longer discards reusable paragraph cache entries
  for paragraphs without note references.
- Restart body identities are computed at most once per block per layout.
- Safe prefix restart survives body-length changes while tail reuse remains
  gated by valid source provenance.
- Each Issue 69 change has focused correctness, cache-reuse, fresh-layout
  equality, performance, and hash-harness evidence before integration.
