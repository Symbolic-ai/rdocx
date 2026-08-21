# F-X035, all, pass 1

**Reviewed**: the complete 45-file working diff, 337 changed lines, against the approved F-X035 plan and its release, release-note, HLD, dependency, archive, WASM, and workflow contracts
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, the release precondition miscounts the required local patches

`.claude/commands/release.md:66`

The canonical release command says that `/verify` step 10 contains 21 local
patches. The actual reviewed command contains one patch for every member of the
22-package publishable union, from `.github/workflows/publish.yml:32` through
`.github/workflows/publish.yml:53`, and the mutation-sensitive regression
requires exactly 22 at `scripts/test_sprint_workflow.py:3634`. This leaves the
release precondition internally inconsistent at the point where an operator is
required to validate the exact archive command. Correct the stated count to 22
and regenerate the command adapter before using `/release`.

### D2, the canonical full gate still omits the incubating WASM package

`.claude/commands/verify.md:59`

The release plan requires both WASM target checks and explicitly names
`rpptx-wasm` at `.claude/plans/F-X035-design.md:107`, but `/verify --full` still
checks only `rdocx-wasm` and carries a stale instruction to add `rpptx-wasm`
when F-138 lands. The crate has landed and is one of this story's 0.4.0 prepared
carriers. As written, a recorded full gate can satisfy the `/release`
precondition without compiling the incubating WASM graph. The combined direct
check passes in this working tree, so the code is compatible, but the canonical
gate does not preserve that proof. Add `-p rpptx-wasm` to step 8, remove the
stale note, and cover the command contract with the existing workflow
regressions.

## Smells

None.

## Nitpicks

None.

## Not found

- Version carriers and metadata: all 15 publishable incubating packages, all
  matching workspace pins, and all 16 lockfile and manifest preparation
  carriers are 0.4.0. The stable family remains 0.7.0, and `rpptx-wasm` remains
  unpublished.
- Dependency and publication structure: metadata reports exact internal 0.4.0
  requirements without a forbidden format dependency in an `oxml-*` crate.
  The publish predicates are disjoint, the 15-package incubating allowlist is
  exact and dependency ordered, commands are bare and fail closed, and waits
  remain between dependency layers.
- Release notes: the exact `rpptx-v0.4.0` section passes check and render. Its
  chart, provenance, cache, OPC, deterministic PDF, compatibility, and WASM
  claims are supported by the `rpptx-v0.3.0..HEAD` range and remain scoped to
  the shared OOXML and PowerPoint family. GitHub records confirm
  `@emptinessform` for Issues 38 and 39, `@pedroassumpcao` for PRs 33 and 34,
  and `@jonstokes` for the entry-admission commit in PR 34.
- HLD prepared state: only the two plan-listed HLD files change. They describe
  the current 0.4.0 preparation, the planned stable 0.8.0 source boundary, the
  exact 15-package crates.io set, and the unpublished WASM state without
  claiming that tagging or publication has occurred.
- Archives and compatibility: all 15 selected crates package successfully with
  the working changes. Every archive is below 10 MiB, `oxml-layout` includes
  its bundled fonts and legal files, and `rpptx` includes
  `assets/default.pptx`. A combined locked wasm32 check for `rdocx-wasm` and
  `rpptx-wasm` passes.
- Tests and diagnostics: all 62 sprint-workflow regressions pass, including the
  renamed 0.4.0 preflight. Release-note check and render, prose, generated-skill
  drift, metadata, archive listing, and diff checks pass. The exact 0.4.0 tag is
  absent locally and from `origin`.
- No panic, OOXML-order, unmodelled-XML, or structural-rule issue is introduced
  by this version and release-record diff.
