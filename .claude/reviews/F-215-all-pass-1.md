# F-215, all, pass 1

**Reviewed**: complete working diff against the F-215 worker base, 8 files,
2,406 additions and 5 deletions, including the approved design, cited HLD
sections, progress notes, both pinned Apache POI media decks, and every default
microscope aspect
**Verdict**: 9 defects, 1 smell, 0 nitpicks

## Defects

### D1, media trim is read and authored at the wrong OOXML location

`crates/rpptx-oxml/src/timing.rs:532`

`crates/rpptx-oxml/src/timing.rs:1251`

`crates/rpptx-oxml/src/picture.rs:815`

PowerPoint stores trim bounds as the `st` and `end` attributes of the Office
2010 `p14:trim` child under `p14:media`. The picture projection stops after
reading the relationship attributes from `p14:media`, while the timing parser
instead searches for `trimStart`, `trimEnd`, `st`, and `end` on
`p:cMediaNode`. New media is authored with `trimStart` and `trimEnd` attributes
on that common timing node. As a result, valid producer trim values are never
reported, and newly supplied trim values are written outside the extension
schema. Valid `p14:trim` values can also carry fractional milliseconds, which
the current `u64` parser cannot project. The approved trim-range contract is
therefore neither readable nor writable in its actual schema location.

### D2, playback trigger inspection discards the timing-tree order that owns it

`crates/rpptx/src/lib.rs:2739`

`crates/rpptx/src/lib.rs:2761`

`crates/rpptx-oxml/src/timing.rs:441`

The facade collects media commands as flat references and derives the trigger
only from the command's own `p:cTn`. It does not retain or inspect the ancestor
sequence, parallel container, start condition, or ancestor `nodeType`. A valid
`playFrom` command nested beneath a `clickEffect` node therefore has no local
click facts and is reported as `Automatic`. This is the producer shape used by
the pinned audio and video decks. Other valid automatic and interactive
sequence placements can likewise collapse to the default `OnClick`. The
approved contract says playback trigger order remains owned by the existing
timing tree, but inspection removes that context before classifying it.

### D3, adding media can put `p:tnLst` after later schema children

`crates/rpptx-oxml/src/timing.rs:591`

When a timing root has no `p:tnLst`, the insertion path writes the new list
immediately before the closing `p:timing` tag. `CT_Timing` orders `p:tnLst`
before `p:bldLst` and `p:extLst`. A valid producer root containing only a build
list or extension list is therefore rewritten as `p:bldLst`, then `p:tnLst`,
which violates the declared sequence and can make PowerPoint reject the slide.
The alias round-trip test covers an existing list and cannot exercise this
branch.

### D4, timing insertion can select an unmodelled same-local-name element

`crates/rpptx-oxml/src/timing.rs:611`

`find_start_tag` matches only the local bytes `tnLst` or `timing`. It neither
tracks namespace bindings nor verifies that the selected element is the
PresentationML child of the current timing root. If retained producer XML
contains an earlier `<x:tnLst>` in another namespace, `add_media` inserts the
new PresentationML nodes into that unmodelled subtree. This changes unrelated
producer content and leaves the real timing tree unchanged, contrary to the
namespace-aware parsing and byte-preservation riders.

### D5, replacing one shape rewrites every user of its relationship ids

`crates/rpptx/src/lib.rs:2461`

`crates/rpptx/src/lib.rs:2481`

`replace_media` removes the old standard and Microsoft relationships, then
runs `rewrite_rel_ids` over the complete serialized slide. Relationship ids are
scope-wide resources and may be referenced by more than one picture or by
retained producer XML. If two shapes share either old id, replacing one shape
retargets both shapes to the new payload. Any other retained reference changes
as well. The shape-id API promises replacement of one media object while
preserving unrelated content, so the rewrite must be limited to the targeted
picture or shared references must keep the old relationship.

### D6, removal deletes relationships that retained slide XML may still use

`crates/rpptx/src/lib.rs:2542`

`crates/rpptx/src/lib.rs:2557`

Removal treats all three ids found on the selected picture as exclusively
owned and deletes those relationship records without checking the remaining
slide XML. A producer may legally reuse one image, audio, video, or Microsoft
media relationship id from multiple elements. Removing one such picture then
leaves every retained reference to the shared id dangling. The regression
creates two shapes that share payload bytes but receive distinct relationship
ids, so it proves part reachability but not relationship ownership.

### D7, byte deduplication can discard the caller's explicit media metadata

`crates/rpptx/src/lib.rs:514`

The explicit media insertion path reuses the first byte-identical entry in the
package-wide media hash bucket without comparing its extension or content
type. That bucket also contains poster images and producer media with other
metadata. Supplying bytes that already exist as `image1.png`, or supplying the
same opaque bytes under two incompatible MIME values, returns the old part and
silently ignores the requested safe extension and explicit content type. The
resulting media relationship can therefore resolve to an image part or to a
part carrying the wrong MIME value. Collision-safe byte comparison is present,
but the metadata compatibility required by an OPC part is not.

### D8, invalid MIME strings pass the safe-content-type contract

`crates/rpptx/src/lib.rs:2815`

Validation requires only a slash and the absence of ASCII control characters.
Values such as `/`, `audio /mpeg`, or `not a/type with spaces` pass and are
written into `[Content_Types].xml`. Those are not valid media types under the
OPC content-type grammar. The approved opaque-codec path requires a safe MIME
value, so malformed caller metadata must fail before package mutation rather
than producing a package whose content type is not interoperable.

### D9, the declared round-trip gate self-compares incomplete observations

`crates/rpptx/tests/integration.rs:289`

`crates/rpptx/tests/integration.rs:309`

`crates/rpptx/tests/integration.rs:325`

The corpus gate compares reopened `MediaInfo` with the same implementation's
pre-save `MediaInfo`, counts media parts, and checks only that two element names
occur. It never asserts the expected playback settings from the source decks,
the standard and Microsoft relationship types and targets, content types,
poster relationship ownership, or any unsupported metadata bytes. The trigger
misclassification in D2 therefore passes before and after save, and removing
or changing one of the paired relationships can also pass while the remaining
relationship still supports inspection. This does not prove the exact backlog
gate stated in the approved test plan.

## Smells

### S1, format-neutral audio and video classification lives in the facade

`crates/rpptx/src/lib.rs:2832`

The MP3, WAV, and ISO base media signature rules are implemented directly in
`rpptx`, while the routed design assigns media classification and naming to the
dependency-free `oxml-media` leaf. `crates/oxml-media/README.md:3` already
describes that crate as the media-format detection seam, but neither its API nor
its documentation changed. F-216 and F-227 will need the same classification
facts, so leaving them private to the Presentation facade invites a second
sniffer and inconsistent codec diagnostics.

## Nitpicks

None.

## Not found

- Panics: the added production indexing and `expect` sites are dominated by
  validated slide indices, fixed-size local construction, or parser-owned root
  events. No reachable untrusted-input panic was found.
- Required poster and ordinary atomicity: the public add signature requires a
  poster, validates image bytes, stages mutations on a clone, serializes,
  reopens, and commits only after success.
- Linked media: external targets are retained exactly, use `TargetMode` equal
  to `External`, and are never fetched.
- Public API and layout scope: the new surface is additive for the published
  pre-1.0 crates. The `rpptx-layout` match extension keeps the completed static
  evaluator exhaustive without introducing a second media model.
- Structure beyond S1: no new crate, module, file, dependency, trait, generic,
  forwarding wrapper, builder, or feature flag was added.
