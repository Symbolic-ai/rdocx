# S34 sprint review, pass 1

**Reviewed**: `sprint/s34` at `eb255b48d6d7b15a986a94b16267d6553f64dd09`
against merge base `b3df723a4b0bd655aeccc32435259b88bb7ec98e`, 65 files,
8,889 insertions and 106 deletions, crates: `oxml-py-support`, `rdocx-py`,
`rpptx`, and `rpptx-py`
**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have

## Blocking

None.

## Should-fix

None.

## Nice-to-have

None.

## Run-sprint disposition

- `fix-now`: none.
- `tracked-follow-up`: none.
- `human-action`: before M13 can close, obtain the first reviewed hosted
  cross-platform wheel execution evidence required by
  `docs/hld/14-development-backlog.md:1077`. This action is already part of the
  approved F-137 gate and is not a missing S34 implementation item.
- `refuted`: none.

## Sprint definition of done

The S34 definition of done holds at its approved evidence boundary.

- Both fresh cp39-abi3 wheels carry typed-package markers and pass exact strict
  mypy plus live stubtest. The durable evidence names seven rdocx and six rpptx
  strict sources plus eleven live modules at `docs/sprints/AS_BUILT.md:4905` and
  `docs/sprints/AS_BUILT.md:4922`.
- The exact tagged python-docx 1.2.0 suite contains seventeen documented
  examples, uses both writers and both readers, and compares normalized public
  structure at `docs/sprints/AS_BUILT.md:4938` and
  `docs/sprints/AS_BUILT.md:4956`.
- The unpublished rpptx package runs the seven python-pptx 1.0.2 Getting Started
  examples with bidirectional structural comparison and strict global stale
  handles at `docs/sprints/AS_BUILT.md:4971` and
  `docs/sprints/AS_BUILT.md:4992`.
- The release workflow contains the exact two-package, six-platform abi3 matrix,
  native install and test steps, two-source-distribution build, fourteen-product
  collection, and tag-only OIDC publication boundary at
  `.github/workflows/wheels.yml:17`, `.github/workflows/wheels.yml:58`,
  `.github/workflows/wheels.yml:118`, and `.github/workflows/wheels.yml:146`.
  Its local acceptance evidence includes both native wheels and source
  distributions, clean installs, parity, typing, stubtest, archive inventory,
  and 155 rejected workflow mutations at `docs/sprints/AS_BUILT.md:5019` and
  `docs/sprints/AS_BUILT.md:5029`.
- Pull requests run the exact two-package Python matrix in fresh environments,
  build each extension before its full package test directory, propagate test
  failure, and hold only read authority at `.github/workflows/ci.yml:12`,
  `.github/workflows/ci.yml:29`, and `.github/workflows/ci.yml:40`. The integrated
  evidence records all 33 rdocx tests, all 10 rpptx tests, and a real failing-test
  mutation at `docs/sprints/AS_BUILT.md:5062`.
- The authoritative full verification is recorded as passed with an unchanged
  harness at `.claude/scratch/S34-run.json:75`. The final commit changes only the
  four sprint ledger files. Independent review reran the complete 28-test
  workflow regression module, the six shared path tests, the focused rpptx facade
  regression, both python-pptx 1.0.2 oracle tests, prose validation, generated
  skill synchronization, and diff hygiene. All passed. The initial full rpptx
  test attempt encountered only sandbox denial of the default uv cache. The two
  affected oracle tests passed when rerun with a writable temporary cache.
- All five CURRENT rows are done and unowned at
  `docs/sprints/CURRENT_SPRINT.md:27`. BACKLOG reports M13 as ten done and eight
  pending at `docs/sprints/BACKLOG.md:31`, and its F-134 through F-138 rows agree
  at `docs/sprints/BACKLOG.md:268`. The five tracker rows and estimates agree at
  `docs/sprints/SPRINT_TRACKER.md:198`. Each story has exactly one durable
  AS_BUILT entry beginning at `docs/sprints/AS_BUILT.md:4899`.

## Milestone gate

The M13 end gate is: "wheels install and pass the parity suites on every target
platform" at `docs/hld/14-development-backlog.md:994`.

That end-of-milestone gate does not yet hold. The integrated workflow and its
local native executions prove the exact product graph, native installation, and
parity behavior, but no hosted run has supplied execution evidence for every
matrix platform. F-137 expressly assigns that evidence to the first reviewed
hosted dispatch at `docs/hld/14-development-backlog.md:1077`, and its AS_BUILT
record accurately says that no dispatch, tag, or publication occurred at
`docs/sprints/AS_BUILT.md:5019`. M13 also retains eight planned features,
F-139 through F-146, at `docs/sprints/BACKLOG.md:273`. The milestone therefore
remains open without invalidating the completed S34 feature gate.

## Not found

No integrated interaction, duplicated helper, forbidden dependency direction,
unexplained hash delta, sprint-gate failure, HLD contradiction, unowned new
dependency, speculative public surface, workflow overlap, packaging conflict,
release-authority leak, status mismatch, or generated artifact was found.

The common `oxml-py-support` layer remains format-neutral, PyO3 remains confined
to the two Python binding crates, and the WASM dependency graph remains free of
PyO3. The typed surfaces, parity suites, stale-handle contract, wheel workflow,
and pull-request job exercise the same package boundaries without introducing a
second implementation path. The latest independent feature reviews report zero
defects, smells, or nitpicks at `.claude/reviews/F-134-all-pass-2.md:6`,
`.claude/reviews/F-135-all-pass-2.md:6`,
`.claude/reviews/F-136-all-pass-4.md:4`,
`.claude/reviews/F-137-all-pass-10.md:6`, and
`.claude/reviews/F-138-all-pass-2.md:6`.
