# S86 sprint review, pass 2

**Reviewed**: incremental remediation `0b36e552..e76cf36f`, 8 files, 85 changed lines, plus its interaction with the S86 integrated tree. Crates touched: `rdocx-cli`, `rpptx-cli`.
**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have

## Blocking

None.

## Should-fix

None.

## Nice-to-have

None.

## Milestone gate

M24 remains open for S87 through S91 work. S86's authoring and local release preparation gates remain satisfied. The first hosted build-only run at `0b36e552` exposed two Cargo-dependent tests in Alpine and Windows main-thread stack overflows in both CLIs. The musllinux selections now omit only those source-built CLI chains. The native wheel and pinned CI suites retain them. Both Windows CLIs run their commands on an eight MiB thread stack. A fresh six-platform hosted rehearsal at the final pushed remediation SHA remains required before sprint closure. The first failed run is not publication evidence.

## Not found

Interaction: the CLI stack change is Windows-specific. Unix CLI behavior and both local CLI suites pass.

Duplication: the two CLI entry points have the same small platform-specific stack rule because they are separate executables. No cross-crate forwarding layer was added.

Layering: no crate dependency changed.

Harness: all 49 entries match after remediation.

Gate: the hosted failures identify exact test names and Windows exit code `0xc00000fd`. The workflow policy suite checks the narrowed musllinux selections, and local CLI tests and Clippy pass. The second hosted run is the remaining release-preparation gate.

Docs: both release plans, HLD build policy, and release notes describe the observed environment and CLI changes.

Deps: no dependency was added.

Surface: no public Rust or Python API was added.
