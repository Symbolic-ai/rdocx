# F-X012, all aspects, pass 1

**Reviewed**: the complete five-file working tree delta at
`a75e2b906eb632d8543ebde9db6922bfda653d44`, 330 additions and 16 deletions,
including the untracked authorized installer
**Verdict**: 4 defects, 0 smells, 0 nitpicks

## Defects

### D1, the archive member bound is applied after unbounded allocation

`scripts/install_pinned_poppler.py:50`
`scripts/install_pinned_poppler.py:52`
`.claude/plans/F-X012-design.md:37`

`archive.getmembers()` reads every tar header and retains every `TarInfo` in a
list before the code compares its length with `MAX_ARCHIVE_MEMBERS`. A small
XZ input can expand to a very large tar stream containing zero-length members.
That input consumes memory proportional to the entire member table before the
2,048-member check runs, while the extracted-size sum remains zero. The stated
member limit therefore does not bound the resource it is meant to protect.
Iterate the archive, increment the count and total size, and reject as soon as
either bound is crossed.

### D2, a populated prefix bypasses the pinned source guarantee

`scripts/install_pinned_poppler.py:118`
`scripts/install_pinned_poppler.py:119`
`scripts/install_pinned_poppler.py:123`
`.claude/plans/F-X012-design.md:32`
`.claude/plans/F-X012-design.md:86`

Any non-empty caller-selected prefix takes the reuse branch. That branch checks
only that three files report the expected version text, then returns without
downloading or hashing the reviewed source. Three arbitrary executables that
print `26.01.0`, or a differently sourced build of that version, are therefore
accepted as the exact oracle. This contradicts the source-provenance boundary
that motivated replacing the package-manager installation. Build into an
empty owned prefix every time, or authenticate an installer-owned provenance
record before permitting reuse.

### D3, the installer regression checks declarations rather than enforcement

`scripts/test_sprint_workflow.py:112`
`scripts/test_sprint_workflow.py:125`
`scripts/test_sprint_workflow.py:319`
`scripts/test_sprint_workflow.py:345`
`.claude/plans/F-X012-design.md:65`
`.claude/plans/F-X012-design.md:101`

The installer helper only searches the source text for constants, function
names, one CMake option, and the expected-version assignment. It never calls
the download, extraction, verification, or build behavior. Focused mutations
changed the checksum comparison, member-count comparison, and runtime identity
comparison to `if False` while retaining those strings. All three mutated
installers still passed `assert_pinned_poppler_installer_contract()`. The
declared version, checksum, and bounds can consequently remain present while
none is enforced. Add behavioral tests with controlled responses and synthetic
archives so bypassing each gate fails the named unit test.

### D4, every installer step can be disabled without failing the consumer test

`scripts/test_sprint_workflow.py:127`
`scripts/test_sprint_workflow.py:139`
`scripts/test_sprint_workflow.py:347`
`scripts/test_sprint_workflow.py:362`
`.claude/plans/F-X012-design.md:66`
`.claude/plans/F-X012-design.md:101`

The consumer helper proves that the script command text occurs in each named
job before a use marker, but it does not inspect step conditions or failure
policy. Adding `if: false` to any of the four installer steps leaves the step,
command, count, and ordering unchanged, so the regression still passes while
the consuming job runs without Poppler. `continue-on-error: true` is likewise
accepted for the Test, MSRV, and Presentation fidelity installer steps. Extend
the contract and negative mutations to reject conditional or failure-tolerant
installer steps in every consumer.

## Smells

None.

## Nitpicks

None.

## Focused evidence

- The workflow invokes the shared installer in Test, both Python matrix rows,
  Presentation fidelity, and MSRV before their Poppler-dependent work
  (`.github/workflows/ci.yml:26`, `.github/workflows/ci.yml:58`,
  `.github/workflows/ci.yml:224`, `.github/workflows/ci.yml:368`). No workflow
  command installs Poppler from Homebrew or `poppler-utils`. LibreOffice remains
  a separate step (`.github/workflows/ci.yml:221`).
- The Binaryen archive checksum still precedes extraction and the exact
  official Linux identity assertion
  (`.github/workflows/ci.yml:121`, `.github/workflows/ci.yml:126`). The existing
  exact WASM job contract and its weakened-gate mutations cover that ordering
  and identity (`scripts/test_sprint_workflow.py:704`,
  `scripts/test_sprint_workflow.py:721`,
  `scripts/test_sprint_workflow.py:959`,
  `scripts/test_sprint_workflow.py:970`).
- All 41 workflow tests pass. The installer compiles and exposes its CLI help.
  The hash harness remains 28 of 28, prose and generated skills pass, and the
  working diff has no whitespace error. The progress record also reports the
  unchanged installer executing successfully on disposable Ubuntu 24.04 and
  macOS, with the three tool identities verified
  (`.claude/scratch/F-X012-progress.md:20`,
  `.claude/scratch/F-X012-progress.md:24`,
  `.claude/scratch/F-X012-progress.md:33`). These positive results do not cover
  D1 through D4.
- The implementation adds the one authorized installer and no crate, module,
  trait, generic, public API, package dependency, version, publication path, or
  rendering baseline. The exact planned HLD impact remains testing strategy,
  development backlog, and build/toolchain
  (`.claude/plans/F-X012-design.md:75`). Those HLD updates correctly remain for
  completion rather than being mixed into this in-progress implementation.
- Sprint state is consistent. F-X012 is the sole in-progress story under owner
  `codex`, and the backlog arithmetic reports 165 done, one in progress, and
  zero pending (`docs/sprints/CURRENT_SPRINT.md:23`,
  `docs/sprints/BACKLOG.md:32`, `docs/sprints/BACKLOG.md:33`,
  `docs/sprints/BACKLOG.md:299`).

## Not found

No additional workflow consumer, moving Poppler binary installation, wrong
Poppler literal or source checksum, wrong Binaryen identity, checksum-order
regression, product-code change, dependency-direction change, package or
release mutation, rendering-baseline delta, hash delta, unapproved file,
structural indirection, exception-swallowing path, arithmetic overflow, HLD
scope omission, tracker mismatch, prose violation, or nitpick was found beyond
D1 through D4.
