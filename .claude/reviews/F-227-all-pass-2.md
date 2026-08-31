# F-227, all, pass 2

**Reviewed**: complete remediated working tree against the F-227 worker base,
7 implementation files, 2,075 changed lines including the approved untracked
animation module
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, the two-machine gate does not pin any AVI identity
`crates/rpptx/tests/integration.rs:594`

The independent parser now calculates AVI payload hashes and decoded JPEG
frame hashes, but the golden compares none of them to reviewed constants. It
also never calculates or compares an AVI container hash and never asserts the
AVI export diagnostics. Lines 595 to 604 check fixed header facts and only
require that some decoded frames differ. Lines 606 to 630 require quality 40
to differ from quality 80, but either quality may produce arbitrary
platform-specific bytes and pixels. The exact test can therefore pass on both
machines when macOS and Linux produce different Motion JPEG payloads,
containers, or decoded frames. This fails the approved exact frame identity,
container hash, diagnostics, and two-machine determinism contract for the AVI
format.

## Smells

None.

## Nitpicks

None.

## Not found

- Pass 1 D1 is remediated. `prepare_render_context` performs one static package
  and resource preparation, and each sample resolves and drops on demand. The
  50-frame regression records one preparation and no more than one active
  sample.
- Pass 1 D2 is remediated. GIF and JPEG write through `CappedBuffer`, which
  rejects a write before crossing the cap. AVI writes frame chunks into one
  seekable buffer, patches sizes in place, and retains only one 16-byte index
  record per bounded frame. Both codecs have a passing mid-encode cap case.
- Pass 1 D3 is otherwise remediated. The independent AVI parser validates RIFF
  and LIST sizes, exact chunk boundaries, zero padding, `avih`, `strh`, `strf`,
  rate and scale, frame counts, dimensions, suggested buffer sizes, `movi`,
  every index offset, flag and unpadded size, JPEG dimensions, decoded pixels,
  quality response, and the derived 600 ms duration.
- Pass 1 D4 is remediated. The source-built fixture has distinct outgoing and
  incoming slides, a real fade, a click-triggered media object, and a separate
  click-controlled shape. The integration test proves media phase, evaluated
  shape state, changed raster pixels, transition sampling, and ordered
  per-frame diagnostics. The GIF golden pins six non-identical frame hashes.
- Sampling and validation produced zero findings. Exact ceiling frame counts,
  floor timestamps, segment boundaries, fixed click state, explicit outgoing
  slides, checked duration arithmetic, missing-slide rejection, dimensions,
  quality, per-frame pixels, total pixels, frame count, and preflight output
  caps remain closed before package preparation and rendering.
- Panics produced zero findings. Production indexing and arithmetic introduced
  by F-227 are bounded or checked. The GIF delay `expect` is protected by the
  validated frame-rate cap, and AVI field casts are protected by dimension and
  frame-count caps.
- GIF and raster output produced zero findings. Cumulative delays retain the
  required 30 fps cadence, total-play mapping subtracts only the initial play,
  opaque composition retains the page background, exact output dimensions are
  enforced, and pure-Rust quantization is covered by the exact GIF container
  and decoded-frame manifest.
- Diagnostics ordering produced zero findings. Each sample retains incoming
  F-214 diagnostics, outgoing F-214 diagnostics, F-216 package and playback
  diagnostics, and transition-composition diagnostics in the established
  order.
- Public API and dependency boundaries produced zero findings. The additive
  pre-1.0 values remain under the existing `render` feature in `rpptx`.
  `gif`, `jpeg-encoder`, and `tiny-skia` are optional facade dependencies, the
  no-default library check passes, and no shared crate depends on a format
  crate.
- CI routing and recorded platform execution produced zero findings. The exact
  golden command is present in both the Ubuntu workspace job and the macOS
  presentation-fidelity job under Rust 1.97.1-era runners. The progress record
  names the passing macOS command and two Linux arm64 executions from the same
  working source, with Linux reporting Rust 1.97.1. The repository and current
  review environment are pinned to Rust 1.97.1.
- Static isolation and structure produced zero findings. The static PDF and
  raster regression passes, the hash baseline is unchanged, and the approved
  private module adds no feature, trait, generic, wrapper, builder, crate,
  integration binary, or binary asset.
