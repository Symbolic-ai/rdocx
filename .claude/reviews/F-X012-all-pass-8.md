# F-X012, all aspects, pass 8

**Reviewed**: the seven-file authorized LibreOffice amendment at
`0f3a9243f6ad7d5a60bdf71d196c0ff4fb02a378`, 442 additions and 9 deletions,
including the untracked authorized installer, then its interaction with the
complete seventeen-file F-X012 delta from
`a75e2b906eb632d8543ebde9db6922bfda653d44`, 2,055 additions and 40 deletions
**Verdict**: 3 defects, 0 smells, 0 nitpicks

## Defects

### D1, the reviewed Debian packages do not make soffice runnable on minimal Ubuntu

`scripts/install_pinned_libreoffice.py:149`
`scripts/install_pinned_libreoffice.py:155`
`scripts/install_pinned_libreoffice.py:161`
`.github/workflows/ci.yml:40`
`.github/workflows/ci.yml:393`
`.claude/plans/F-X012-design.md:61`
`.claude/plans/F-X012-design.md:68`
`docs/hld/15-build-and-toolchain.md:366`
`docs/hld/15-build-and-toolchain.md:373`

The installer gives `apt-get` only the reviewed local Debian packages and
disables recommended packages. A disposable minimal Ubuntu 24.04 run completed
that package installation, but the required `soffice --version` command exited
127. `ldd` identified absent runtime libraries across NSS and NSPR, D-Bus,
Cairo, GLib, X11, Xext, Xinerama, CUPS, Fontconfig, Freetype, and Kerberos.
The installer therefore fails at its own identity check before either Test or
MSRV can run the workspace suite. Provision the explicit reviewed system
runtime dependency set needed by the official build, while continuing to
install LibreOffice itself only from the checksum-pinned archive.

### D2, the regression does not prove the installer invokes final identity verification

`scripts/install_pinned_libreoffice.py:146`
`scripts/install_pinned_libreoffice.py:147`
`scripts/install_pinned_libreoffice.py:149`
`scripts/install_pinned_libreoffice.py:161`
`scripts/test_sprint_workflow.py:141`
`scripts/test_sprint_workflow.py:510`
`scripts/test_sprint_workflow.py:596`
`scripts/test_sprint_workflow.py:612`
`.claude/plans/F-X012-design.md:97`
`docs/hld/14-development-backlog.md:1308`

The runtime test calls download, extraction, and identity helpers separately,
then exercises `install()` only through the populated-prefix early failure. It
never reaches a successful mocked installation. A focused mutation removed the
`verify_soffice()` call from `install()`. The lexical installer contract still
passed, and the complete runtime-guard test also passed against the mutated
module. The suite consequently permits a future installer to download and
install packages without verifying the executable that the viewer tests will
use. Exercise the successful orchestration path with controlled helpers and
require download, extraction, package installation, identity verification,
and path exposure in order.

### D3, two declared archive guards can be removed without failing the tests

`scripts/install_pinned_libreoffice.py:86`
`scripts/install_pinned_libreoffice.py:87`
`scripts/install_pinned_libreoffice.py:101`
`scripts/install_pinned_libreoffice.py:107`
`scripts/test_sprint_workflow.py:141`
`scripts/test_sprint_workflow.py:553`
`scripts/test_sprint_workflow.py:566`
`scripts/test_sprint_workflow.py:573`
`docs/hld/12-testing-strategy.md:214`
`docs/hld/14-development-backlog.md:1308`

The archive test covers traversal, but it never supplies a symlink or another
unsupported non-file member. Its incomplete archive omits both required
packages, so rejection stops at the core package and never proves that Impress
is independently required. Focused mutations removed the non-file rejection
and removed the Impress entry from the required-package tuple one at a time.
Both the lexical contract and complete runtime-guard test stayed green for
both mutations. Add an unsupported-member case and construct required-package
cases that omit core and Impress independently. The HLD promise that behavioral
regressions execute every source and resource guard does not currently hold.

## Smells

None.

## Nitpicks

None.

## LibreOffice amendment evidence

- The official LibreOffice checksum endpoint reports the exact configured
  SHA-256, and the official archive response reports 218,075,885 bytes, below
  the 224 MiB download ceiling
  (`scripts/install_pinned_libreoffice.py:18`,
  `scripts/install_pinned_libreoffice.py:19`,
  `scripts/install_pinned_libreoffice.py:20`,
  `scripts/install_pinned_libreoffice.py:31`). The URL, version, archive name,
  and checksum are internally consistent.
- The extraction implementation is streaming and checks member count and
  declared expanded bytes before writing each member. It enforces one archive
  root, resolved-path containment, regular-file or directory types, the core
  and Impress packages, and an absent installation prefix
  (`scripts/install_pinned_libreoffice.py:63`,
  `scripts/install_pinned_libreoffice.py:66`,
  `scripts/install_pinned_libreoffice.py:71`,
  `scripts/install_pinned_libreoffice.py:74`,
  `scripts/install_pinned_libreoffice.py:78`,
  `scripts/install_pinned_libreoffice.py:86`,
  `scripts/install_pinned_libreoffice.py:101`,
  `scripts/install_pinned_libreoffice.py:135`). D2 and D3 concern the test
  contract, not additional defects in those current guard implementations.
- Test and MSRV each invoke the exact one-line installer step before the named
  full workspace suite
  (`.github/workflows/ci.yml:40`, `.github/workflows/ci.yml:44`,
  `.github/workflows/ci.yml:393`, `.github/workflows/ci.yml:397`). The consumer
  regression rejects missing, conditional, failure-tolerant, and successfully
  short-circuited steps in both jobs
  (`scripts/test_sprint_workflow.py:164`,
  `scripts/test_sprint_workflow.py:172`,
  `scripts/test_sprint_workflow.py:485`,
  `scripts/test_sprint_workflow.py:508`). D1 prevents the currently invoked
  installer from completing on its target runner.
- The two LibreOffice tests and four prior focused workflow tests pass, followed
  by all 46 workflow tests. All three Python files compile and the CI YAML
  parses. The hash harness remains 28 of 28. Prose, generated-skill
  synchronization, and diff hygiene pass. These green local results do not
  cover D1 through D3.
- The user-authorized new path and hosted trigger are recorded in the progress
  note (`.claude/scratch/F-X012-progress.md:113`,
  `.claude/scratch/F-X012-progress.md:121`,
  `.claude/scratch/F-X012-progress.md:123`). The amendment edits only the
  approved plan, CI workflow, existing workflow regression file, authorized
  installer, and HLD12, HLD14, and HLD15
  (`.claude/plans/F-X012-design.md:107`,
  `.claude/plans/F-X012-design.md:111`). HLD12 owns the exact viewer oracle,
  HLD14 owns the story gate, and HLD15 owns installation mechanism
  (`docs/hld/12-testing-strategy.md:203`,
  `docs/hld/12-testing-strategy.md:216`,
  `docs/hld/14-development-backlog.md:1303`,
  `docs/hld/14-development-backlog.md:1315`,
  `docs/hld/15-build-and-toolchain.md:366`,
  `docs/hld/15-build-and-toolchain.md:374`). The HLD files agree with the
  intended contract, though D1 means the implementation does not yet satisfy
  their clean-Ubuntu statement.

## Complete feature evidence

- Pass 7 remains clean for the pinned Poppler source and consumers, Binaryen
  identity, exact uv setup, runner-temporary cache, 8 MiB stack scope, YAML key
  normalization, and all earlier mutation remediations
  (`.claude/reviews/F-X012-all-pass-7.md:21`,
  `.claude/reviews/F-X012-all-pass-7.md:46`,
  `.claude/reviews/F-X012-all-pass-7.md:85`). The LibreOffice amendment does
  not alter those implementations.
- The complete F-X012 delta still contains no crate source, manifest, lockfile,
  public API, dependency, package version, publication workflow, rendering
  baseline, or hash-baseline change. The existing macOS Presentation fidelity
  setup remains separate as required (`.claude/plans/F-X012-design.md:69`,
  `docs/hld/15-build-and-toolchain.md:373`). F-X012 remains the sole in-progress
  S40 story, with 165 done, one in progress, and zero pending
  (`docs/sprints/CURRENT_SPRINT.md:23`, `docs/sprints/BACKLOG.md:32`,
  `docs/sprints/BACKLOG.md:33`, `docs/sprints/BACKLOG.md:299`).

## Not found

No wrong LibreOffice URL or checksum, archive larger than its declared download
ceiling, unbounded member-table allocation, current unsafe-path acceptance,
current unsupported-member acceptance, current missing core or Impress guard,
populated-prefix reuse, incorrect current identity literal, missing Test or
MSRV consumer, conditional or failure-tolerant current consumer step, product
or public API change, crate or dependency change, package or publication
change, rendering or hash delta, unlisted HLD edit, dependency-direction
regression, structural indirection, panic, prose violation, smell, or nitpick
was found beyond D1 through D3.
