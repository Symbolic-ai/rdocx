# F-150, Accept and reject revisions

**Status**: completed
**Sprint**: S47
**Size**: L
**Depends on**: F-149

## Problem

F-149 makes tracked revisions inspectable but deliberately retains their raw
subtrees as the serialization source. The facade still has no operation that
chooses the current or recorded state of those revisions. Callers cannot
accept or reject every change, scope the operation by author, date range, or
revision id, or guarantee that a malformed selected change leaves the document
unchanged.

The transformation is placement-sensitive. Accepting an inserted content
wrapper keeps its children, while accepting a deletion removes them. Rejecting
does the reverse. Moves pair those same choices across `w:moveFrom` and
`w:moveTo`. Property changes choose between the current properties and the
recorded prior value. Contextual insertion and deletion markers on paragraph
marks or table rows cannot be treated as content wrappers.

## Spec reference

- `docs/hld/14-development-backlog.md`, "F-150, Accept and reject revisions".
- `docs/hld/03-architecture.md`, "What stays put" and "Facade conventions".
- `docs/hld/04-opc-and-packaging.md`, "Package integrity".
- `docs/hld/10-bindings-spec.md`, "Native Word facade stability".
- `docs/hld/12-testing-strategy.md`, "Test taxonomy".

## Approach

Extend the approved facade revision module with explicit operations whose
names mirror the backlog and avoid a new public filter abstraction:

```rust
impl Document {
    pub fn accept_all(&mut self) -> Result<usize>;
    pub fn reject_all(&mut self) -> Result<usize>;
    pub fn accept_revisions_by_author(&mut self, author: &str) -> Result<usize>;
    pub fn reject_revisions_by_author(&mut self, author: &str) -> Result<usize>;
    pub fn accept_revisions_in_date_range(
        &mut self,
        start: &str,
        end: &str,
    ) -> Result<usize>;
    pub fn reject_revisions_in_date_range(
        &mut self,
        start: &str,
        end: &str,
    ) -> Result<usize>;
    pub fn accept_revision_id(&mut self, id: i32) -> Result<usize>;
    pub fn reject_revision_id(&mut self, id: i32) -> Result<usize>;
}
```

Each method returns the number of matched revision elements. Author matching
is exact and case-sensitive. Id matching selects every modeled element with
that id so a producer may represent one logical move with several placed
elements. Date ranges are inclusive RFC 3339 instant ranges. Validate and
normalize both bounds before mutation without adding a date dependency.
Revisions with no timestamp do not match a date range. A malformed bound or a
start later than the end returns an error without mutation.

Transform a cloned `CT_Document`. For a selected content revision, unwrap or
remove the raw subtree according to its kind and action, then parse the staged
document again so retained children become ordinary typed content. Convert
`w:delText` to `w:t` when rejecting a deletion. For a selected property
revision, remove the change marker when accepting or replace current
properties with the projected prior value when rejecting. Contextual markers
use their owning property grammar to keep or remove the affected paragraph
mark or table row. Apply nested revisions from the inside out so one selected
wrapper cannot hide a separately selected descendant.

Serialize and reparse the complete staged document before assignment. Commit
the replacement and invalidate both layout caches only after every selected
revision resolves successfully. The package and live document therefore remain
unchanged on invalid metadata, malformed content, serialization failure, or
scope validation failure.

No source file beyond the two modules proposed by F-149 is needed. No new
crate, dependency, trait, generic parameter, feature flag, binding method, or
test binary is added.

## Rejected alternatives

- Expose one public `RevisionFilter` enum. Eight explicit methods match the
  requested surface without another type callers must construct.
- Mutate raw slots as revisions are discovered. A later failure could leave a
  partially accepted document.
- Rewrite the package part directly. That would create a second document
  representation and bypass the facade's typed ownership and layout caches.
- Compare date strings lexically. Equivalent RFC 3339 timestamps with different
  offsets would sort incorrectly.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `revision_scope_matches_author_id_and_inclusive_instants` | Exact author matching, shared ids, offset timestamps, inclusive endpoints, and missing dates behave as specified |
| unit | `invalid_date_ranges_leave_the_document_unchanged` | Malformed or reversed bounds fail before any staged edit becomes visible |
| regression | `rejecting_insertions_and_deletions_restores_the_recorded_content` | Insertions disappear, deletions and deleted text return, and move pairs choose the correct source or destination |
| regression | `rejecting_property_changes_restores_every_recorded_prior_value` | Run, paragraph, table, and section properties revert without duplicate change nodes |
| regression | `scoped_revision_actions_change_only_matching_revisions` | Author, date, and id operations leave every unselected raw subtree byte-identical |
| regression | `accepting_every_revision_matches_word_normalized_body_xml` | The accepted body tree equals the pinned Word oracle output for the same in-code input |
| integration | `a_failed_revision_action_is_atomic` | Document XML, package bytes, and layout cache behavior remain unchanged on a selected malformed revision |

The **test gate**, from the backlog, is regression. Accepting every revision
produces the normalized body XML that Word produces from the same input.

The oracle input and expected normalized body XML are text constants in the
existing regression entrypoint. The harness records the exact Microsoft Word
build that produced the expected tree. Comparison is structural after XML
normalization, not a byte comparison. No binary fixture or published build
dependency is added.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/10-bindings-spec.md`

Record document-owned revision resolution, placement-specific semantics,
atomic staging and validation, the scoped native methods, and unchanged
binding surfaces.

## Risk routing

- Any parser or serialiser. Read HLD 04 and HLD 06. Add schema-order,
  prefix-tolerant, nested-revision, selected-versus-unselected raw
  preservation, accepted-output reparse, and atomic failure checks.
- Public API of a published crate. Read HLD 10 and the structural rules. The
  eight native methods are additive and story-required. Run
  `cargo publish --workspace --dry-run` and assert every produced `.crate`
  remains below 10 MiB.
- External oracle comparison. Read the differential-testing skill. Pin the
  Microsoft Word build in test infrastructure and compare normalized XML trees
  instead of prefixes, whitespace, or attribute order.

## Hash harness

Expected unchanged across all entries. Existing generated samples contain no
tracked revisions. Any delta is unexplained and blocks the sprint.

## Implementation checklist

- [x] Add total selectors for all revisions, author, inclusive date range, and id.
- [x] Resolve inserted, deleted, moved, and deleted-text content by placement.
- [x] Accept or restore run, paragraph, table, and section property changes.
- [x] Handle contextual insertion and deletion markers in their owning grammar.
- [x] Stage, serialize, reparse, commit atomically, and invalidate layout once.
- [x] Preserve every unselected revision subtree byte-for-byte.
- [x] Add selector, transformation, prior-property, nesting, and atomicity tests.
- [x] Add the pinned Word normalized-body regression without a binary fixture.
- [x] Run focused checks plus the parser, packaging, and oracle riders.
- [x] Update exactly HLD 03, HLD 04, and HLD 10 at completion.

## Open questions

None. The focused OXML and facade revision modules are approved, and the story
covers every reachable main-document placement of the listed elements. Date
ranges use inclusive RFC 3339 instants, malformed bounds fail atomically,
missing revision dates do not match, and one id-scoped operation resolves every
modeled element carrying that id.
