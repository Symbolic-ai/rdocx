# F-X012, all aspects, pass 3

**Reviewed**: the complete five-file working implementation delta at
`a75e2b906eb632d8543ebde9db6922bfda653d44`, 458 additions and 16 deletions,
plus passes 1 and 2 and all remediations
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Prior finding dispositions

- Pass 1 D1 remains fixed. The XZ archive is streamed one member at a time,
  the member count is checked before extraction, and the regression forbids
  `getmembers()` while requiring rejection of member 2,049
  (`scripts/install_pinned_poppler.py:52`,
  `scripts/install_pinned_poppler.py:56`,
  `scripts/test_sprint_workflow.py:382`,
  `scripts/test_sprint_workflow.py:397`).
- Pass 1 D2 remains fixed. An existing file or non-empty directory at the
  selected prefix fails before download, and a populated exact-version prefix
  is behaviorally rejected (`scripts/install_pinned_poppler.py:118`,
  `scripts/install_pinned_poppler.py:123`,
  `scripts/test_sprint_workflow.py:430`,
  `scripts/test_sprint_workflow.py:448`).
- Pass 1 D3 and pass 2 D1 are fixed. The tests now execute a bad checksum, an
  over-limit download, an over-limit extracted member, the streaming member
  ceiling, and a wrong identity for each of `pdftoppm`, `pdfinfo`, and
  `pdftotext` independently (`scripts/test_sprint_workflow.py:360`,
  `scripts/test_sprint_workflow.py:380`,
  `scripts/test_sprint_workflow.py:393`,
  `scripts/test_sprint_workflow.py:406`,
  `scripts/test_sprint_workflow.py:412`,
  `scripts/test_sprint_workflow.py:428`). The progress record reports that
  disabling either byte ceiling or narrowing verification to the first tool
  made the unchanged regression fail
  (`.claude/scratch/F-X012-progress.md:70`,
  `.claude/scratch/F-X012-progress.md:73`).
- Pass 1 D4 and pass 2 D2 are fixed. Every consumer requires exactly the
  ordinary Bash run step, rejects any `continue-on-error`, applies the shared
  successful-short-circuit check, and precedes its use marker
  (`scripts/test_sprint_workflow.py:129`,
  `scripts/test_sprint_workflow.py:146`,
  `scripts/test_sprint_workflow.py:151`). All four jobs have negative mutations
  for `if: false`, `continue-on-error: true`, and an in-body `exit 0`
  (`scripts/test_sprint_workflow.py:450`,
  `scripts/test_sprint_workflow.py:477`,
  `scripts/test_sprint_workflow.py:487`).

## Focused evidence

- The five focused Poppler and WASM tests pass, followed by all 43 workflow
  tests. Both Python files compile. The hash harness remains 28 of 28. Prose,
  generated-skill synchronization, and diff hygiene pass.
- The installer pins the official version, source URL, and SHA-256, bounds the
  compressed download, streams and bounds extraction, rejects traversal and
  unsupported archive entries, isolates the build, caps parallel work at four,
  copies only the three requested utilities, verifies each exact runtime
  identity, and exposes the verified directory only after success
  (`scripts/install_pinned_poppler.py:18`,
  `scripts/install_pinned_poppler.py:24`,
  `scripts/install_pinned_poppler.py:39`,
  `scripts/install_pinned_poppler.py:52`,
  `scripts/install_pinned_poppler.py:66`,
  `scripts/install_pinned_poppler.py:70`,
  `scripts/install_pinned_poppler.py:125`,
  `scripts/install_pinned_poppler.py:161`,
  `scripts/install_pinned_poppler.py:175`,
  `scripts/install_pinned_poppler.py:179`).
- Test, both Python matrix rows, Presentation fidelity, and MSRV invoke the one
  installer before their Poppler-dependent work
  (`.github/workflows/ci.yml:26`, `.github/workflows/ci.yml:58`,
  `.github/workflows/ci.yml:224`, `.github/workflows/ci.yml:368`). Package
  managers install build dependencies only. No workflow command installs a
  moving Poppler binary, and LibreOffice remains separate
  (`.github/workflows/ci.yml:221`).
- Binaryen verifies the reviewed archive checksum before testing the exact
  official Linux identity (`.github/workflows/ci.yml:122`,
  `.github/workflows/ci.yml:126`). Its exact contract and negative checksum and
  identity mutations remain intact
  (`scripts/test_sprint_workflow.py:828`,
  `scripts/test_sprint_workflow.py:845`,
  `scripts/test_sprint_workflow.py:1088`,
  `scripts/test_sprint_workflow.py:1095`).
- The positive cross-platform evidence remains current. The progress record
  reports the finished installer executing on disposable Ubuntu 24.04 and
  macOS, all three tools executing at 26.01.0, the former native and binding
  failures passing, and the required 421-slide corpus gate completing without
  missing output (`.claude/scratch/F-X012-progress.md:20`,
  `.claude/scratch/F-X012-progress.md:23`,
  `.claude/scratch/F-X012-progress.md:35`,
  `.claude/scratch/F-X012-progress.md:37`).
- Product code, public APIs, dependencies, crate metadata, package versions,
  publication authority, and rendering baselines remain unchanged. The exact
  planned HLD impact remains testing strategy, development backlog, and build
  and toolchain (`.claude/plans/F-X012-design.md:75`). Those current-state HLD
  updates correctly remain part of completion rather than this in-progress
  implementation.
- F-X012 is the sole in-progress story under owner `codex`. Backlog arithmetic
  remains 165 done, one in progress, and zero pending
  (`docs/sprints/CURRENT_SPRINT.md:23`, `docs/sprints/BACKLOG.md:32`,
  `docs/sprints/BACKLOG.md:33`, `docs/sprints/BACKLOG.md:299`).

## Not found

No wrong checksum or tool identity, bypassed download or extraction ceiling,
unbounded archive member allocation, unsafe archive path, accepted unsupported
member, populated-prefix provenance bypass, missing consumer, conditional or
failure-tolerant installer step, successful in-body short circuit, moving
Poppler installation, workflow-order error, Binaryen checksum-order or identity
error, product-code change, public API change, dependency change, package or
release mutation, rendering-baseline delta, hash delta, unapproved file, HLD
scope omission, sprint-state mismatch, structural indirection, panic, prose
violation, smell, or nitpick was found.
