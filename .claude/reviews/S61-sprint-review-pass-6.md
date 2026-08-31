# S61 sprint review, pass 6

**Reviewed**: sprint/s61 at
`c36ea3aecadaaee6649156ec51f2a6ca6a2fc229` against
`7c7742616134ec34aebd6aef945f8682e70d19fd`, 56 files, 24,791 changed
lines, crates: oxml-media, oxml-opc, rpptx-layout, rpptx-oxml,
rpptx-render, rpptx-wasm, rpptx
**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have
**Clean status**: clean
**Disposition**: no fix-now, tracked-follow-up, or human-action finding
remains. Pass-5 B1 and S1 are refuted by the cited remediation and evidence.

**Bound extension**: explicitly extended. Global pass 6 exceeds the configured
three passes only because earlier scheduled dependency-prefix boundaries
consumed passes. This is pass 3, the last allowed pass within the scheduled
final-closure boundary. The recorded extension reason is `scheduled
dependency-prefix boundary`.

## Blocking

None.

## Should-fix

None.

## Nice-to-have

None.

## Prior-pass dispositions

Pass-5 B1 is refuted. Sprint state records full verification as passing with
49 of 49 harness entries unchanged at the exact reviewed HEAD at
`.claude/scratch/S61-run.json:134`. This satisfies the S61 full-verification
clause at `docs/sprints/CURRENT_SPRINT.md:62` for the implementation state
reviewed here.

Pass-5 S1 is refuted. The two cross-crate hooks are marked `#[doc(hidden)]` at
`crates/rpptx-render/src/lib.rs:260` and
`crates/rpptx-render/src/timeline.rs:42`. Their documentation names `rpptx` as
the internal facade consumer and states that the same manager must assign the
stored `FontId` values and supply the raster font data at
`crates/rpptx-render/src/lib.rs:254` and
`crates/rpptx-render/src/timeline.rs:37`. An ordinary
`cargo doc -p rpptx-render --no-deps` build omits both hooks from the crate
index, module index, all-items page, and search index.

This matches established cross-crate internal-hook treatment. The private media
diagnostic identity hook uses the same hidden-public pattern at
`crates/rpptx-layout/src/context.rs:164`, and reusable font-engine hooks use it
at `crates/oxml-layout/src/font.rs:794`. Repository-wide call-site inspection
finds the new hooks only in the compatibility wrappers within `rpptx-render`
and in the native facade assembly at `crates/rpptx/src/lib.rs:88`. The public
facade regression exercises the supported consumer at
`crates/rpptx/tests/integration.rs:141`. No Python, WASM, or CLI source imports
or exposes either hook or the animation facade.

Pass-4 B1 remains refuted. `PreparedRenderAssembly` retains one deterministic
manager at `crates/rpptx/src/lib.rs:5443`. Incoming and outgoing resolution,
dynamic Audio or Video fallback shaping, page lowering, and rasterization all
use that manager and its retained font table at
`crates/rpptx/src/lib.rs:2648`, `crates/rpptx/src/lib.rs:2700`,
`crates/rpptx/src/lib.rs:2719`, and `crates/rpptx/src/animation.rs:359`. The
unit identity regression covers Caladea, Liberation Sans, and the dynamic
Carlito `Video` label at `crates/rpptx/src/animation.rs:857`, while the public
facade regression covers repeated GIF and AVI pixels at
`crates/rpptx/tests/integration.rs:203`.

Pass-4 B2 remains refuted. Timeline lowering consumes the selected evaluated
slide directly without cloning `RenderInput` at
`crates/rpptx-render/src/timeline.rs:56`, and animation constructs its
deterministic manager once at `crates/rpptx/src/lib.rs:5555`. The 50-frame
regression asserts one package, resolver, media, layout, and font preparation
and at most one active resolved sample at
`crates/rpptx/src/animation.rs:820`. Source inspection finds no per-sample
`RenderInput` clone or deterministic font-database construction in the
animation path.

Pass-4 S1 remains refuted. The completion ledger assigns timeline position,
click count, and transition source to each segment while assigning one fallback
policy to the whole export at `docs/sprints/AS_BUILT.md:10492`.

## Milestone gate

The M21 gate is: "one representative modern deck round-trips its comments,
sections, SmartArt, media, animation timeline, signatures, and package variant
without repair. Its static frames, animated export, notes, and handouts match
the pinned PowerPoint oracle at their declared fidelity boundaries."

The M21 gate does not yet hold, as expected before F-218 through F-220, F-222,
F-223, and F-226 complete. Its contract remains at
`docs/hld/14-development-backlog.md:1896`. This is not an S61 closure finding
because those named stories belong to later milestone sprints.

The S61 gate holds at the reviewed implementation HEAD. F-215 has exact corpus
package round-trip evidence at `docs/sprints/AS_BUILT.md:10408`. F-216 has the
deterministic 150 dpi poster, fallback, playback, diagnostic, and legacy-path
evidence at `docs/sprints/AS_BUILT.md:10463`. F-227 binds one unchanged exact
GIF and AVI identity to macOS and Linux arm64 at
`docs/sprints/AS_BUILT.md:10518`, and the locked golden runs in both CI jobs at
`.github/workflows/ci.yml:193` and `.github/workflows/ci.yml:413`.

Focused pass-6 execution passed both pass-4 remediation unit regressions, the
renamed public-facade GIF and AVI regression, the exact two-format manifest,
the exact F-216 poster and playback golden, the legacy diagnostic regression,
all 97 `rpptx-render` tests, the rpptx-wasm manifest regression, and both
default and render-enabled WASM target checks. The generated public-doc search
found neither hidden hook. Exact-HEAD full verification, including the
workspace, rustdoc, packaging, supply-chain checks, archive ceilings, and the
unchanged 49-entry harness, is recorded in sprint state.

## Not found

Interaction produced zero findings. F-227 consumes the completed F-214
transition and F-216 media paths through one shared assembly. Incoming and
outgoing samples retain their slide identities, click schedule, playback state,
fallbacks, and ordered F-214, F-215, and F-216 diagnostics.

Duplication produced zero findings. Media classification, package mutation,
timing evaluation, timeline lowering, sampling, and the GIF and AVI writers
each retain one implementation owner. The hidden hooks expose the existing
lowering implementation to the facade rather than create another path.

Layering produced zero findings. `oxml-media` remains dependency-free. No
`oxml-*` crate gained an `rdocx-*` or `rpptx-*` dependency, and codec knowledge
remains at the `rpptx` facade edge.

Harness produced zero findings. Every plan and completion record declares 49
of 49 unchanged, no baseline file changed, and the exact-HEAD full verification
record agrees.

Gate produced zero findings. Sampling arithmetic, fixed click state, explicit
outgoing sources, streaming, frame and pixel caps, capped codec writes,
in-place RIFF patching, bounded index retention, immediate sample release,
cumulative GIF delays, diagnostic order, exact dimensions, and deterministic
font use retain executable coverage.

Docs and HLD impact produced zero findings. The sprint changed exactly the
union of the three approved HLD impact lists. The package, media, resolver,
renderer, native facade, CI, and deterministic-container descriptions match
the implementation. The hidden inter-crate hooks do not add a supported public
surface that the binding specification must inventory.

Dependencies produced zero findings. `gif` 0.14.2 has the named animated GIF
consumer, while the existing pinned JPEG encoder and tiny-skia have the Motion
JPEG and raster consumers. No remediation dependency, crate, feature, trait,
generic, module, integration binary, or binary asset was added.

Native and WASM isolation produced zero findings. The d754050 regression
asserts the complete native render dependency set and its absence from the
default WASM graph at `crates/rpptx-wasm/src/lib.rs:462`. Both WASM target
profiles pass, and the JavaScript, Python, and CLI surfaces gain no animation
method or hidden renderer hook.

Surface compatibility produced zero findings. Existing supported
`rpptx-render` entry points retain their signatures at
`crates/rpptx-render/src/lib.rs:231` and
`crates/rpptx-render/src/timeline.rs:23`. The new same-manager hooks are hidden
implementation seams, generated public docs exclude them, and the supported
animation consumer remains the approved native `rpptx` facade.

Diagnostics and legacy compatibility produced zero findings. Package, timing,
media, and composition diagnostics retain their stable order. Fallback
replacement remains source scoped, static PDF and raster bytes remain
unchanged, and the existing static and one-frame timeline entry points retain
their exact pages and diagnostic strings.

OOXML and payload preservation produced zero findings. F-215 retains
namespace-tolerant reads, fixed-prefix schema order, direct relationship
ownership, raw siblings, timing targets, relationship-id remapping, atomic
mutation, and opaque unsupported media bytes. F-216 and F-227 add no parser or
serializer path, and audio or video payloads never enter renderer media.

Delivery records produced zero findings. F-215, F-216, and F-227 are done and
unowned in the current sprint and backlog, each has one tracker row and one
AS_BUILT entry, all handoffs are consumed, pass 5 is recorded against its
review commit, and full verification is recorded at the exact pass-6
implementation HEAD.
