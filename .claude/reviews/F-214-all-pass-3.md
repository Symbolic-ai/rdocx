# F-214, all, pass 3

**Reviewed**: complete stable pass 2 remediation diff against the F-214 worker
base, 11 implementation files, 3,565 additions and 29 deletions, including both
approved untracked timeline modules and the prior pass 2 review
**Verdict**: 12 defects, 1 smell, 0 nitpicks

## Defects

### D1, motion origins still use the wrong PowerPoint coordinate model

`crates/rpptx-layout/src/timeline.rs:95`

The resolver multiplies `parent` coordinates by the slide size and appends the
result as a translation, while it multiplies `layout` coordinates by the target
bounds. PowerPoint motion-path coordinates are expressed as percentages of the
slide. The origin changes the reference point: `parent` uses the slide's upper
left and `layout` uses the object's centre. A parent absolute point must
position against the slide reference rather than move the existing shape by
that amount, and a layout offset must not be scaled by the target width and
height. The revised regression encodes the same incorrect target-size layout
expectation. Pass 2 D1 is not closed.

### D2, common-node end conditions never affect an active interval

`crates/rpptx-layout/src/timeline.rs:376`

`declared_end` considers only the declared duration and a container child end.
It never reads `CommonTimeNode::end_conditions`. A supported finite end delay
therefore cannot stop a longer animation, and an unsupported shape-event end
trigger emits no diagnostic. This violates both deterministic active intervals
and the unsupported-trigger diagnostic contract.

### D3, an indefinite leaf consumes zero time in a sequence

`crates/rpptx-layout/src/timeline.rs:356`

Every leaf passes its own start as `child_end`. `declared_end` then returns that
same start for `TimingDuration::Indefinite`. The leaf remains active forever in
`phase`, but the sequence cursor advances immediately, so the following normal
or after-effect node starts at the same timestamp. An indefinite leaf should
block the sequence until a supported end condition resolves it.

### D4, group animation uses descendant ink bounds instead of the group extent

`crates/rpptx-layout/src/context.rs:3186`

The remediation reconstructs each group rectangle as the axis-aligned union of
its resolved descendant bounds. A PresentationML group owns an explicit
`a:xfrm` extent and child coordinate mapping. Descendants need not fill that
extent, and a rotated group cannot be represented by the union of page-space
axis-aligned leaf boxes. Scale and spin therefore use the wrong centre, parent
motion uses the wrong reference rectangle, and the page-space wipe at line
3239 loses the group's oriented coordinate space. The focused test uses an
empty group transform whose children define the entire union, so it does not
exercise this case. Pass 2 D6 is only partially closed.

### D5, F-214 makes a breaking change to the published F-213 API

`crates/rpptx-oxml/src/timing.rs:93`

`TimingCondition::target` changes from `TimingTarget` to
`Option<TimingTarget>`. This breaks every caller that constructs the public
struct or reads the field without an option match. The approved F-214 contract
says to consume the exact integrated F-213 values and limits public additions
to `rpptx-layout`, `rpptx-render`, and `rpptx`. Fixing unsupported condition
evaluation cannot silently revise the published `rpptx-oxml` model in this
story.

### D6, morph-name recovery adds a production dependency prohibited by the plan

`crates/rpptx-layout/Cargo.toml:25`

The pass 2 remediation adds `quick-xml` to the published `rpptx-layout`
production graph and changes `Cargo.lock`. The approved approach and risk rider
both state that F-214 adds no dependency. Being an existing workspace
dependency does not make this a dependency-free change to the crate's package
or graph.

### D7, a compatible morph shows incoming content at progress zero

`crates/rpptx-render/src/timeline.rs:292`

For a compatible explicit-name pair, the compositor clones the complete
incoming group and interpolates only its outer extent, transform, and opacity.
At progress zero the geometry is moved to the outgoing bounds, but fill,
stroke, text, image, effects, and child content are already the incoming
shape's. Two otherwise compatible rectangles with different fills therefore
jump to the new fill at the start instead of producing the outgoing endpoint or
a bounded crossfade. The focused morph test uses empty group children and the
oracle source gives its outgoing and incoming morph shapes the same fill, so
neither check can expose the jump.

### D8, morph option semantics are ignored

`crates/rpptx-render/src/timeline.rs:207`

The compositor reads only `transition.progress` and never branches on
`morph_option`. Valid `byWord` and `byChar` transitions therefore run the same
whole-shape explicit-name path as `byObject`. The bounded implementation may
support only object morphing, but unsupported word and character modes must
crossfade with a stable diagnostic rather than claim different semantics.

### D9, zoom ignores its in or out direction

`crates/rpptx-render/src/timeline.rs:111`

The transition direction is extracted at line 75, but the zoom branch never
uses it. Both valid `dir="in"` and `dir="out"` values fade the outgoing slide
and grow the incoming slide from half size. These are distinct standard zoom
transitions. The focused transition test supplies no parameters and asserts
only a finite two-group result, so it cannot distinguish them.

### D10, every outgoing transition frame is hardcoded to timestamp zero

`crates/rpptx/src/lib.rs:4604`

The facade resolves the supplied outgoing slide with
`TimelinePosition::default()`. A transition begins from the outgoing frame that
is visible when the caller advances, which can differ from the beginning of
that slide after entrance, emphasis, motion, exit, or click effects. The API
accepts no outgoing position, and hardcoding zero makes every such transition
compose from the wrong frame. The new regression proves only that the page and
state agree with each other at the chosen wrong timestamp.

### D11, diagnostics from the visible outgoing frame are discarded

`crates/rpptx/src/lib.rs:790`

The returned list includes incoming resolver diagnostics, incoming evaluator
diagnostics, and composition diagnostics only. The outgoing slide is now
timeline-resolved and rendered into the composed page, but both
`outgoing.slide.diagnostics` and `outgoing.state.diagnostics` are omitted. An
unsupported outgoing timing node, rejected transform, name-recovery failure,
or resolver approximation can affect visible output without reaching the
facade caller.

### D12, unsupported effect filters can execute as supported effects

`crates/rpptx-layout/src/timeline.rs:554`

Filter selection uses substring checks for `appear`, `wipe`, and `fade`.
Any unrecognised producer token containing one of those strings executes the
matching supported effect, and a wipe with an unrecognised direction silently
defaults to right. Unsupported filter syntax must remain diagnostic rather
than acquire approximate semantics by substring coincidence.

## Smells

### S1, name recovery still serializes and reparses every timeline shape

`crates/rpptx-layout/src/context.rs:2991`

The manual string parser and private entity decoder are gone, and failures now
produce stable diagnostics. However, timestamped resolution still serializes a
complete shape solely to parse one already-modelled non-visual name back out.
This couples morph identity to every per-kind serializer, allocates full XML
for every timeline leaf, and is the reason for D6. The pass 2 S1 hazard is
reduced but not eliminated.

## Nitpicks

None.

## Pass 2 closure

- Pass 2 D2, D3, D5, D7, D8, D9, and D10 are closed by focused code and
  regressions. Relative commands accumulate, unsupported position attributes
  and effect transition modes diagnose, arithmetic stays finite, static
  methods use the identity-free path, custom geometry is compared structurally,
  outgoing pages and states are internally coherent, and valid effect
  transition modes no longer fall through as entrances.
- Pass 2 D1 remains open as this pass D1.
- Pass 2 D4 fixes the evaluator distinction, but does so through the
  out-of-contract public API change in this pass D5.
- Pass 2 D6 is partial because one transform is shared across descendants, but
  this pass D4 derives its coordinate space from the wrong rectangle.
- Pass 2 S1 fixes string parsing, entity decoding, and swallowed failure, but
  the serialize-and-reparse indirection remains this pass S1 and adds the
  prohibited dependency in this pass D6.
- Focused checks passed: 6 `rpptx-layout` timeline unit tests, the group-space
  regression, the namespace-aware name regression, 5 `rpptx-render` timeline
  unit tests, the unsupported-condition `rpptx-oxml` regression, the outgoing
  morph facade regression, and the static-path integration regression. This is
  16 focused tests. `git diff --check` also passed. No broad test command ran.

## External evidence blocker

The PowerPoint oracle directory contains presentation attempts but still has
no `manifest.tsv` and no PNG frames. GUI automation failed, so the required
pinned PowerPoint differential has not run and must not be reported as passed.
Required-corpus mode remains correctly fail-closed at the missing manifest.
The full non-GUI 50-deck SSIM rider was interrupted and is also unclaimed.
These external evidence gaps are separate from the implementation findings
above.

## Not found

No additional production panic path was found. The new `unreachable!` and
`expect` sites remain guarded. No schema child-order, namespace preservation,
retained raw-XML, reverse family dependency, new trait, generic, feature flag,
crate, backend animation variant, runtime oracle dependency, or binary fixture
defect was found. Ordinary static resolution now avoids identity recovery and
timeline evaluation. Slide targets still use `p:cNvPr/@id`, containing-group
lineage is retained, and master and layout target scopes stay excluded from
slide timing. The revised oracle harness remains fail-closed in required mode,
but its absent external artifacts provide no fidelity evidence.
