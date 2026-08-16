# S42 sprint review, pass 1

**Reviewed**: `sprint/s42` against `1db18fd`, 60 files, 1495 insertions and 309
deletions across 9 commits. Crates: all eight `oxml-*`, `rdocx-oxml`,
`rdocx-py`, `rdocx-wasm`, and all seven `rpptx-*`.
**Verdict**: 0 blocking, 1 should-fix, 2 nice-to-have

The sprint changed almost no product logic. One trait impl moved between crates
and everything else is a version, a manifest, a document or a test expectation.
The interesting risk was therefore not correctness but completeness: whether
every carrier of two versions moved together, and whether the architectural fix
actually removed what it claimed to.

## Blocking

None.

## Should-fix

### S1, `/verify` does not run the release regressions it is trusted to cover
`.claude/commands/verify.md` and `scripts/test_sprint_workflow.py`

F-X022 moved every version carrier under `crates/` and passed `cargo fmt`,
`cargo clippy`, all 53 test binaries, the hash harness, README doctests and
`cargo deny`. It was still incomplete: the release-family preflight in
`scripts/test_sprint_workflow.py`, which `.github/workflows/publish.yml` invokes
by name as the publication gate, still asserted 0.2.0, as did the `ci.yml`
`verify_package` literal.

Neither `cargo test` nor any `/verify` step runs the Python suite. The gap
passed every local gate and would have failed in CI at publication time. It was
caught only because F-X023 read the same file and noticed the stale expectation.

`/verify --full` should run `python3 -m unittest scripts.test_sprint_workflow`,
or at minimum the two release-family tests that `publish.yml` names. Without it,
"the gate passed" and "the release will work" are different claims, and this
sprint proved they can diverge silently.

Not blocking, because both trains are now consistent and all 46 regressions
pass. Should be fixed before the next version bump, and it wants its own F-ID
rather than a quiet edit to a command file during a release sprint.

## Nice-to-have

### N1, the incubating train has no shared version key
`Cargo.toml` and 15 crate manifests

The eleven stable packages inherit `version.workspace = true`, so F-X023 moved
them with one line. The fifteen incubating packages each carry a literal, so
F-X022 edited fifteen manifests plus fourteen pins plus eleven READMEs plus
seven Rust sources for the same outcome. Raised in both stories' reviews.

### N2, the F-X024 invariant test cannot exercise its own crate
`crates/oxml-drawing/src/lib.rs`

Reintroducing the `oxml-drawing -> rdocx-oxml` edge now produces a genuine cargo
cycle that fails to resolve before any test runs. That is a stronger guarantee
than the test, but a reader proving the test works should edit a different
`oxml-*` crate or they will get a resolver error rather than a clean assertion
failure. Carried from F-X024's review.

## Milestone gate

S42 is an X cross-cutting sprint and closes no milestone, so
`docs/hld/14-development-backlog.md` has no end-of-milestone gate for it.

The sprint's own definition of done, clause by clause with evidence:

1. **"Every semver-compatible update outstanding at the sprint's start is taken."**
   Holds. All 16 taken in F-X020, `Cargo.lock` the only product change.

2. **"`cargo audit` reports zero vulnerabilities and `cargo deny check` passes,
   with `ttf-parser` still the single documented exception."** Holds. Zero
   vulnerabilities across 152 dependencies, all four `cargo deny` sections ok,
   RUSTSEC-2026-0192 still the one allowlisted entry with its documented route
   out.

3. **"The hash harness is either unchanged, or its delta names the dependency."**
   Holds, and with a caveat the sprint recorded rather than buried: the harness
   is unchanged at 28 of 28, but F-X020 changed all seven sample PDFs, traced to
   `font-types 0.12.3`, characterised with the pinned Poppler oracle as
   identical extracted text in 7 of 7 samples. The harness has no PDF coverage,
   which is filed as F-X021.

4. **"The pinned toolchain and MSRV still build the workspace."** Holds.
   `rust-toolchain.toml` still pins 1.97.1 and the workspace builds, including
   the WASM targets and the bundled-fonts-off path.

5. **"No `oxml-*` package depends on any `rdocx-*` or `rpptx-*` package, and the
   spec documents no exception."** Holds. Checked mechanically across all eight
   `oxml-*` manifests, zero format dependencies. `cargo tree -i rdocx-oxml`
   lists only `rdocx-*` consumers. `docs/hld/03-architecture.md` and `CLAUDE.md`
   both state the rule without an exception, and
   `no_shared_crate_depends_on_a_format_crate` enforces it.

6. **"Fifteen incubating packages read 0.3.0 and eleven workspace-version
   packages read 0.7.0, with every pin, lock entry, README, Python version and
   WASM literal agreeing."** Holds. 15 manifests at 0.3.0, workspace at 0.7.0,
   and no `0.2.0` or `0.6.0` remains anywhere under `crates`, `scripts`,
   `.github` or the root manifest. All 46 release regressions pass.

7. **"The exact publication sets hold: fourteen incubating and seven stable."**
   Holds. The patched workspace dry run stages exactly 21 packages with no
   error and every archive under 10 MiB. `rpptx-wasm` remains `publish = false`,
   and the four unpublished Python and WASM packages inherit their version
   without publication authority.

8. **"Nothing is tagged or published without the separate approval `/release`
   requires."** Holds. No tag exists and nothing has been published. Both
   release F-IDs sit at `reviewed` in the run state and `in-progress` in both
   trackers, with their plans `approved` and their test gates recorded as
   deferred, which is the release-preparation exception rather than an
   incomplete story.

## Not found

Aspects checked that produced nothing:

- **interaction**. The four stories are sequential and each verified after the
  previous. F-X024 is what made F-X022 possible, and F-X022's pin moves are what
  F-X023 depends on.
- **duplication**. No helper written twice.
- **layering**. The sprint's central subject, and the invariant is now stronger
  than when the sprint began: an exception became a test.
- **deps**. `cargo update` moved 16 semver-compatible versions and no manifest
  gained or lost a dependency, except the deliberate `oxml-drawing` and
  `rdocx-oxml` swap in F-X024.
- **harness**. 28 of 28 throughout, consistent with every AS_BUILT entry and
  commit message, with the PDF caveat stated in F-X020's entry.
- **surface**. One trait impl moved, one constant became public with a named
  consumer, one unused method was deleted. All in crates taking a breaking minor
  bump this sprint.
- **docs**. `03-architecture.md` and `CLAUDE.md` were both updated by F-X024 to
  drop the exception. No spec section is contradicted and left standing.
