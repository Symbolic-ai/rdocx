# S51 sprint review, pass 1

**Reviewed**: `sprint/s51` at `a4320e111d6b82fac66221e667104d2c8cccab35`
against merge base `cd3b34109e8d45da7d06a11d11964971c8d1568d`,
135 files and 17,814 changed lines. Crates: `oxml-chart`,
`oxml-cli-support`, `oxml-core`, `oxml-drawing`, `oxml-layout`,
`oxml-media`, `oxml-opc`, `oxml-pdf`, `oxml-sml`, `rdocx-layout`,
`rdocx-oxml`, `rdocx-wasm`, `rdocx`, `rpptx-chart`, `rpptx-cli`,
`rpptx-layout`, `rpptx-oxml`, `rpptx-render`, `rpptx-wasm`, and `rpptx`

**Verdict**: 1 blocking, 0 should-fix, 0 nice-to-have

## Blocking

### B1, watermark mutations discard the reusable normal layout engine

`crates/rdocx/src/document.rs:477`

`clone_for_staging` correctly creates an isolated candidate with
`normal_layout_engine` set to `None`. Both watermark setters then commit that
candidate by replacing the whole live `Document` at
`crates/rdocx/src/document.rs:1617` and
`crates/rdocx/src/document.rs:1652`. A successful watermark edit therefore
throws away the live F-X038 paragraph and shaping caches, even though no safe
body paragraph changed.

This is an F-168 and F-X038 interaction defect. It violates the sprint
definition that warm relayout rebuild only changed safe paragraphs at
`docs/sprints/CURRENT_SPRINT.md:80`, the approved persistent-engine ownership
at `.claude/plans/F-X038-design.md:57`, and the current HLD requirement that
every public mutation preserve the normal engine at
`docs/hld/08-rendering-spec.md:553`. It also makes the AS_BUILT statement that
warm layouts rebuild only changed safe paragraphs untrue for the watermark
surface at `docs/sprints/AS_BUILT.md:7872`.

The fix must keep the watermark operation atomic while committing its staged
package and typed state without discarding the live reusable engine. A focused
regression must populate the normal engine, perform each watermark setter, and
prove that unchanged safe body paragraphs and shaping work remain reusable
while completed normal and deterministic results are invalidated.

## Should-fix

None.

## Nice-to-have

None.

## Milestone gate

The M16 gate is: "a template with loops, conditionals and a repeating table row
produces a correct document from a JSON data model, and every field in it
evaluates to the value Word computes" at
`docs/hld/14-development-backlog.md:1299`.

The milestone gate itself holds. The parser corpus at
`crates/rdocx-oxml/src/text.rs:5139`, the pinned Word field matrix at
`crates/rdocx/tests/regression_test.rs:140`, the nested loop and conditional
fixture at `crates/rdocx/tests/regression_test.rs:3323`, and the three-row by
ten-record fixture at `crates/rdocx/tests/regression_test.rs:3389` all pass at
the reviewed SHA. The complete `rdocx` regression binary passes 120 tests,
including the S51 mail merge and comparison gates.

S51 is not ready for release or close despite that milestone evidence because
B1 leaves the sprint warm-relayout definition incomplete. The release boundary
is otherwise in the expected prepared state. F-X035 is reviewed and F-X036 is
still pending at `.claude/scratch/S51-run.json:78` and
`.claude/scratch/S51-run.json:91`. The release command requires a clean sprint
review at the exact SHA before any external mutation at
`.claude/commands/release.md:57`.

## Evidence

- The sprint state records a full verification with an unchanged harness at
  the exact reviewed SHA at `.claude/scratch/S51-run.json:139`. Independent
  review checks also passed `cargo test -p rdocx --test regression_test`, the
  focused parser, warm-cache, diagnostic-replay, caller-font provenance, and
  ordered-body tests.
- `python3 scripts/hash_harness.py --check` reports all 49 entries unchanged.
  Neither hash nor golden baseline files differ from the merge base. The eight
  completed implementation records each declare the unchanged result, including
  F-X038 at `docs/sprints/AS_BUILT.md:7900` and PR 36 at
  `docs/sprints/AS_BUILT.md:7939`.
- `cargo metadata --no-deps` reports 27 workspace packages and no forbidden
  `oxml-*` dependency on an `rdocx-*` or `rpptx-*` crate. The manifest and lock
  delta adds no external dependency. It changes only the approved incubating
  version carriers and internal pins shown at `Cargo.toml:55`.
- The prepared `rpptx-v0.4.0` notes pass deterministic check and render. They
  cover the intended incubating scope and contributor evidence at
  `CHANGELOG.md:103` and `CHANGELOG.md:142`. The workflow validates those notes
  before publication at `.github/workflows/publish.yml:26` and creates the
  GitHub release only from a freshly byte-compared render at
  `.github/workflows/publish.yml:113`. Metadata, exact allowlist routing,
  publication failure propagation, and rendered-notes workflow regressions
  pass. The requested tag is absent locally and on `origin`.
- PR 36 remains a GitHub merge whose second parent is Pedro Assumpcao's original
  commit `79390535acba0a116b25ac986b863bdb941c8f15`. GitHub reports PR 36 merged
  into `sprint/s51` through merge commit
  `92951e71474383b48ce7ede194be4d0f34729488`, and current-base CI run
  `32516942671` succeeded for the contributor head. The tracked delivery record
  preserves that chain and names the public integration tests at
  `docs/sprints/AS_BUILT.md:7913` and `docs/sprints/AS_BUILT.md:7933`.
- `python3 scripts/prose_check.py`,
  `python3 scripts/sync_agent_skills.py --check`, metadata layering inspection,
  and `git diff --check` pass. The reviewed source tree was clean at the pinned
  SHA before this review artifact was written.

## Not found

- `duplication`: one staging-clone boundary serves template, merge, comparison,
  and watermark atomicity. Comparison and mail merge have distinct alignment
  and identity-allocation responsibilities. No duplicate sprint subsystem was
  found.
- `layering`: no forbidden crate edge was added.
- `harness`: no baseline changed, and every completed feature's declaration
  agrees with the independent 49-entry check.
- `gate`: the exact M16 end gate has executable passing evidence above. B1 is a
  separate S51 definition-of-done failure.
- `docs`: apart from the implementation contradiction reported in B1, the union
  of feature plans and the updated HLD sections agree on package ownership,
  public surfaces, preservation, release preparation, and test boundaries.
- `deps`: no external dependency was added.
- `surface`: every added public type, field, and function belongs to F-166,
  F-167, F-168, F-X032, F-X033, F-X037, or the cross-crate implementation needs
  of F-X038. Python, WASM, and CLI facade exposure stays within the approved
  compatibility boundary.
- `structure`: `comparison.rs` is the only added Rust source module and was
  explicitly approved by F-167. The release-notes command and generated agent
  adapter are the ceremony required by F-X034. No unowned trait, generic,
  feature flag, crate, or forwarding wrapper was added.
