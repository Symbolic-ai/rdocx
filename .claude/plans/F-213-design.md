# F-213, Animation and transition timing model

**Status**: approved
**Sprint**: S60
**Size**: L
**Depends on**: none

## Problem

`CT_Slide`, `CT_SlideLayout`, and `CT_SlideMaster` currently retain
`p:timing` and `p:transition` only as ordered raw children in
`crates/rpptx-oxml/src/slide_parts.rs`. Callers cannot inspect or mutate timing
trees, supported effects, motion paths, slide transitions, or morph metadata.

The M21 contract supersedes opaque-only preservation, but it does not permit
data loss. Every unsupported timing sibling, attribute, extension, namespace
binding, and relationship-bearing subtree must remain in its original schema
slot while the supported projection becomes editable.

## Spec reference

- ECMA-376 Part 1, PresentationML timing, animation, target, condition, and
  slide-transition complex types.
- Microsoft Office PresentationML extensions for morph transition metadata.
- `docs/hld/02-scope-and-non-goals.md`, "Beyond v1" and "Superseded".
- `docs/hld/04-opc-and-packaging.md`, "Relationship types", "Package
  integrity", and canonical serialization policy.
- `docs/hld/06-presentationml-model.md`, "Preservation strategy" and
  "Validation".
- `docs/hld/10-bindings-spec.md`, published native Rust API policy.
- `docs/hld/12-testing-strategy.md`, "Test taxonomy" and "The deck corpus".
- `docs/hld/14-development-backlog.md`, "F-213, Animation and transition
  timing model".

## Approach

Add one focused `rpptx-oxml` timing module. It will model the supported
PresentationML tree with concrete values:

```rust
pub struct CT_Timing {
    pub nodes: Vec<TimingNode>,
    pub builds: Vec<TimingBuild>,
}

pub enum TimingNode {
    Parallel(TimingContainer),
    Sequence(TimingSequence),
    Set(TimingSet),
    Animate(TimingAnimate),
    Effect(TimingEffect),
    Motion(TimingMotionPath),
    Unsupported(TimingUnsupported),
}

pub struct CommonTimeNode {
    pub id: u32,
    pub duration: TimingDuration,
    pub fill: Option<TimingFill>,
    pub restart: Option<TimingRestart>,
    pub node_type: Option<TimingNodeType>,
    pub start_conditions: Vec<TimingCondition>,
    pub end_conditions: Vec<TimingCondition>,
    pub children: Vec<TimingNode>,
}

pub enum TimingTarget {
    Shape(u32),
    Slide,
    Unsupported,
}

pub struct CT_SlideTransition {
    pub speed: Option<TransitionSpeed>,
    pub duration_ms: Option<u64>,
    pub advance_after_ms: Option<u64>,
    pub effect: TransitionEffect,
    pub morph: Option<MorphMetadata>,
}
```

`TimingDuration` represents finite integer milliseconds and `indefinite`
without floating point. Conditions type the bounded trigger vocabulary and
retain unsupported events. Effects and behaviors type entrance, exit,
emphasis, set, animate, and motion metadata without evaluating it. Build
entries retain target and paragraph ordering for the main timing tree.
Transition effect names and parameters remain typed enough for F-214 to select
its supported subset, while unknown choices remain explicit and lossless.

Every model keeps private raw attributes and ordered raw children. Readers
resolve expanded names, accept namespace aliases, and use
`capture_element`. Writers use fixed model prefixes, enforce
`xsd:sequence`, and place unsupported content back into its recorded slot.
Untouched producer-shaped subtrees serialize byte-identically. A supported
mutation rewrites only the owning modelled element and keeps its unsupported
siblings and relationship attributes intact.

Replace the raw timing and transition slots in `CT_Slide`, `CT_SlideLayout`,
and `CT_SlideMaster` with optional typed fields. Add constructors and mutation
accessors on the low-level values only. F-213 adds no facade execution API,
renderer behavior, trait, generic, feature, crate, or dependency.

The public change is additive for the pre-1.0 published `rpptx-oxml` crate.

## Rejected alternatives

- Parsing only effect names would leave container order, triggers, targets,
  and duration semantics inaccessible to F-214.
- Canonicalizing the whole timing subtree after one supported mutation would
  destroy unsupported producer XML and relationship-bearing extensions.
- Evaluating timing in `rpptx-oxml` would mix format modelling with layout
  policy and reverse the existing layer boundary.
- Adding timing fields directly to the already large slide-parts module would
  make unrelated root parsing harder to reason about.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| round-trip | `timing_sequences_parallel_groups_triggers_and_effects_round_trip_in_schema_order` | Prefix-tolerant parsing, typed supported nodes, fixed-prefix writing, schema order, and structural reparse equality. |
| round-trip | `timing_mutation_preserves_every_unsupported_sibling_and_relationship_attribute` | Unsupported nodes, extensions, attributes, namespace bindings, and relationship references remain byte-identical around one supported mutation. |
| round-trip | `slide_transitions_and_morph_metadata_survive_mutation_save_and_reopen` | Transition speed, duration, advance, effect parameters, morph metadata, and raw siblings survive package staging and reopen. |
| regression | `malformed_timing_does_not_partially_mutate_the_slide` | Duplicate required children, bad finite durations, invalid ids, and out-of-order modelled children fail before live state changes. |
| round-trip | `the_corpus_timeline_preserves_every_unsupported_sibling` | Nonzero corpus timing coverage parses, serializes, reparses, and keeps every unsupported subtree in place. |

The exact backlog **test gate is round-trip**: "The corpus timeline parses into
the declared model, serializes in schema order, and preserves every unsupported
sibling."

Add tests to the existing `crates/rpptx-oxml/tests/integration.rs` and
`crates/rpptx/tests/integration.rs` binaries. Do not add a new integration
binary or binary fixture.

## HLD impact

- `docs/hld/06-presentationml-model.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`

## Risk routing

- Any parser or serialiser: re-read `docs/hld/04-opc-and-packaging.md` and
  `docs/hld/06-presentationml-model.md`. Add namespace-alias, fixed-prefix,
  schema-order, structural-reparse, and byte-exact unmodelled-subtree checks.
- Public API of a published crate: state the additive pre-1.0 semver impact.
  Run the `rpptx-oxml` publish dry run and assert its archive remains below
  10 MiB.
- New module or file: obtain explicit approval for
  `crates/rpptx-oxml/src/timing.rs`. No new trait, generic, crate, feature, or
  dependency is introduced.

## Hash harness

Expected unchanged. This story changes PresentationML modelling and package
round-trip behavior only. All 49 ordinary deterministic rendering entries must
remain byte-identical.

## Implementation checklist

- [ ] Add the approved timing module and concrete duration, condition, target,
  container, build, set, animate, effect, motion, transition, and morph values.
- [ ] Parse supported timing elements by expanded name and retain unsupported
  XML in ordered schema slots.
- [ ] Serialize typed timing and transition values with fixed prefixes and
  schema-valid child order.
- [ ] Expose optional typed timing and transition fields on slide, layout, and
  master roots.
- [ ] Add safe mutation paths that retain unsupported siblings and
  relationship attributes.
- [ ] Add source-built round-trip, mutation, malformed-input, and package tests
  to the existing integration binaries.
- [ ] Run focused `rpptx-oxml` and `rpptx` checks plus every routed rider.

## Open questions

None. The new timing module and the declared model boundary are approved.
Media commands and all other unsupported behaviors remain explicit raw nodes.
