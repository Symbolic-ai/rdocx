# S28 sprint review, pass 2

**Reviewed**: `sprint/s28` at `463f7037e565838fece475ba376443bcad6dd2f9` against merge base `1c94efa7cd4d635c8a20a90bd9d0b2bc4dffbf90`, 38 files, 6,574 changed lines, comprising 6,470 insertions and 104 deletions, crates: `oxml-drawing`, `rpptx-oxml`, and `rpptx`
**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have

## Blocking

None.

## Should-fix

None.

## Nice-to-have

None.

## Pass 1 follow-up

The completed-owner remediation is correct. Every F-113 through F-116 row is
`done` and uses the canonical `-` owner sentinel at
`docs/sprints/CURRENT_SPRINT.md:38`. The execution backlog independently marks
the same four stories done at `docs/sprints/BACKLOG.md:233`. The remediation
changes no product code, test evidence, or milestone behavior.

## Milestone gate

The M11 gate is: "a generated 10-slide deck opens clean in PowerPoint,
Keynote, Google Slides and LibreOffice" at
`docs/hld/14-development-backlog.md:839`.

The gate holds. `build_f116_ten_slide_deck` at
`crates/rpptx/tests/integration.rs:4187` composes all F-107 through F-115 write
features in one package, then performs image-bearing duplication, removal, and
final-index movement before returning exactly ten slides. Its structural check
at `crates/rpptx/tests/integration.rs:4408` requires clean validation, the final
slide size and core properties, unique slide ids, hidden and background state,
constructor kinds, a table, and three slide-scoped pictures.

The ignored gate at `crates/rpptx/tests/integration.rs:218` regenerates the
candidate, checks SHA-256
`d36da6e8849eabd4487d2572baea19c3716ee7d0fe03aaa4714a28ce3c41de4f`, reruns
the pinned PowerPoint and LibreOffice operations, and validates every clean
evidence row. The PowerPoint, Keynote, Google Slides, and LibreOffice records at
`crates/rpptx/tests/integration.rs:88` all bind to that SHA and record ten
observed slides with no repair or conversion error. HLD 12 maps every M11 story
to candidate coverage at `docs/hld/12-testing-strategy.md:160` and records the
four exact viewer results at `docs/hld/12-testing-strategy.md:187`.

The focused sprint gates also exist. Table merge and split reopen with the
original grid at `crates/rpptx/tests/integration.rs:1174`. Slide collection
duplication resolves images through the destination scope at
`crates/rpptx/tests/integration.rs:784`. Slide and presentation properties
round-trip together at `crates/rpptx/tests/integration.rs:373`. The completion
records report all 28 deterministic hashes unchanged at
`docs/sprints/AS_BUILT.md:3689`, `docs/sprints/AS_BUILT.md:3727`,
`docs/sprints/AS_BUILT.md:3765`, and `docs/sprints/AS_BUILT.md:3805`.

## Not found

- Interaction: zero findings. The acceptance deck applies the four sprint
  stories after the earlier M11 surface and asserts the resulting package as a
  whole at `crates/rpptx/tests/integration.rs:4187` and
  `crates/rpptx/tests/integration.rs:4408`.
- Duplication: zero findings. Table construction and mutation, package-graph
  collection edits, property persistence, and acceptance evidence remain
  separate concrete responsibilities. No equivalent sprint helper was added
  under a second name.
- Layering: zero findings. No Cargo manifest changed. Format-neutral table and
  text behavior remains under `oxml-drawing`, PresentationML XML behavior
  remains under `rpptx-oxml`, and package-owning facade behavior remains under
  `rpptx`. The existing documented `oxml-drawing` to `rdocx-oxml` theme adapter
  is the only cross-family edge.
- Harness: zero findings. The reviewed range contains no hash baseline change,
  and all four completion records declare the same unchanged 28-entry result.
- Gate: zero findings. The structural, package, focused story, and performed
  four-viewer evidence all name executable tests or recorded observations.
- Docs: zero findings. HLD 04 covers core-property ownership and candidate-only
  media pruning, HLD 05 covers table construction and merge semantics, HLD 06
  covers slide collection and property facade behavior, and HLD 12 covers the
  SHA-bound acceptance deck. These descriptions match the reviewed code.
- Deps: zero findings. No manifest or lockfile changed, so the sprint adds no
  dependency without a named consumer.
- Surface: zero findings. The table and cell handles are required by the
  approved F-113 contract at `.claude/plans/F-113-design.md:54`, the three slide
  collection methods by `.claude/plans/F-114-design.md:34`, and the property
  methods by `.claude/plans/F-115-design.md:28`. F-116 adds no production API,
  as required at `.claude/plans/F-116-design.md:64`.
