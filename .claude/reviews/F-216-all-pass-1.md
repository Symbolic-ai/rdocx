# F-216, all, pass 1

**Reviewed**: working-tree diff, 4 files, 1,239 insertions and 29 deletions
**Verdict**: 4 defects, 0 smells, 0 nitpicks

## Defects

### D1, play discards a seek performed while stopped
`crates/rpptx-layout/src/timeline.rs:358`

A `Play` command without an explicit offset resets the source position to the
trim start whenever the current phase is stopped. A preceding `Seek` changes
the source position but leaves the phase stopped, so a source-ordered
`seek(0.2)` followed by `play` starts at the trim boundary instead of 200 ms.
Initial state and `Stop` already set the trim boundary, so this branch breaks
the distinct seek-then-play case.

### D2, finite non-looping playback never reaches the stopped phase
`crates/rpptx-layout/src/timeline.rs:617`

Position advancement receives the phase by value and can only clamp the source
position. When a playing, non-looping media object reaches a finite trim end or
known duration, the returned position remains at the end while the public phase
remains `Playing` forever. The exact end boundary must transition to `Stopped`
so exporters do not treat completed media as active indefinitely.

### D3, media fallback can remove an unrelated picture diagnostic
`crates/rpptx/src/lib.rs:5594`

For an unresolved media poster, fallback handling removes the first slide
diagnostic whose text looks picture-related. The search is not associated with
the media shape id or its poster relationship. If an ordinary missing picture
precedes the media object, its diagnostic is removed while the media poster
diagnostic remains, changing an unrelated sibling and producing the wrong
per-object diagnostic set.

### D4, the labelled fallback is absent from the required 150 dpi golden gate
`crates/rpptx/tests/integration.rs:8526`

The only decoded RGBA hash in the F-216 golden test is for a valid poster. The
deterministic `Audio` and `Video` fallback is checked only by extracting label
text elsewhere. A change to its bundled font selection, glyph placement,
clipping, or pixels would therefore leave the named golden gate green. The
approved risk rider requires every labelled fallback baseline to use
deterministic fonts at fixed 150 dpi, and the test plan says exact golden values
cover the fallback case.

## Smells

None.

## Nitpicks

None.

## Not found

No additional findings were found for panic safety, OOXML namespace or child
order, public API compatibility, renderer admission of audio or video payload
bytes, poster relationship scoping and content-type validation, fallback
policy reachability, volume normalization, one-assembly result construction,
or the repository structural rules.
