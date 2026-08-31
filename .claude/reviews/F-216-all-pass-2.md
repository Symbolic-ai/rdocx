# F-216, all, pass 2

**Reviewed**: working-tree diff, 5 files, 1,491 insertions and 29 deletions
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, poster diagnostic identity is not source-scoped
`crates/rpptx/src/lib.rs:5607`

The replacement key contains only the shape id, even though shape ids are
local to the slide, layout, or master shape tree. The resolver emits master and
layout shapes before slide shapes, and its new prefix also omits the source at
`crates/rpptx-layout/src/context.rs:671`. If an unresolved inherited media
picture and the slide media picture share a legal shape id, fallback handling
replaces the inherited diagnostic and leaves the slide poster diagnostic
unchanged. The fallback diagnostic must be matched with source-scoped object
identity, not shape id alone.

### D2, diagnostic tagging changes the existing static and timeline paths
`crates/rpptx-layout/src/context.rs:663`

The shared resolver now rewrites every unresolved media-picture diagnostic
before it knows whether the caller selected the new media-aware entry point.
Consequently `render_deterministic` and `render_timeline_deterministic` return
different diagnostic text for an unresolved media poster even though the
approved contract keeps those existing entry points unchanged and applies the
new fallback policy only through `render_media_timeline_deterministic`. The
object-precise replacement metadata must stay inside the media-aware assembly
path or otherwise avoid changing legacy observable diagnostics.

## Smells

None.

## Nitpicks

None.

## Not found

Pass-1 seek handling now preserves a stopped seek through offset-free play,
while initial and explicit stop states still restart at the trim boundary.
Finite non-looping playback now stops at exact trim-end and known-duration
boundaries. The ordinary sibling diagnostic regression covers the immediate
slide case, and exact independent Audio and Video fallback RGBA hashes are
recorded at 150 dpi with deterministic fonts.

No additional findings were found for command source order, click ordinals,
checked playback arithmetic, loop behavior, volume normalization, fallback
policy reachability, renderer admission of payload bytes, poster content-type
validation, one-assembly result construction, panic safety, OOXML preservation,
or repository structure.
