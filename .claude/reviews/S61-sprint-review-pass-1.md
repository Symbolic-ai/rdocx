# S61 sprint review, pass 1

**Reviewed**: sprint/s61 against 7c7742616134ec34aebd6aef945f8682e70d19fd,
34 files, 6,877 lines, crates: oxml-media, oxml-opc, rpptx-layout,
rpptx-oxml, rpptx
**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have
**Disposition**: no findings require fix-now, tracked-follow-up,
human-action, or refuted classification

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

The M21 gate does not yet hold at this F-215 dependency-prefix boundary. Its
exact contract is at `docs/hld/14-development-backlog.md:1896`. F-216 and F-227
remain pending at `docs/sprints/CURRENT_SPRINT.md:39` and
`docs/sprints/CURRENT_SPRINT.md:40`, so poster and playback rendering plus
animated export are not present. Later M21 stories still own SmartArt, package
variants, notes export, handout export, and the combined representative-deck
oracle. This is expected at the prefix boundary and does not block F-216 from
starting.

The F-215 dependency gate does hold. The story gate at
`docs/hld/14-development-backlog.md:1920` requires media bytes, relationships,
playback settings, and unsupported metadata to survive save and reopen without
duplication. The named gate begins at
`crates/rpptx/tests/integration.rs:806`, binds the two pinned audio and video
decks to explicit media kinds, shape ids, part names, MIME values, relationship
types, and retained metadata at `crates/rpptx/tests/integration.rs:810`, and
requires the corpus rather than silently skipping when the required-corpus
environment is active at `crates/rpptx/tests/integration.rs:830`. It compares
exact media identity and playback values at
`crates/rpptx/tests/integration.rs:842`, exact extracted bytes at
`crates/rpptx/tests/integration.rs:875`, and saved relationship, part-count,
metadata, and content-type facts at `crates/rpptx/tests/integration.rs:899`.
The completion record reports that exact required-corpus run at
`docs/sprints/AS_BUILT.md:10408`.

Full integrated verification is recorded at the reviewed HEAD
ea394fbd28e6ce778350a593261634a90148f3d1 in
`.claude/scratch/S61-run.json:44`. The plan requires 49 of 49 harness entries
unchanged at `.claude/plans/F-215-design.md:220`, the completion record reports
that result at `docs/sprints/AS_BUILT.md:10420`, and the same exact-HEAD
verification record carries the unchanged result at
`.claude/scratch/S61-run.json:45`. This review inspected the recorded full
verification and executable gate evidence. It did not rerun the broad suite.

## Not found

Interaction produced zero findings. F-215 is the only implemented story in the
reviewed prefix. Its shape-id identity, picture-owned trim, timing trigger
context, poster relationship, and diagnostics are the inputs assigned to F-216
at `docs/sprints/AS_BUILT.md:10422`, while the strict dependency remains visible
at `docs/sprints/CURRENT_SPRINT.md:46`.

Duplication produced zero findings. Format-neutral MIME and container checks
have one implementation in `oxml-media` at
`crates/oxml-media/src/lib.rs:86`, picture projection and mutation remain in
`rpptx-oxml` at `crates/rpptx-oxml/src/picture.rs:117`, and package assembly
remains in the facade at `crates/rpptx/src/lib.rs:2205`.

Layering produced zero findings. No Cargo manifest or lockfile changed. The
dependency-free media helpers stay in `oxml-media`, package relationship
constants stay in `oxml-opc`, PresentationML projections stay in `rpptx-oxml`,
and the facade owns package mutation. This matches the declared boundary at
`docs/hld/03-architecture.md:88` and `docs/hld/03-architecture.md:142`.

Harness produced zero findings. The approved plan, F-215 completion record,
and exact-HEAD sprint state all agree on 49 of 49 unchanged. No harness
baseline file changed in the reviewed delta.

Gate produced zero prefix-scope findings. The executable round-trip test uses
independent producer expectations rather than a self-comparison, and the
source-built lifecycle gates named in the approved plan at
`.claude/plans/F-215-design.md:177` remain in the existing integration
binaries. The broader sprint and M21 gates remain explicitly open rather than
being inferred from F-215.

Docs produced zero findings. The changed HLD files are exactly the six-file
impact list at `.claude/plans/F-215-design.md:197`. They describe current
implemented ownership, package behavior, native API, validation, and test
evidence without claiming F-216 rendering or F-227 export behavior.

Dependencies produced zero findings. No dependency, crate, feature flag, or
lockfile entry was added. The existing facade is the named consumer of the
new leaf media checks and relationship constants.

Surface produced zero findings. The typed picture projection and concrete
facade values and operations match the approved surface at
`.claude/plans/F-215-design.md:41` and `.claude/plans/F-215-design.md:73`.
The additions remain native Rust APIs in the published pre-1.0 crates, with no
Python, WASM, CLI, decoder, trait, generic, module, or production dependency
surface, as recorded at `docs/hld/10-bindings-spec.md:779`.

Delivery ledgers produced zero findings. F-215 is done and unowned at
`docs/sprints/CURRENT_SPRINT.md:38`, done in the backlog at
`docs/sprints/BACKLOG.md:414`, recorded in AS_BUILT at
`docs/sprints/AS_BUILT.md:10375`, and recorded in the sprint tracker at
`docs/sprints/SPRINT_TRACKER.md:356`. The run state records the consumed
handoff and completed integration at `.claude/scratch/S61-run.json:3`.
