# F-148, Comment API

**Status**: completed
**Sprint**: S46
**Size**: M
**Depends on**: F-147

## Problem

The public facade exports paragraphs, runs, and tables but no comment handle or
stable document range value (`crates/rdocx/src/lib.rs:24`). `Document` also has
no comments-extended state beside its other relationship-resolved parts
(`crates/rdocx/src/document.rs:41`). After F-147 makes comments reachable at
the OOXML layer, native callers still cannot add a ranged comment, reply to it,
mark a thread resolved, or remove it consistently from all related parts.

## Spec reference

- `docs/hld/14-development-backlog.md`, "F-148, Comment API".
- `docs/hld/03-architecture.md`, "Facade conventions".
- `docs/hld/04-opc-and-packaging.md`, "Relationship types", "Part naming",
  and "Package integrity".
- `docs/hld/10-bindings-spec.md`, "Native Word facade stability".

## Approach

Add additive native facade values for a document run position, an inclusive
start and exclusive end run range, comment metadata, and read-only comment
views. `Document::comments()` returns comments in part order. Mutations live on
`Document` so each operation can validate coordinates, allocate collision-free
ids and paragraph ids, update anchors and parts atomically, and invalidate the
layout cache once.

The proposed surface is:

```rust
pub struct RunPosition {
    pub body_index: usize,
    pub run_index: usize,
}

pub struct RunRange {
    pub start: RunPosition,
    pub end: RunPosition,
}

impl Document {
    pub fn comments(&self) -> Vec<CommentRef<'_>>;
    pub fn add_comment(
        &mut self,
        range: RunRange,
        author: &str,
        initials: Option<&str>,
        text: &str,
    ) -> Result<i32>;
    pub fn reply_to(&mut self, parent_id: i32, author: &str, text: &str) -> Result<i32>;
    pub fn resolve_comment(&mut self, id: i32, resolved: bool) -> Result<bool>;
    pub fn remove_comment(&mut self, id: i32) -> Result<bool>;
}
```

Replies and resolved state use a typed `commentsExtended` model keyed by the
comment paragraph id and parent paragraph id. Saving creates or updates the
comments and comments-extended relationships, content types, and required
namespace declarations together. Removal deletes the selected thread entry,
its extension entry, and its three body anchors. Removing the final comment
removes the now-unused parts, relationships, and overrides only when they are
owned by this typed model.

Proposed new source files:

```text
crates/rdocx-oxml/src/comments_extended.rs
crates/rdocx/src/comments.rs
```

No new trait, generic parameter, crate, dependency, or feature flag is
introduced.

## Rejected alternatives

- Accept borrowed `RunRef` values. They do not carry document coordinates and
  Rust borrowing prevents retaining two facade borrows while mutating the
  document.
- Accept four loose indices. A named range makes the half-open contract visible
  and can be shared by bookmarks without another coordinate convention.
- Encode replies only inside comments text. Word reads parent linkage and
  resolved state from comments-extended metadata.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `run_ranges_reject_reverse_missing_and_nonparagraph_positions` | Invalid coordinates return an error without changing body XML or package state |
| regression | `a_ranged_comment_reply_and_resolution_keep_one_intact_thread` | The comment, reply, anchors, paragraph ids, parent linkage, and resolved state reload as one thread |
| regression | `removing_a_comment_removes_only_its_anchors_and_thread_metadata` | Removal leaves adjacent runs, unrelated comments, and unmodelled extension bytes intact |
| integration | `comment_parts_relationships_and_content_types_are_word_compatible` | The generated package has reachable comments and comments-extended parts with the required content types and namespaces |

The **test gate**, from the backlog, is regression. A comment added over a
range, replied to, and resolved opens in Word with the thread intact.

Tests join the existing integration and regression binaries. The final Word
open is SHA-bound human-action evidence if no scriptable Word installation is
available.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/10-bindings-spec.md`

Record facade ownership, the comments-extended relationship graph, atomic
thread mutation, the half-open range contract, and the additive native API.

## Risk routing

- Any parser or serialiser. Read HLD 04 and HLD 06. Add prefix, schema-order,
  paragraph-id, relationship-target, round-trip, and byte-preservation checks.
- Public API of a published crate. Read HLD 10 and the structural rules. The
  comment and range APIs are additive and story-required. Run affected package
  dry-runs and archive size assertions.
- A new module or file. The focused extension and facade modules need explicit
  approval before implementation.
- An external oracle comparison. Follow differential-testing guidance. Record
  the exact Microsoft Word version and build, bind the generated document to
  its SHA-256, and record that Word opens the thread without repair and shows
  its reply and resolved state.

## Hash harness

Expected unchanged across all 49 entries. Existing generated samples do not
author comments.

## Implementation checklist

- [x] Add the stable run-position and half-open run-range values.
- [x] Add read, add, reply, resolve, and remove facade operations.
- [x] Parse and write comments-extended paragraph linkage and resolved state.
- [x] Make multi-part comment mutations atomic and collision-safe.
- [x] Add invalid-range, intact-thread, removal, and package graph tests.
- [x] Produce SHA-bound Word acceptance evidence or classify it human-action.
- [x] Update exactly HLD 03, HLD 04, and HLD 10 at completion.

## Open questions

None. The shared half-open `RunRange` contract and both focused source modules
are approved.
