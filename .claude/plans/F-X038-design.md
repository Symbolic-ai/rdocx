# F-X038, Cache relayout work across document edits

**Status**: draft
**Sprint**: S51
**Size**: L
**Depends on**: F-X037, F-X032

## Problem

An interactive consumer lays out after every edit. The current normal-font path
creates a new `rdocx-layout::Engine`, creates a new `FontManager`, scans system
fonts, copies file-backed face bytes, reshapes every run, and rebuilds every
paragraph after each result-cache invalidation. Issue 39 reports 1,144 ms for a
63-page mixed Latin and Korean document, with about 750 ms in system font
discovery and 230 ms in repeated face-byte access. Its prototype reduces the
same relayout to 101 ms.

The measurements justify the work, but the prototype cache keys are not safe
enough to merge unchanged. A 64-bit text hash can alias shaping results. Its
font fingerprint omits font bytes. Its paragraph key omits shared layout
context and diagnostics. Reusing a cached block would also retain F-X037
source ids that are local to an older layout result. Cache correctness and
invalidation are therefore part of this story, not deferred cleanup.

## Spec reference

- `docs/hld/03-architecture.md`, "What stays put" and native document cache
  ownership.
- `docs/hld/08-rendering-spec.md`, "Performance", deterministic rendering, and
  Word revision views.
- `docs/hld/10-bindings-spec.md`, native Word facade stability and the planned
  0.8 Rust boundary.
- `docs/hld/12-testing-strategy.md`, "Test taxonomy" and "The hash harness".
- `docs/hld/14-development-backlog.md`, "F-X038, Cache relayout work across
  document edits".
- `docs/hld/15-build-and-toolchain.md`, feature boundaries, WASM, and the two
  release families.

## Approach

In the existing `oxml-layout/src/font.rs`, cache bundled plus system
`fontdb::Database` discovery once per process when `system-fonts` is enabled.
Each normal `FontManager` clones that discovered face table. Deterministic and
caller-font construction remains isolated from process system fonts.

Cache file-backed font bytes process-wide by canonical file identity, sharing
one `Arc<[u8]>` across all collection indices in the same TTC file. In-memory
bundled, embedded, and caller-provided faces remain manager-owned. A poisoned
cache lock recovers by rebuilding the requested entry rather than failing all
future layout.

Add a bounded shaping memo to `FontManager`. Its identity is the exact
`FontId`, owned source text, and `size_pt.to_bits()`. A hash table may index the
entry, but equality compares the complete key before reuse. Loading or
replacing additional fonts clears resolution, coverage, and shaping state.

Make the existing `rdocx-layout::Engine` reusable and give its normal-font
path a bounded paragraph-block cache. Retain one private normal-font engine in
`Document` across completed-result cache invalidation. Deterministic layouts
and caller-provided font layouts keep their existing isolated construction.
The private engine remains behind the document's synchronization boundary so
`Document` stays `Send + Sync`.

Cache only ordinary body paragraphs proven independent of traversal state.
Numbering, drawings, fields, hyperlinks, media, generated markers, and other
context-sensitive paragraphs bypass the block cache. A complete cache identity
includes the paragraph content, content width, revision view, and every shared
style, numbering, theme, embedded-font, media, and relationship input that can
change its layout. Public document mutations either supply the current
generation to that identity or clear affected entries.

Cache and replay paragraph diagnostics together with each block. Publish an
entry only after successful layout. Never retain F-X037 result-local
`SourceNodeId` values. Cached scalar ranges are rebound to the source node
allocated for the current `WordLayoutResult`, so inserting or moving a prior
paragraph cannot return stale provenance.

Use explicit entry and byte ceilings with simple private eviction in the
existing files. Add no dependency, trait, generic, feature flag, module, or
new source file.

## Rejected alternatives

- Cherry-pick issue 39 commit `cac4e2ec` unchanged. Its lossy cache identities
  can return content from a different text, font set, document context, or
  provenance result.
- Assert the reporter's 1,144 ms to 101 ms timing in CI. The fixture and system
  font installation are private and machine-dependent. CI asserts reuse and
  exact output instead.
- Cache every paragraph immediately. Numbering and other traversal-dependent
  content must continue to advance through the live engine.
- Store result-local source ids in paragraph blocks. F-X037 defines those ids
  as local to one output and they must be rebound on every layout.
- Introduce a session facade. `Document` is the existing owner used by the
  editor path, and no second session implementation exists today.
- Use an unbounded process cache. A long-running editor may encounter many
  fonts and documents, so bounds and recovery are required behavior.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `normal_font_discovery_initializes_once_per_process` | Two normal managers share one discovery while deterministic and caller-font managers do not consume it |
| unit | `file_backed_collection_faces_share_one_byte_buffer` | Repeated resolution and multiple TTC face indices read one file once without aliasing another file |
| unit | `shaping_memo_uses_complete_text_size_and_font_identity` | Exact repeats hit, while different text, size, face bytes, and embedded-font replacement never alias |
| regression, gate | `warm_relayout_matches_cold_and_rebuilds_only_changed_safe_paragraphs` | A mixed document has byte-equivalent cold and warm results and edits rebuild only the changed cache-safe paragraph |
| regression | `shared_layout_context_changes_cannot_serve_stale_blocks` | Styles, numbering, theme, hyperlinks, images, relationships, and embedded fonts invalidate or bypass reuse |
| regression | `warm_provenance_rebinds_to_current_word_source_nodes` | Inserted and reordered paragraphs resolve every F-X037 span to the current exact paragraph |
| regression | `cold_and_warm_diagnostics_are_identical` | Diagnostics and ordering match and failed layouts publish no entries |
| boundary | `relayout_caches_are_bounded_and_recover_from_poison` | Entry and byte ceilings evict safely and poisoned process locks recover |
| compatibility | `document_remains_send_and_sync` | Persistent engine ownership does not weaken the facade's threading contract |

The **test gate** is regression. A warm normal-font relayout must equal a cold
layout, including pages, fonts, diagnostics, revision projection, and resolved
F-X037 provenance. It rebuilds only the changed cache-safe paragraph. Every
context mutation either invalidates or bypasses stale work, cache bounds and
poison recovery are proved, both WASM targets compile, and all 49 output hashes
remain unchanged.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/08-rendering-spec.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- **Layout, pagination, line breaking, and text shaping**. Read HLD 08. Use
  deterministic cold-versus-warm equality for behavior and require the full
  hash harness to remain unchanged.
- **System-font feature boundary**. Read HLD 15. Run
  `cargo test -p oxml-layout --no-default-features` and both WASM target checks.
  Deterministic and caller-font paths must not observe the system snapshot.
- **Public behavior of published crates**. Read HLD 10 and the structural
  rules. The public API stays source-compatible. Run all package dry-runs and
  enforce the 10 MiB archive ceiling.
- **Threading and process-global state**. Prove `Document: Send + Sync`, bound
  memory, recover poisoned locks, and run repeated and concurrent focused
  cache tests.

No parser, serializer, external oracle, dependency, feature flag, new module,
or new file is introduced by implementation.

## Hash harness

Expected unchanged across all 49 entries. Caching may skip repeated work but
must not change layout, fonts, diagnostics, generated packages, or renders.

## Implementation checklist

- [ ] Share normal system-font discovery without changing deterministic fonts.
- [ ] Share file-backed bytes by file identity across TTC indices.
- [ ] Add a bounded exact-key shaping memo with complete font invalidation.
- [ ] Retain one synchronized normal-font engine per document.
- [ ] Cache only context-independent body paragraphs with complete invalidation.
- [ ] Replay diagnostics and publish entries only after successful layout.
- [ ] Rebind cached scalar spans to current result-local source ids.
- [ ] Prove cold and warm equality, bounds, poison recovery, and threading.
- [ ] Run no-default, WASM, package, archive-size, and full verification riders.
- [ ] Update exactly the HLD files listed above.
- [ ] Credit `@emptinessform` and document the process-lifetime system-font
  snapshot in the 0.4.0 and 0.8.0 release notes.

## Open questions

None. The user requested issue 39 in S51. The story adopts the measured cache
layers only with collision-free identity, complete invalidation, bounded
memory, exact diagnostics, and current-layout provenance.
