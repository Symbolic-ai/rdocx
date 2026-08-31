# F-227, all, pass 1

**Reviewed**: complete working tree against the F-227 worker base, 7 files,
1,335 changed lines including the approved untracked animation module
**Verdict**: 4 defects, 0 smells, 0 nitpicks

## Defects

### D1, the exporter retains every resolved frame before encoding
`crates/rpptx/src/lib.rs:5392`

The prepared assembly allocates one `TimelineFrameAssembly` for every sample,
then stores a full incoming and optional outgoing `ResolvedTimelineSlide` in
each entry at lines 5523 and 5553. The animation facade supplies the complete
request vector before either encoder starts at
`crates/rpptx/src/animation.rs:113`. A valid 10,000-frame request therefore
retains 10,000 resolved slide trees, text directions, media states, and
diagnostics before the first raster frame is emitted. The pixel and output
caps do not bound the size of those resolved trees. A complex but valid slide
can exhaust memory even though its requested raster and encoded output are
within every declared cap. This does not meet the approved one-frame streaming
contract.

### D2, encoded output is capped only after unrestricted full buffering
`crates/rpptx/src/animation.rs:303`

The GIF encoder writes to an unrestricted `Vec<u8>` and checks the byte cap
only after every frame has been encoded. The AVI path likewise accumulates the
complete `movi` body at line 328, copies it into a second complete `avi` buffer
at line 494, copies that into the returned buffer at line 501, and reaches its
cap check only at line 372. The preflight calculation at line 247 is a raw
pixel estimate, not a limit enforced by either writer. A valid near-cap export
can allocate multiple complete encoded bodies, and an encoder result above the
declared cap is rejected only after that oversized allocation has happened.
The fixed output-byte cap and streaming requirement must constrain writes as
they occur.

### D3, the AVI gate records bytes without proving the AVI contract
`crates/rpptx/tests/integration.rs:212`

The two-machine golden checks only timestamps, two file tags found anywhere,
a count of every `00dc` byte sequence, and a whole-file hash. The unit case at
`crates/rpptx/src/animation.rs:619` similarly searches for byte patterns rather
than parsing fields at their required offsets. Neither test independently
checks the RIFF and LIST sizes, padding, `avih` frame count and dimensions,
`strh` rate and scale, stream length, `idx1` offsets, flags and unpadded sizes,
decoded JPEG dimensions and frame hashes, declared quality behavior, or AVI
duration. Payload bytes can also contain the searched tags. Recording a hash
of the current writer can freeze a malformed container, so this does not
satisfy the approved Motion JPEG AVI unit or golden contracts.

### D4, the transition and click integration case makes both inputs inert
`crates/rpptx/tests/integration.rs:54`

The fixture contains one slide and configures its media trigger as automatic
at `crates/rpptx/tests/integration.rs:8469`. It adds no slide transition. The
test then supplies click count 1 and names that same slide as its own outgoing
source at line 61. Click count cannot affect automatic playback, and an
outgoing page cannot be composed when the incoming slide has no transition.
The assertions at lines 75 to 95 cover timestamps, a GIF signature, and
repeated diagnostics only. They never compare decoded frame pixels or a
synchronized media state. The separate golden at line 183 uses a blank slide
whose three frame hashes are identical. A broken click advance or transition
sampler would pass all of this evidence, contrary to the approved integration
and frame-identity gates.

## Smells

None.

## Nitpicks

None.

## Not found

- Panics produced zero findings. The internal GIF-delay `expect` is protected
  by the validated frame-rate cap, and the frame index is derived from the
  prepared sample sequence.
- OOXML produced zero findings. This diff changes no parser, serializer,
  namespace handling, schema ordering, or unmodelled subtree preservation.
- Sampling arithmetic produced zero additional findings. Segment frame counts
  use exact ceiling arithmetic, local timestamps use exact floor arithmetic,
  segment bounds are checked, and timestamp and duration additions are checked.
- GIF semantics produced zero additional findings. Cumulative delays implement
  the required 30 fps 3, 3, 4 cadence, and total-play requests map to the
  Netscape repeat count after subtracting the initial play.
- Raster composition produced zero additional findings. Frames are composited
  to opaque RGBA and resized to the exact requested dimensions before either
  deterministic pure-Rust encoder consumes them.
- Diagnostics produced zero additional findings. Incoming F-214 diagnostics,
  outgoing F-214 diagnostics, F-216 package and playback diagnostics, and
  composition diagnostics remain in frame and sample order.
- Public boundaries produced zero findings. The additive pre-1.0 API and both
  optional codec dependencies remain under the existing `render` feature in
  the `rpptx` facade. No shared crate gains a format dependency, and no new
  feature, trait, generic, crate, integration binary, or binary asset appears.
- CI routing produced zero findings. The exact golden command is present in
  both the Ubuntu workspace job and the macOS presentation-fidelity job, and
  changes under `crates/**` select both jobs.
