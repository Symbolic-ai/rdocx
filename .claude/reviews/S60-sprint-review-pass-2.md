# S60 sprint review, pass 2

**Reviewed**: sprint/s60 against ece49e64fd8e3ea94ec7eb47a417a0742e522cec, 53 files, 14,086 lines, crates: rpptx-oxml, rpptx-layout, rpptx-render, rpptx
**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have

## Blocking

None.

## Should-fix

None.

## Nice-to-have

None.

## Milestone gate

The M21 gate is: "one representative modern deck round-trips its comments,
sections, SmartArt, media, animation timeline, signatures, and package variant
without repair. Its static frames, animated export, notes, and handouts match
the pinned PowerPoint oracle at their declared fidelity boundaries."

The M21 gate does not yet hold. Its exact contract is recorded at
`docs/hld/14-development-backlog.md:1896`, and the backlog still assigns media,
SmartArt, package variants, animated export, notes export, and handout export to
pending stories at `docs/sprints/BACKLOG.md:414`,
`docs/sprints/BACKLOG.md:418`, `docs/sprints/BACKLOG.md:422`, and
`docs/sprints/BACKLOG.md:425`. S60 completes the timing and transition slice
without claiming that the later representative-deck work is done.

The S60 sprint gate does hold. The F-213 story gate requires the corpus timeline
to parse into the declared model, serialize in schema order, and preserve every
unsupported sibling at `docs/hld/14-development-backlog.md:1905`. The named
corpus gate parses, serializes, reparses, and structurally compares every slide,
layout, and master at `crates/rpptx-oxml/tests/integration.rs:537`, compares
retained timing and transition source bytes at
`crates/rpptx-oxml/tests/integration.rs:647`, and compares the unsupported-node
inventory at `crates/rpptx-oxml/tests/integration.rs:669`. Its nonzero 50-deck
coverage and completed integrated verification are recorded at
`docs/sprints/AS_BUILT.md:10305`.

The F-214 story gate requires pinned timestamps to match the PowerPoint frame
oracle within the declared geometric and pixel tolerances at
`docs/hld/14-development-backlog.md:1913`. This review independently reran
`pinned_timestamps_match_powerpoint_frame_oracle_within_declared_tolerances` in
required-corpus mode against the retained SHA-bound PowerPoint 16.104 oracle at
reviewed HEAD 68cc3160394c60f723214cd96902eb4af5c2817e. All nine cases passed.
Maximum geometry error was 0.96 point and minimum SSIM was 0.997866, within the
1 point and 0.99 limits. The test binds provenance and artifact identity at
`crates/rpptx/tests/integration.rs:6751`, re-extracts each exact movie sample and
requires byte identity at `crates/rpptx/tests/integration.rs:6860`, and enforces
the two quantitative limits at `crates/rpptx/tests/integration.rs:7011`. The
recorded hashes, nine-case scope, and aggregate measurements agree with the HLD
at `docs/hld/12-testing-strategy.md:734` and the completion record at
`docs/sprints/AS_BUILT.md:10356`. Zoom is correctly limited to Rust regression
evidence because the pinned movie interval is unobservable, as documented at
`docs/hld/12-testing-strategy.md:749`.

The ordinary static-output condition also holds. This review independently
reran `ordinary_static_rendering_does_not_execute_the_timeline`, whose exact PDF
and raster comparisons are at `crates/rpptx/tests/integration.rs:6688`, and
reproduced all 49 unchanged hash-harness entries. The completion record reports
the same unchanged result at `docs/sprints/AS_BUILT.md:10368`. The broader
50-deck SSIM result remains accurately labelled trend evidence with
`target_met=false` at `docs/hld/12-testing-strategy.md:919`, so it is not being
substituted for the still-open M21 fidelity gate. This review ran only the
focused F-214 differential, static-isolation regression, and hash harness, not a
broad test suite.

## Not found

Interaction produced zero findings. F-214 consumes the F-213 typed timing model
through the additive layout evaluator and preserves the strict dependency order
declared at `docs/sprints/CURRENT_SPRINT.md:38`. Slide, layout, and master target
identity stays source-scoped at `crates/rpptx-layout/src/context.rs:582`, while
the timeline facade returns the same evaluated incoming state used to compose
the page at `crates/rpptx/src/lib.rs:762` and
`crates/rpptx/src/lib.rs:805`.

Duplication produced zero findings. Timeline evaluation has one implementation
at `crates/rpptx-layout/src/timeline.rs:224`, timestamped resolution reuses the
existing resolver at `crates/rpptx-layout/src/context.rs:338`, and transition
composition has one renderer implementation at
`crates/rpptx-render/src/timeline.rs:49`.

Layering produced zero findings. No Cargo manifest changed. `rpptx-oxml` exposes
only model queries, evaluation remains in `rpptx-layout`, composition remains in
`rpptx-render`, and the facade assembles those layers without adding a reverse
dependency. The intended separation is documented at
`docs/hld/10-bindings-spec.md:748`.

Harness produced zero findings. The F-214 plan declares all 49 entries expected
unchanged at `.claude/plans/F-214-design.md:173`, the completion ledger reports
49 of 49 at `docs/sprints/AS_BUILT.md:10368`, and this review reproduced that
result at the exact reviewed HEAD.

Gate produced zero sprint-scope findings. The F-213 corpus preservation gate and
the F-214 nine-case PowerPoint differential have concrete executable evidence.
The broader M21 gate remains explicitly open rather than being inferred from
the two completed S60 stories.

Docs produced zero findings. The changed HLD files are exactly the six-file
F-214 impact list at `.claude/plans/F-214-design.md:145`, together with the
three-file F-213 subset at `.claude/plans/F-213-design.md:134`. They describe
the implemented layering, model, rendering, native surface, external oracle,
static isolation, and zoom evidence boundary without contradicting the
integrated code.

Dependencies produced zero findings. No manifest or lockfile changed, and no
new dependency, crate, feature flag, trait, generic, backend animation variant,
runtime oracle dependency, or committed binary fixture was introduced. This
matches the approved boundary at `.claude/plans/F-214-design.md:101`.

Surface produced zero findings. The two low-level OXML queries, layout timeline
types and evaluator, renderer module, and one native facade method are the exact
additive pre-1.0 surface documented at `docs/hld/10-bindings-spec.md:735` and
`docs/hld/10-bindings-spec.md:748`. Static methods remain on their independent
path at `crates/rpptx/src/lib.rs:725`, and no Python, WASM, or CLI surface was
added.

Delivery ledgers produced zero findings. Both stories are done and unowned in
`docs/sprints/CURRENT_SPRINT.md:31`, both are done in the backlog at
`docs/sprints/BACKLOG.md:412` and `docs/sprints/BACKLOG.md:413`, both have
completion records beginning at
`docs/sprints/AS_BUILT.md:10276` and `docs/sprints/AS_BUILT.md:10321`, and both
have matching tracker entries at `docs/sprints/SPRINT_TRACKER.md:353` and
`docs/sprints/SPRINT_TRACKER.md:354`. The sprint run state records F-214
completed from integration commit 322ccc2 at `.claude/scratch/S60-run.json:16`
and records the sprint in review phase at `.claude/scratch/S60-run.json:30`.
