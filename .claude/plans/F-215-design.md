# F-215, Audio and video package model

**Status**: completed
**Sprint**: S61
**Size**: L
**Depends on**: none

## Problem

`rpptx` currently treats slide media as image relationships only. The package
assembly in `crates/rpptx/src/lib.rs` admits `rel_types::IMAGE`, while
`CT_Picture` in `crates/rpptx-oxml/src/picture.rs` preserves the non-visual
audio and video payload only as raw XML. Callers cannot inspect, add, replace,
extract, or remove the embedded and linked media represented by the tracked
`EmbeddedAudio.pptx` and `EmbeddedVideo.pptx` corpus decks.

The producer XML binds one picture shape to a poster image, an audio or video
relationship, the Microsoft media relationship, and timing commands. Any
mutation must update those resources atomically, retain unsupported codec bytes
and metadata, and preserve relationship ids inside retained XML.

## Spec reference

- ECMA-376 Part 1, PresentationML audio, video, common media node, command,
  timing target, and non-visual application properties.
- Microsoft Office 2010 PresentationML `p14:media` extension.
- `docs/hld/02-scope-and-non-goals.md`, "Explicitly not in v1" and "Beyond
  v1".
- `docs/hld/03-architecture.md`, "The dependency rule" and "Why these seams".
- `docs/hld/04-opc-and-packaging.md`, "Relationship types", "Part naming",
  "Media", and "Package integrity".
- `docs/hld/06-presentationml-model.md`, "Public facade", "Preservation
  strategy", "Relationship remapping", and "Validation".
- `docs/hld/10-bindings-spec.md`, published native Rust API policy.
- `docs/hld/12-testing-strategy.md`, "Test taxonomy" and "The deck corpus".
- `docs/hld/14-development-backlog.md`, "F-215, Audio and video package
  model".

## Approach

Extend the existing concrete picture, timing, relationship, package, and facade
types. Do not create a second media object tree.

`CT_Picture` gains an optional typed projection of its non-visual audio or video
attachment. The projection retains its complete original XML as the
serialization source and exposes only the fields this story reads or edits:

```rust
pub enum MediaKind {
    Audio,
    Video,
}

pub enum MediaSource {
    Embedded { relationship_id: String },
    Linked { relationship_id: String },
}

pub struct PictureMedia {
    pub kind: MediaKind,
    pub source: MediaSource,
    pub poster_relationship_id: Option<String>,
}
```

Extend the existing timing module with concrete audio, video, common media
node, and media-command projections. They expose shape target, checked integer
trim bounds, volume in the schema's integer range, loop request, display
policy, and play, pause, stop, and seek commands. Unknown commands, attributes,
and siblings stay explicit and byte-preserved. Playback trigger order remains
owned by the existing timing tree rather than being duplicated on the picture.

The facade identifies a media object by slide index and `p:cNvPr/@id`, which is
already the stable timeline target. Add owned inspection values and atomic
mutations:

```rust
pub struct MediaInfo {
    pub slide_index: usize,
    pub shape_id: u32,
    pub kind: MediaKind,
    pub source: MediaLocation,
    pub poster_relationship_id: Option<String>,
    pub settings: MediaPlaybackSettings,
    pub diagnostics: Vec<MediaDiagnostic>,
}

pub enum MediaLocation {
    Embedded { part_name: String, content_type: String },
    Linked { target: String },
}

pub struct EmbeddedMediaInput<'a> {
    pub bytes: &'a [u8],
    pub filename: &'a str,
    pub content_type: &'a str,
}

pub enum MediaSourceInput<'a> {
    Embedded(EmbeddedMediaInput<'a>),
    Linked {
        target: &'a str,
        content_type: &'a str,
    },
}

pub struct MediaPoster<'a> {
    pub bytes: &'a [u8],
    pub filename: &'a str,
}

impl Presentation {
    pub fn media(&self, slide_index: usize) -> Result<Vec<MediaInfo>>;
    pub fn add_media(
        &mut self,
        slide_index: usize,
        kind: MediaKind,
        source: MediaSourceInput<'_>,
        poster: MediaPoster<'_>,
        left: Emu,
        top: Emu,
        width: Emu,
        height: Emu,
        settings: MediaPlaybackSettings,
    ) -> Result<ShapeRef<'_>>;
    pub fn replace_media(
        &mut self,
        slide_index: usize,
        shape_id: u32,
        source: MediaSourceInput<'_>,
    ) -> Result<()>;
    pub fn extract_media(&self, slide_index: usize, shape_id: u32) -> Result<Option<Vec<u8>>>;
    pub fn remove_media(&mut self, slide_index: usize, shape_id: u32) -> Result<()>;
}
```

Embedded additions require bytes, a filename, and an explicit content type.
The facade validates supported MP3, WAV, and ISO base media container signatures
without claiming a codec decoder. Unknown content types are accepted as opaque
payloads when the caller supplies a safe extension and MIME value, then remain
diagnosable. Linked sources retain their exact external target and use
`TargetMode="External"`.

Package mutation clones the package and slide model, allocates `/ppt/media/mediaN.ext`
with the existing `MediaNamer`, stages content types and both standard and
Microsoft media relationships, updates the picture and timing projections,
serializes, reparses, and commits only on success. Equal embedded bytes reuse
one package part after byte comparison inside a hash bucket. Poster images use
the existing image path and remain independently relationship-owned.

Replacement preserves the shape id, geometry, poster, and unrelated timing
siblings unless the caller replaces them explicitly. Removal deletes the
picture and its owned media timing nodes. Candidate embedded payload and poster
parts are removed only when no package relationship still reaches them.
Unsupported retained XML is rewritten through `rewrite_rel_ids` when slide
duplication or transfer assigns new ids.

No new trait, generic, feature, crate, module, file, or dependency is added.
The public change is additive for the pre-1.0 `rpptx-oxml`, `oxml-opc`, and
`rpptx` crates.

## Rejected alternatives

- Modelling media only in the facade would make save and relationship remapping
  depend on rescanning raw XML after every operation.
- Treating audio and video as image formats in `oxml-media` would mix package
  identity with image decoding and intrinsic-size policy.
- Rejecting unknown codecs would violate the preservation and diagnostics
  contract.
- Deleting every unreferenced `/ppt/media/` part after removal would destroy
  producer orphans that this mutation did not own.
- Replacing the complete timing subtree would lose unsupported producer
  commands and sibling order.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| round-trip | `embedded_audio_and_video_corpus_media_round_trip_without_duplication` | Exact media bytes, relationship types and targets, content types, poster ownership, playback settings, and unsupported metadata survive save and reopen. |
| round-trip | `media_model_reads_aliases_and_writes_schema_order_without_losing_raw_siblings` | Prefix-tolerant parsing, fixed-prefix writing, schema order, and byte-exact unmodelled subtrees. |
| integration | `add_replace_extract_and_remove_embedded_media_are_atomic` | Every facade operation updates picture, timing, relationships, parts, and content types together, and failure leaves bytes unchanged. |
| integration | `linked_media_keeps_external_targets_and_never_fetches_them` | Exact external target, target mode, poster, settings, and diagnostics survive mutation without network access. |
| regression | `unsupported_codec_bytes_remain_packaged_extractable_and_diagnosable` | Opaque bytes are retained exactly and do not acquire an implicit decoder. |
| regression | `media_removal_deletes_only_parts_owned_by_the_removed_relationships` | Shared payloads and unrelated orphan media survive while unreachable owned candidates are removed. |
| regression | `duplicated_media_slides_rewrite_every_retained_relationship_id` | Standard and extension relationship attributes point at transferred or deduplicated targets. |

The exact backlog **test gate is round-trip**: "Media bytes,
relationships, playback settings, and unsupported metadata survive save and
reopen without duplication."

Use the tracked `EmbeddedAudio.pptx` and `EmbeddedVideo.pptx` decks for corpus
coverage. Construct mutation fixtures in the existing
`crates/rpptx-oxml/tests/integration.rs` and
`crates/rpptx/tests/integration.rs` binaries. Do not add a new binary fixture or
integration binary.

## HLD impact

- `docs/hld/02-scope-and-non-goals.md`
- `docs/hld/03-architecture.md`
- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/06-presentationml-model.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`

## Risk routing

- Any parser or serialiser: re-read `docs/hld/04-opc-and-packaging.md` and
  `docs/hld/06-presentationml-model.md`. Add namespace-alias, fixed-prefix,
  schema-order, structural-reparse, and byte-exact unmodelled-subtree checks.
- Crate dependency graph and cross-family uses: keep media classification and
  naming in the dependency-free `oxml-media` leaf, package relationships in
  `oxml-opc`, and PresentationML projections in `rpptx-oxml`. Run
  `cargo tree -p rpptx -e normal` and the shared-crate dependency-direction
  test.
- Public API of published crates: state the additive pre-1.0 semver impact.
  Run publish dry runs for `rpptx-oxml`, `oxml-opc`, and `rpptx`, then assert
  every archive remains below 10 MiB.

## Hash harness

Expected unchanged, 49 of 49. The ordinary static samples do not exercise the
new opt-in mutation API. Any delta is unexplained and blocks integration.

## Implementation checklist

- [x] Add typed picture media and timing media projections without replacing
  retained raw XML.
- [x] Add audio, video, and Microsoft media relationship constants.
- [x] Add concrete inspection values and atomic facade mutations.
- [x] Reuse package-wide content hashing and collision-safe media naming.
- [x] Preserve linked targets without fetching them.
- [x] Keep unsupported codec bytes extractable and diagnostic.
- [x] Remove only relationship-owned candidates that become unreachable.
- [x] Extend duplicate and transfer relationship remapping to media.
- [x] Add corpus round-trip, source-built mutation, failure-atomicity, and
  orphan-preservation tests to existing binaries.
- [x] Run focused `rpptx-oxml`, `oxml-opc`, `oxml-media`, and `rpptx` checks,
  then every routed rider.

## Open questions

None. The caller-supplied filename and content type contract, shape-id
identity, required poster for new media, automatic and click trigger subset,
lexical timing preservation, replacement behavior, and no-fetch linked-media
policy are approved.
