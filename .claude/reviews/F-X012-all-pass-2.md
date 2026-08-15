# F-X012, all aspects, pass 2

**Reviewed**: the complete five-file working implementation delta at
`a75e2b906eb632d8543ebde9db6922bfda653d44`, 419 additions and 16 deletions,
plus pass 1 and its remediation record
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, the behavioral guard test still leaves two bounds and two identities unproved

`scripts/test_sprint_workflow.py:359`
`scripts/test_sprint_workflow.py:369`
`scripts/test_sprint_workflow.py:371`
`scripts/test_sprint_workflow.py:399`
`scripts/install_pinned_poppler.py:39`
`scripts/install_pinned_poppler.py:60`
`scripts/install_pinned_poppler.py:90`
`.claude/plans/F-X012-design.md:36`
`.claude/plans/F-X012-design.md:65`

The new behavioral test proves a bad digest, the member-count limit, and a
wrong first tool identity. It never exceeds `MAX_DOWNLOAD_BYTES`, never exceeds
`MAX_EXTRACTED_BYTES`, and makes all three fake tools wrong so verification
stops at `pdftoppm`. Disabling either byte-limit comparison, or changing
`verify_tools()` to iterate only the first tool, therefore leaves the behavioral
test green. The lexical assertions retain the constants and all three tool
names, so they do not close those gaps. Exercise both byte ceilings and make
`pdfinfo` and `pdftotext` fail one at a time while the preceding tools report
the exact version.

### D2, three consumer jobs can still short-circuit the installer successfully

`scripts/test_sprint_workflow.py:129`
`scripts/test_sprint_workflow.py:145`
`scripts/test_sprint_workflow.py:421`
`scripts/test_sprint_workflow.py:448`
`.claude/plans/F-X012-design.md:66`
`.claude/plans/F-X012-design.md:101`

The remediation rejects step-level `if` and `continue-on-error`, but it does
not reject success short circuits inside the run body. A focused mutation added
`exit 0` before the exact installer command in the Test step.
`assert_poppler_consumers_contract()` still accepted the mutated workflow
because the command, step fields, count, and ordering remained present. The
separate exact Python job assertion happens to reject this mutation for the
matrix job, but no equivalent assertion protects Test, MSRV, or Presentation
fidelity. Apply the existing `assert_no_success_short_circuit()` helper to every
installer run body and add the corresponding negative mutation.

## Smells

None.

## Nitpicks

None.

## Pass 1 dispositions

- D1 is fixed. Extraction uses streaming XZ mode, counts each member before
  extraction, rejects the 2,049th member immediately, and no longer calls
  `getmembers()` (`scripts/install_pinned_poppler.py:52`,
  `scripts/install_pinned_poppler.py:56`,
  `scripts/test_sprint_workflow.py:371`,
  `scripts/test_sprint_workflow.py:386`).
- D2 is fixed. Any existing file or non-empty directory at the selected prefix
  fails before download, so a successful invocation cannot reuse tools based
  on version text alone (`scripts/install_pinned_poppler.py:118`,
  `scripts/install_pinned_poppler.py:123`,
  `scripts/test_sprint_workflow.py:401`,
  `scripts/test_sprint_workflow.py:419`).
- D3 is partially fixed by real checksum, member-count, and runtime probes
  (`scripts/test_sprint_workflow.py:359`). D1 records the remaining behavioral
  coverage gap.
- D4's cited conditional and failure-tolerant step policies are fixed. The
  direct step-field contract and eight negative policy mutations reject both
  forms for all four consumers (`scripts/test_sprint_workflow.py:139`,
  `scripts/test_sprint_workflow.py:151`,
  `scripts/test_sprint_workflow.py:426`,
  `scripts/test_sprint_workflow.py:448`). D2 records the remaining in-body
  short-circuit gap.

## Focused evidence

- All 43 workflow tests pass, both Python files compile, and the exact focused
  early-success mutation described in D2 is accepted by the consumer helper.
  The hash harness remains 28 of 28. Prose, generated-skill synchronization,
  and diff hygiene pass.
- The workflow itself remains correctly wired. Test, both Python rows,
  Presentation fidelity, and MSRV invoke the one installer before their
  Poppler-dependent work (`.github/workflows/ci.yml:26`,
  `.github/workflows/ci.yml:58`, `.github/workflows/ci.yml:224`,
  `.github/workflows/ci.yml:368`). Each current installer step is unconditional
  and failure-propagating. No package manager installs Poppler, and LibreOffice
  remains separate (`.github/workflows/ci.yml:221`).
- The source checksum and official version remain exact, extraction paths and
  unsupported member types fail closed, work is isolated, build concurrency is
  capped at four, and only the three requested tools are copied and verified
  (`scripts/install_pinned_poppler.py:18`,
  `scripts/install_pinned_poppler.py:43`,
  `scripts/install_pinned_poppler.py:62`,
  `scripts/install_pinned_poppler.py:70`,
  `scripts/install_pinned_poppler.py:125`,
  `scripts/install_pinned_poppler.py:161`,
  `scripts/install_pinned_poppler.py:175`).
- Binaryen still verifies the reviewed archive checksum before requiring the
  exact official Linux identity (`.github/workflows/ci.yml:122`,
  `.github/workflows/ci.yml:126`). The existing exact job contract and negative
  checksum and identity mutations remain intact
  (`scripts/test_sprint_workflow.py:790`,
  `scripts/test_sprint_workflow.py:806`,
  `scripts/test_sprint_workflow.py:1049`,
  `scripts/test_sprint_workflow.py:1056`).
- The progress record reports successful remediated macOS installation and
  fail-closed populated-prefix behavior
  (`.claude/scratch/F-X012-progress.md:54`,
  `.claude/scratch/F-X012-progress.md:55`). Product code, crate metadata,
  package versions, publication authority, and rendering baselines remain
  unchanged. The exact planned HLD impact is still 12, 14, and 15
  (`.claude/plans/F-X012-design.md:75`).

## Not found

No functional regression in streaming extraction or populated-prefix refusal,
unsafe archive path, accepted unsupported member type, unbounded build
parallelism, wrong source checksum or runtime literal, missing Poppler consumer,
moving Poppler package installation, Binaryen checksum-order error, wrong
Binaryen identity, product-code change, public API change, dependency change,
package or release mutation, rendering-baseline delta, hash delta, unapproved
file, HLD scope omission, sprint-state mismatch, structural indirection, prose
violation, smell, or nitpick was found beyond D1 and D2.
