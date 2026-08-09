# S28 sprint review, pass 1

**Reviewed**: `sprint/s28` at `90c203627788c292b24ba67213873225b4d6a2df` against merge base `1c94efa7cd4d635c8a20a90bd9d0b2bc4dffbf90`, 37 files, 6,499 changed lines, comprising 6,395 insertions and 104 deletions, crates: `oxml-drawing`, `rpptx-oxml`, and `rpptx`
**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have

## Blocking

None.

## Should-fix

None.

## Nice-to-have

None.

## Milestone gate

The M11 gate is: "a generated 10-slide deck opens clean in PowerPoint,
Keynote, Google Slides and LibreOffice" at
`docs/hld/14-development-backlog.md:839`.

The gate holds. `generated_ten_slide_write_api_deck_opens_clean_in_all_four_viewers`
at `crates/rpptx/tests/integration.rs:218` regenerates the frozen candidate,
checks its SHA-256, reruns the pinned PowerPoint and LibreOffice operations, and
validates all four clean evidence rows. The PowerPoint, Keynote, Google Slides,
and LibreOffice observations at `crates/rpptx/tests/integration.rs:88` all bind
to SHA-256
`d36da6e8849eabd4487d2572baea19c3716ee7d0fe03aaa4714a28ce3c41de4f`.
HLD 12 records the executable and human-action boundaries at
`docs/hld/12-testing-strategy.md:174` and the exact four results at
`docs/hld/12-testing-strategy.md:187`. The completion record names the pinned
viewer versions, Google acceptance date and Chrome build, exact ignored gate,
and integrated full verification at `docs/sprints/AS_BUILT.md:3797`.

The remaining sprint definition of done also has evidence. The table gate is
named at `crates/rpptx/tests/integration.rs:1174`, the duplicated-image scope
gate at `crates/rpptx/tests/integration.rs:784`, and the combined property gate
at `crates/rpptx/tests/integration.rs:373`. The completion record reports the
unchanged 28-entry hash harness at `docs/sprints/AS_BUILT.md:3805`. The affected
development crates retain version `0.0.0` and `publish = false`, with no manifest
or lockfile delta in the reviewed range.

## Not found

- Interaction: zero findings. The F-116 deck composes table mutation, slide
  duplication, removal and movement, hidden and background state, core
  properties, ordinary save, and slideshow save in one validated package. Its
  final order, relationship scopes, media deduplication, and property state are
  asserted after reopen at `crates/rpptx/tests/integration.rs:140`.
- Duplication: zero findings. Sprint helpers retain distinct responsibilities
  for table mutation, package-graph collection edits, property staging, and
  acceptance evidence. No equivalent helper was added under a second name.
- Layering: zero findings. No Cargo manifest changed. The shared
  `oxml-drawing` additions remain format-neutral, PresentationML XML behavior
  remains in `rpptx-oxml`, and package-owning facade behavior remains in
  `rpptx`.
- Harness: zero findings. Every F-113 through F-116 design declares an unchanged
  harness, every completion entry records all 28 hashes unchanged, and the
  reviewed range contains no baseline delta.
- Gate: zero findings. The named focused gates exist, the integrated full gate
  is recorded as passed, and the manual Keynote and Google Slides portions are
  recorded as performed rather than inferred from automated checks.
- Docs: zero findings. HLD 04 covers core-property ownership and collection
  media pruning, HLD 05 covers table construction and merge semantics, HLD 06
  covers the facade and package-graph behavior, and HLD 12 records the complete
  cross-viewer evidence. These updates match the implementation and sprint
  completion records.
- Deps: zero findings. No dependency or lockfile changed, so the sprint adds no
  dependency without a named consumer.
- Surface: zero findings. The table and cell handles, slide collection methods,
  slide and presentation property methods, and narrow cross-crate XML helpers
  are each called for by an approved F-113, F-114, or F-115 plan. F-116 adds no
  production API.
