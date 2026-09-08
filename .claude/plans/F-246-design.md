# F-246, Corpus style authoring

**Status**: completed
**Sprint**: S71
**Size**: L
**Depends on**: F-243, F-245

## Problem

The facade can enumerate styles and append a `StyleBuilder`, but it cannot
update or remove a style and performs no graph validation before mutation
(`crates/rdocx/src/document.rs:4082`). `CT_Style` already retains based-on and
next links plus conditional table regions, but linked styles and several typed
style flags remain in preserved extras (`crates/rdocx-oxml/src/styles.rs:46`).

The style capability remains partial (`docs/hld/02-scope-and-non-goals.md:218`).
Source-built documents need paragraph, character, and table defaults,
inheritance, links, next styles, and conditional regions to publish as one
validated graph.

## Spec reference

- `docs/hld/02-scope-and-non-goals.md`, capability row `DOCX-011`.
- `docs/hld/03-architecture.md`, typed projection and transaction conventions.
- `docs/hld/04-opc-and-packaging.md`, style-part preservation and schema order.
- `docs/hld/08-rendering-spec.md`, "The renderer's input".
- `docs/hld/10-bindings-spec.md`, "Native Word facade stability".
- `docs/hld/12-testing-strategy.md`, private authoring and Word fidelity gates.
- `docs/hld/13-risks-and-open-questions.md`, R4 and R13.
- `docs/hld/14-development-backlog.md`, "F-246, Corpus style authoring".

## Approach

Extend the existing `StyleBuilder` and `Style` facade with linked style,
priority, visibility, locking, automatic redefinition, UI, and conditional
table-region properties already represented by the style schema. Add
fallible `Document::add_style`, `set_style`, `remove_style`,
`set_default_style`, and `validate_style_graph` operations returning `Result`.

Build a complete candidate `CT_Styles`, validate unique ids, one default per
style type, reference existence, reference type compatibility, based-on cycle
freedom, linked-style reciprocity, and next-style legality, then publish it and
invalidate layout once. Reject removal while an owned style or document
paragraph still refers to the target.

Extend style resolution through authored defaults, theme fonts, based-on
chains, links, next styles, and conditional table regions. Keep the mutation
and resolver changes in the existing style, document, style-resolver, and table
files.

## Rejected alternatives

- Mutate one style before validating the graph. A failing cross-reference
  would expose half an invariant.
- Add a second style model. The existing CT and facade types already own this
  state.
- Silently detach references during removal. Callers need a deterministic
  validation error.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| differential | `source_built_style_graph_matches_pinned_word_effective_formatting` | Paragraph, character, and table styles resolve to the same effective formatting and deterministic render as the sanitized oracle. |
| regression | `invalid_style_graph_never_publishes_a_partial_mutation` | Missing targets, wrong link types, duplicate defaults, and cycles fail with original state intact. |
| round-trip | `authored_style_graph_survives_save_and_reopen` | Defaults, based-on, link, next, flags, and conditional regions return through the public facade. |
| regression | `style_removal_rejects_live_references_and_preserves_unknown_xml` | Live references block removal and unrelated producer extensions remain byte-identical. |

The **test gate is differential**. The source-built style graph resolves to the
same effective formatting and visible output as the sanitized Word oracle.

## HLD impact

- `docs/hld/02-scope-and-non-goals.md`
- `docs/hld/03-architecture.md`
- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/08-rendering-spec.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/13-risks-and-open-questions.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- **Layout, pagination, line breaking, and text shaping**. Run deterministic
  font layout and compare cold source-built output to the pinned render oracle.
- **Any parser or serializer**. Preserve unknown style children byte for byte,
  accept aliases, and emit fixed-prefix schema order.
- **Public API of a published crate**. State additive pre-1.0 impact and run the
  verified workspace package dry-run with archive-size assertions.
- **External oracle comparison**. Pin the structural and render oracle versions
  and compare normalized effective formatting plus a stated raster threshold.

## Hash harness

Expected unchanged for existing documents. New style graph operations are
opt-in and existing resolution must remain byte and render stable.

## Implementation checklist

- [x] Extend existing style CT and facade types with story-owned properties.
- [x] Add create, update, default, remove, and validation operations.
- [x] Validate the complete graph before publishing any mutation.
- [x] Extend effective paragraph, run, and table resolution.
- [x] Add atomic-failure, round-trip, preservation, and differential tests.
- [x] Run the full gate and all routed checks.
- [x] Update exactly the listed HLD files.

## Open questions

None. The user approved the pre-1.0 change that makes `add_style` fallible and
updates workspace callers so every public graph mutation validates atomically.
