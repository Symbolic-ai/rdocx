# F-X012, all aspects, pass 9

**Reviewed**: the seven-file authorized LibreOffice amendment at
`0f3a9243f6ad7d5a60bdf71d196c0ff4fb02a378`, 565 additions and 12 deletions,
including the untracked authorized installer, then its interaction with the
complete seventeen-file F-X012 delta from
`a75e2b906eb632d8543ebde9db6922bfda653d44`, 2,178 additions and 43 deletions
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Pass 8 finding dispositions

- D1 is fixed. Test and MSRV are bound to Ubuntu 24.04, and the installer
  declares all thirteen runtime packages established by the minimal-container
  failure analysis (`.github/workflows/ci.yml:21`,
  `.github/workflows/ci.yml:372`,
  `scripts/install_pinned_libreoffice.py:34`,
  `scripts/install_pinned_libreoffice.py:48`). It passes those system packages
  and every checksum-bound local Debian package to one checked `apt-get`
  invocation before testing the installed executable
  (`scripts/install_pinned_libreoffice.py:163`,
  `scripts/install_pinned_libreoffice.py:177`). The progress record accounts
  for the original exit-127 failure and records that the revised installer ran
  unchanged in fresh minimal Ubuntu 24.04 and returned the exact build identity
  (`.claude/scratch/F-X012-progress.md:140`,
  `.claude/scratch/F-X012-progress.md:148`,
  `.claude/scratch/F-X012-progress.md:150`).
- D2 is fixed. The success-path regression controls the archive download,
  streaming extraction, checked package installation, runtime verification,
  and GitHub PATH exposure, then requires that exact order
  (`scripts/test_sprint_workflow.py:680`,
  `scripts/test_sprint_workflow.py:705`,
  `scripts/test_sprint_workflow.py:717`,
  `scripts/test_sprint_workflow.py:720`). Independent focused mutations that
  removed either the verification call or PATH exposure were rejected.
- D3 is fixed. Extraction rejects every member that is neither a directory nor
  a regular file and independently requires the core and Impress Debian
  packages (`scripts/install_pinned_libreoffice.py:98`,
  `scripts/install_pinned_libreoffice.py:102`,
  `scripts/install_pinned_libreoffice.py:116`,
  `scripts/install_pinned_libreoffice.py:122`). The regression now supplies a
  symlink and separate archives missing each required package
  (`scripts/test_sprint_workflow.py:585`,
  `scripts/test_sprint_workflow.py:607`,
  `scripts/test_sprint_workflow.py:624`). Independent mutations removing the
  unsupported-member guard, core requirement, or Impress requirement all
  failed.

## Focused evidence

- The installer retains the reviewed URL, SHA-256, exact runtime identity,
  224 MiB download ceiling, 256-member streaming bound, 256 MiB expanded-byte
  ceiling, safe-root check, and absent-prefix provenance rule
  (`scripts/install_pinned_libreoffice.py:18`,
  `scripts/install_pinned_libreoffice.py:33`,
  `scripts/install_pinned_libreoffice.py:51`,
  `scripts/install_pinned_libreoffice.py:78`,
  `scripts/install_pinned_libreoffice.py:89`,
  `scripts/install_pinned_libreoffice.py:150`). The Test and MSRV consumers
  invoke only that installer before the full workspace suite
  (`.github/workflows/ci.yml:40`, `.github/workflows/ci.yml:44`,
  `.github/workflows/ci.yml:393`, `.github/workflows/ci.yml:397`).
- The regression pins the exact thirteen-package Ubuntu set and checks that the
  success-path apt command contains every runtime package and both local core
  packages (`scripts/test_sprint_workflow.py:155`,
  `scripts/test_sprint_workflow.py:172`,
  `scripts/test_sprint_workflow.py:683`,
  `scripts/test_sprint_workflow.py:690`). A focused mutation removing one
  runtime package was rejected by the central installer contract. Consumer
  mutations continue to reject a missing, conditional, failure-tolerant, or
  successfully short-circuited step in each job
  (`scripts/test_sprint_workflow.py:182`,
  `scripts/test_sprint_workflow.py:195`,
  `scripts/test_sprint_workflow.py:504`,
  `scripts/test_sprint_workflow.py:527`).
- The six focused workflow tests pass, followed by all 46 workflow tests. All
  three installer and workflow Python files compile, and the CI YAML parses.
  Cargo formatting, prose, generated-skill synchronization, and diff hygiene
  pass. The hash harness remains 28 of 28. The recorded hosted run remains a
  required completion action after this amendment is committed, rather than a
  defect in the reviewed implementation
  (`.claude/plans/F-X012-design.md:101`,
  `.claude/plans/F-X012-design.md:105`,
  `.claude/scratch/F-X012-progress.md:163`,
  `.claude/scratch/F-X012-progress.md:164`).

## Contract and scope evidence

- The plan requires the exact Ubuntu 24.04 LibreOffice archive, checksum,
  bounded extraction, explicit runtime libraries, runtime identity, and both
  unconditional consumers. The implementation satisfies that boundary
  (`.claude/plans/F-X012-design.md:61`,
  `.claude/plans/F-X012-design.md:71`,
  `.claude/plans/F-X012-design.md:98`,
  `.claude/plans/F-X012-design.md:101`).
- The plan lists exactly HLD12, HLD14, and HLD15
  (`.claude/plans/F-X012-design.md:108`,
  `.claude/plans/F-X012-design.md:112`). HLD12 owns the exact rendering oracle,
  HLD14 owns its behavioral and hosted gate, and HLD15 owns the Ubuntu install
  mechanism. All three now agree on Ubuntu 24.04, the reviewed archive and
  identity, explicit runtime libraries, bounded extraction, and unchanged
  baselines (`docs/hld/12-testing-strategy.md:211`,
  `docs/hld/12-testing-strategy.md:218`,
  `docs/hld/14-development-backlog.md:1303`,
  `docs/hld/14-development-backlog.md:1317`,
  `docs/hld/15-build-and-toolchain.md:366`,
  `docs/hld/15-build-and-toolchain.md:376`).
- Pass 7 remains clean for the exact Poppler source and consumers, Binaryen
  identity, pinned uv action, runner-temporary cache, scoped stack budget, and
  YAML mutation handling (`.claude/reviews/F-X012-all-pass-7.md:21`,
  `.claude/reviews/F-X012-all-pass-7.md:46`,
  `.claude/reviews/F-X012-all-pass-7.md:85`). The LibreOffice amendment does
  not modify those installer implementations or expand product runtime.
- The complete F-X012 delta still contains no crate source, manifest, lockfile,
  public API, dependency, package version, publication workflow, rendering
  baseline, or hash-baseline change. F-X012 remains the sole in-progress S40
  story, and workflow state remains in implementation
  (`docs/sprints/CURRENT_SPRINT.md:23`, `docs/sprints/BACKLOG.md:32`,
  `docs/sprints/BACKLOG.md:33`, `docs/sprints/BACKLOG.md:299`).

## Not found

No missing Ubuntu runtime prerequisite, minimal-container identity failure in
the revised evidence, wrong runner image, missing package-install check,
successful orchestration gap, missing final identity verification, missing PATH
exposure, unsupported archive member acceptance, missing core or Impress guard,
unbounded archive operation, populated-prefix reuse, unsafe path acceptance,
wrong LibreOffice source or identity, conditional or failure-tolerant consumer,
successful consumer short circuit, Poppler or Binaryen regression, product or
public API change, crate or dependency change, package or publication change,
rendering or hash delta, unlisted HLD edit, HLD contradiction, sprint-state
mismatch, structural indirection, panic, prose violation, smell, or nitpick was
found.
