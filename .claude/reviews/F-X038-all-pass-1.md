# F-X038, all aspects, pass 1

**Reviewed**: uncommitted working diff, 6 files and 1,057 changed lines, with 983 insertions and 74 deletions
**Verdict**: 11 defects, 1 smell, 0 nitpicks

## Defects

### D1, a warm result retains fonts that a cold result no longer contains
`crates/rdocx-layout/src/engine.rs:713`

The reusable manager returns every face it has loaded over its lifetime. When
an edit removes the only paragraph using one family, or changes a run or style
from one family to another, `load_additional_fonts` takes its unchanged-set
early return and leaves the old `fonts` entries and ids in place. The warm
result therefore contains unused old fonts and can assign a different id to a
new face than an independent cold engine. This violates the gate's required
cold-versus-warm equality for fonts and glyph runs. The current warm test only
changes text while keeping the same family, so it cannot expose the mismatch.

### D2, a failed document layout can publish successful prefix paragraphs
`crates/rdocx-layout/src/engine.rs:803`

Each cache-safe paragraph is inserted immediately, before later body content,
headers, notes, pagination, and field substitution have succeeded. A document
with one successful safe paragraph followed by a failing paragraph or header
returns an error with the prefix entry still resident. The plan and named test
require failed layouts to publish no entries. Cache additions need transaction
scope for the complete layout, or they need to be rolled back on every later
error.

### D3, public caller-font layouts still initialize and consult system fonts
`crates/rdocx/src/document.rs:3263`

`layout_with_fonts_and_options` appends the caller bytes and then calls the
ordinary `layout_document_with_provenance` path. That path constructs
`Engine::new`, which consumes the process system-font snapshot. The isolated
`FontManager::new_with_fonts` constructor is used only by tests. A missing
glyph or family in the caller set can therefore resolve from system fonts, and
the first caller-font layout can initialize system discovery. This contradicts
the approved caller-font isolation contract and is not covered by the unit
test that constructs `FontManager::new_with_fonts` directly.

### D4, tracked normal layouts discard the reusable engine on every call
`crates/rdocx/src/document.rs:686`

Only accepted-view layout reaches `cached_layout` and its retained engine.
Every tracked-view normal layout calls the one-shot facade and loses paragraph,
resolution, and shaping reuse. The completed result may remain uncached, but
the approved private normal-font engine and revision-view cache identity are
supposed to support warm relayout without replacing the accepted `Arc`.
Interactive tracked revision rendering still repeats most work after every
edit.

### D5, persistent coverage state has no bound and appends duplicate entries
`crates/oxml-layout/src/font.rs:440`

The now long-lived `FontManager` retains `coverage_fallbacks` and
`coverage_misses` across edits. A partial fallback can append the same font
index again on every new missing-text query, and each newly unsupported scalar
is retained forever. The new limit applies only to the request-key map. It does
not bound these coverage collections, and a manager can also push more than
256 distinct resolved faces after the request-key map fills. This violates the
required bounded-memory behavior for a long-running editor.

### D6, the paragraph byte ceiling omits the cached reflow payload
`crates/rdocx-layout/src/engine.rs:897`

The stored block is cloned before the caller drops `reflow`, so even documents
without wrapping drawings retain a second set of inline text, glyph, and
advance buffers in every cache entry. `paragraph_cache_entry_bytes` counts only
the line buffers and never counts `block.reflow`. It also approximates the key
through debug text rather than its owned allocations. The recorded 16 MiB
total can therefore be far below the memory retained by the cache, so the byte
ceiling is not enforced.

### D7, AlternateContent drawings are incorrectly classified as safe
`crates/rdocx-layout/src/engine.rs:847`

The predicate examines only `run.content` and accepts a run whose ordinary
content is text even when `run.alt_drawings` contains a layout-only drawing.
The engine explicitly traverses those alternate drawings for wrapping and
anchored content. Reusing such a block can skip traversal-dependent work,
including numbering inside shape text, and directly violates the requirement
that drawings bypass paragraph reuse.

### D8, the context invalidation regression does not exercise invalidation
`crates/rdocx-layout/src/engine.rs:3318`

`shared_layout_context_changes_cannot_serve_stale_blocks` only constructs two
context values and asserts that they compare unequal. It never primes an
engine, changes an input, requests a second layout, or compares the result with
a cold layout. Removing the cache-clear branch from `layout_inner` would leave
this named regression green, so it does not prove that styles, numbering,
theme, relationships, images, or embedded fonts cannot serve stale blocks.

### D9, the diagnostics and failed-publication regression tests neither case
`crates/rdocx-layout/src/engine.rs:3425`

The cold and warm input emits no diagnostics, so comparing two empty vectors
would pass if diagnostic capture and replay were removed. The failure case has
no fonts and fails on its first text paragraph, before any prefix entry can be
published. It therefore cannot catch D2. The gate needs an actual ordered
diagnostic replay and a failure after at least one cache-safe paragraph has
successfully laid out.

### D10, the TTC sharing test never opens a TTC or selects two face indices
`crates/oxml-layout/src/font.rs:940`

The test writes one ordinary TTF and calls the path-only helper twice with the
same argument. The second value is merely named as another collection index.
No `fontdb` face, collection, or distinct face index is involved, so the test
does not prove that multiple TTC faces share one buffer or that another file
cannot alias through the production face-loading path.

### D11, the cache-boundary tests do not reach any byte ceiling
`crates/oxml-layout/src/font.rs:1018`

The file-cache test inserts four-byte values and copies an eviction loop into
the test instead of driving production insertion. The shaping test reaches its
entry ceiling with short strings before 16 MiB, and the paragraph test does the
same with short paragraphs. These tests remain green if the production byte
eviction is removed or if byte accounting undercounts retained payloads. The
boundary gate requires both entry and byte ceilings.

## Smells

### S1, every relayout deep-clones the complete shared context twice
`crates/rdocx-layout/src/engine.rs:516`

`Document::build_layout_input` already owns fresh copies of package-backed
fonts, images, charts, headers, footers, notes, styles, and numbering. The
engine then deep-clones all of them into `ParagraphCacheContext` before doing
an equality comparison. Large media and embedded-font documents retain a
second full snapshot in the persistent engine and pay another full byte copy
on every edit. That works against the performance objective and places memory
outside the advertised paragraph-cache ceiling.

## Nitpicks

None.

## Not found

No OOXML parser, serializer, namespace, schema-order, or raw-preservation
changes are present. No new trait, generic, feature flag, crate, module, source
file, or dependency was introduced. The exact shaping key covers font id,
owned text, and floating-point size bits, and additional-font changes clear
that memo. The direct source-node rebinding covers cached line items, tab
leaders, and retained reflow items. The inspected `expect` and constant panic
sites are guarded by immediately established invariants. The compile-time
`Document: Send + Sync` assertion covers the threading marker contract.
