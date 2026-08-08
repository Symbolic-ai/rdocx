# F-108, validate()

**Status**: approved
**Sprint**: S26
**Size**: M
**Depends on**: F-107

## Problem

The facade serializes its owned package at `crates/rpptx/src/lib.rs:140`
without checking the cross-part invariants that PowerPoint reports only as a
repair prompt. Neither callers nor debug builds can currently identify invalid
slide ids, duplicate shape ids, missing layout or theme relationships,
relationship graph defects, invalid text bodies, placeholder collisions,
custom-show references, or orphaned media before bytes are written.

## Spec reference

- `docs/hld/04-opc-and-packaging.md`, "Package integrity".
- `docs/hld/06-presentationml-model.md`, "Presentation.xml", "The shape
  tree", "Placeholders", and "Validation".
- `docs/hld/12-testing-strategy.md`, "The deck corpus".
- `docs/hld/14-development-backlog.md`, "F-108, validate()".

## Approach

Add the exact issue model from the HLD and a total, non-panicking facade check:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ValidationIssue {
    DuplicateShapeId { slide: usize, id: u32 },
    SlideIdOutOfRange { slide: usize, id: u32 },
    DuplicateSlideId { id: u32 },
    MissingContentTypeOverride { part: String },
    DanglingRelationship { part: String, r_id: String, target: String },
    UnreachableRelationshipTarget { part: String, target: String },
    EmptyTextBody { slide: usize, shape: usize },
    DuplicatePlaceholderIdx { slide: usize, idx: u32 },
    OrphanMedia { part: String },
    CustomShowReference { slide_id: u32 },
    MissingLayoutRel { slide: usize },
    MissingThemeRel { master: usize },
}

impl Presentation {
    pub fn validate(&self) -> Vec<ValidationIssue>;
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()>;
}
```

Walk each slide shape tree recursively, including selected fallbacks, to check
shape ids, nonempty text bodies, and placeholder indices. Check slide ids and
the presentation's preserved custom-show references. Check every part's
content type and every relationship scope. A relationship that is not
referenced by its source XML is dangling, an internal relationship whose
resolved target is absent is unreachable, and a `/ppt/media/` part with no
incoming internal relationship is orphaned. Check each slide for exactly one
layout relationship and each reachable master for a theme relationship.

Keep `validate()` observational. It returns all issues in deterministic order
and does not repair, mutate, panic, or perform I/O. `save()` writes the result
of `to_bytes()`. Under `debug_assertions`, both byte and path save boundaries
assert that `validate()` is empty before serializing. Release builds still
allow callers to inspect issues without forcing policy on intentionally broken
input.

## Rejected alternatives

- Return only the first issue. It makes iterative package repair slow and
  makes corpus evidence dependent on traversal accidents.
- Reject invalid input while opening. Validation is a separate cheap
  diagnostic surface and the facade must be able to inspect corrupted decks.
- Repair relationships or ids automatically. That can change preserved XML
  and hide the producer defect.
- Validate only typed fields. Relationship references and custom shows can
  live in preserved XML, so package XML must be inspected without replacing
  it.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| negative, gate | `every_validation_issue_variant_detects_its_corrupted_deck` | One deliberately corrupted package produces each of the twelve exact variants |
| regression | `validate_collects_all_issues_in_deterministic_order` | Multiple defects are returned together in stable order without panic or mutation |
| integration | `all_pinned_corpus_decks_validate_cleanly` | Every one of the 50 verified corpus decks returns no issue |
| regression | `debug_save_boundaries_assert_on_invalid_presentations` | Debug `to_bytes()` and `save()` reject an invalid owned deck before OPC writing |
| integration | `save_writes_the_same_bytes_as_to_bytes` | The path API uses the deterministic byte serialization and the saved deck reopens |
| round-trip | `validation_xml_scan_is_prefix_tolerant_and_non_mutating` | Alternate relationship prefixes and preserved custom-show XML are inspected without changing bytes |

The backlog test gate is named explicitly: one deliberately corrupted deck per
variant is detected, and the whole corpus validates clean.

## HLD impact

None. The existing HLD already defines all issue variants, validation
semantics, debug-save behavior, and package checks.

## Risk routing

- Parser or serialiser: read `docs/hld/04-opc-and-packaging.md` and
  `docs/hld/06-presentationml-model.md`. Add prefix-tolerant inspection,
  deterministic issue ordering, fixed-prefix reparse, and byte-preservation
  checks.
- Public API of an unpublished crate: read `docs/hld/10-bindings-spec.md` and
  the structural rules in `CLAUDE.md`. State that there is no released semver
  impact. Add the exact HLD enum and concrete methods without a trait or new
  module.

## Hash harness

Expected to be unchanged. Validation reads unpublished PresentationML package
state and does not alter Word rendering output.

## Implementation checklist

- [ ] Add the exact public `ValidationIssue` enum and deterministic collector.
- [ ] Validate slide ids, recursive shapes, text bodies, and placeholders.
- [ ] Validate content types, relationship references and targets, and media
  reachability.
- [ ] Validate custom-show ids plus slide-layout and master-theme links.
- [ ] Add `save()` and debug assertions at both serialization boundaries.
- [ ] Add one corrupted-deck gate per variant and require the full corpus.

## Open questions

None. Validation reports every defect without mutation, and debug assertions
guard the existing byte save plus the new path save.
