# S36 sprint review, pass 1

**Reviewed**: `sprint/s36` at `9f75528296125a82abf110bf7ab1c5ece00d08a1`
against merge base `3c2a019fffccbdd7c7e6465c3c004a74c75dc486`, 65 files,
5,133 insertions and 963 deletions, crates: `oxml-cli-support`, `rdocx`,
`rdocx-cli`, `rdocx-wasm`, `rpptx`, `rpptx-cli`, and `rpptx-wasm`
**Verdict**: 1 blocking, 0 should-fix, 0 nice-to-have

## Blocking

### B1, the v1 release record closes with two Rust packages still unpublished

`docs/sprints/AS_BUILT.md:5235`

The S36 completion record calls `oxml-cli-support` a published crate and calls
`rpptx-cli` a published binary at `docs/sprints/AS_BUILT.md:5268`. Neither
claim is true at the reviewed SHA. Both approved plans explicitly prepared
future publication metadata without publishing at
`.claude/plans/F-143-design.md:101` and
`.claude/plans/F-144-design.md:112`. A fresh crates.io API check on 2026-08-14
found only the reserved 0.0.0 placeholder for each name.

The discrepancy is not only wording. The immutable `rpptx-v0.1.2` release
published the former twelve-package family at
`docs/sprints/AS_BUILT.md:4432`. The expanded release contract now selects
fourteen packages at one exact version at `.claude/commands/release.md:31`,
while refusing an existing requested tag at `.claude/commands/release.md:70`.
It therefore cannot add these two 0.1.2 packages through the already-used tag.
A future release needs a fresh lockstep version and tag, but the backlog records
all 159 stories done with nothing pending at `docs/sprints/BACKLOG.md:31`, even
though S36 is declared the v1 release gate at
`docs/sprints/SPRINT_PLAN.md:562`.

Before closure, the delivery contract needs one explicit, truthful direction.
If crates.io availability is part of the v1 gate, add and execute a reviewed
fresh-version release F-ID through `/release`. If publication is intentionally
deferred, describe both crates as publishable and unpublished, qualify what the
v1 release gate means, and give the future Rust release a tracked backlog home.
This review cannot choose or perform either external publication policy.

**Run-sprint disposition**: `human-action`.

## Should-fix

None.

## Nice-to-have

None.

## Run-sprint disposition

- `fix-now`: none pending the B1 release-policy decision.
- `tracked-follow-up`: none. B1 must first establish whether deferred Rust
  publication is allowed beyond this declared release gate.
- `human-action`: B1. Choose fresh-version crates.io release or explicit
  deferral, then reconcile the delivery records and backlog.
- `refuted`: none.

## Sprint definition of done

All nine technical S36 definition-of-done items hold. B1 blocks the release
record rather than the implemented command and package gates.

- The shared range and JSON contracts pass seven focused tests, including the
  exact `2,4-6` and 100,000-value boundaries implemented at
  `crates/oxml-cli-support/src/lib.rs:33`.
- The presentation CLI has nine commands at
  `crates/rpptx-cli/src/main.rs:18`. Its fourteen fresh integration tests passed,
  including corrupted-deck rejection, all 50 pinned corpus decks, deterministic
  PDF and PNG output, bounded raster and diff work, exact thumbnail dimensions,
  recursive outline ordering, and field-only title identity.
- The document CLI's seven compiled-command integration tests passed. They bind
  schema-one inspect output, document-order text, default conversion paths,
  validation status, replacement persistence, and deterministic rendering at
  `crates/rdocx-cli/tests/integration.rs:85`.
- The local npm workflow remains installation-only. It builds both bundler
  packages, packs and installs each into a separate temporary consumer, checks
  exact inventory, and imports the installed module at
  `.github/workflows/ci.yml:125`. The workflow has only repository read
  permission at `.github/workflows/ci.yml:12` and no npm publication command.
- The root README runner compiled all six `rust,no_run` examples from the sole
  README source. The runner discovers the exact Cargo artifact and invokes
  warning-denied rustdoc at `scripts/readme_doctests.py:36`.
- `generate_all_samples` remains the sole harness generator. A fresh hash check
  matched all 28 entries, consistent with every S36 AS_BUILT harness field.
- Two exact `save_and_load_file` processes passed concurrently. The exercised
  path includes its process identity at
  `crates/rdocx/tests/integration_test.rs:145`.
- The complete 36-test workflow regression module, prose check, generated-skill
  drift check, and diff hygiene passed. The authoritative integrated full gate
  is recorded with an unchanged harness at
  `.claude/scratch/S36-run.json:114`. The later reviewed commit changes only
  sprint delivery ledgers.
- All eight CURRENT rows are done and unowned at
  `docs/sprints/CURRENT_SPRINT.md:32`. Each has exactly one tracker row, one
  AS_BUILT entry, a completed design checklist, and a final independent feature
  review reporting zero defects and zero smells.

## Milestone gate

The M13 end gate is: "wheels install and pass the parity suites on every target
platform" at `docs/hld/14-development-backlog.md:994`.

That narrow milestone gate holds. S35 review bound successful installed-wheel
parity for both packages across all six target families to hosted run
31722258395 at `.claude/reviews/S35-sprint-review-pass-3.md:80`. S36 did not
change the wheel workflow or either Python package surface. B1 concerns the
separate claim that the final v1 Rust CLI release boundary is complete.

## Not found

- **Interaction**: shared CLI helpers, both command binaries, facade text
  replacement, deterministic rendering, README compilation, npm assembly, and
  test isolation compose without a cross-story behavior conflict.
- **Duplication**: range, JSON, and output-path behavior have one shared owner.
  The obsolete sample generator is deleted, and the README remains its own
  single snippet source.
- **Layering**: `oxml-cli-support` depends only on `serde_json` and `thiserror`.
  Its only workspace consumers are the two CLI crates. No new forbidden
  `oxml-*` edge points to `rdocx-*` or `rpptx-*`.
- **Harness**: every plan declared unchanged output, every AS_BUILT entry agrees,
  and the fresh 28-entry check produced no delta.
- **Gate**: no technical S36 gate failure beyond the release-record issue in B1
  was found.
- **Docs**: the exact HLD impact union from the eight plans is present and
  consistently describes shared CLI ownership, facade replacement, deterministic
  rendering, README rustdoc, local npm installation, and publication authority.
- **Dependencies**: each new internal or external edge has a current consumer.
  Both WASM graphs remain isolated from PyO3 and host font discovery.
- **Surface**: the shared helpers, presentation replacement method, `ShapeRef`
  equality, and nine-command presentation CLI are called for by the approved
  plans and covered by focused regressions.
- **Delivery records**: apart from B1, CURRENT, BACKLOG, SPRINT_PLAN,
  SPRINT_TRACKER, AS_BUILT, design plans, feature reviews, handoff consumption,
  estimates, actuals, velocity, and the variance escalation record reconcile.
