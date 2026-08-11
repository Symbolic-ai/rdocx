# F-050, all, pass 2

**Reviewed**: working-tree diff against `5b2fa198`, 1 file and 44 changed
lines, with 40 additions and 4 deletions
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Not found

- Correctness, the Test, Clippy, Docs, and MSRV commands each carry both exact
  binding exclusions in valid Cargo positions at
  `.github/workflows/ci.yml:23`, `.github/workflows/ci.yml:103`,
  `.github/workflows/ci.yml:126`, and `.github/workflows/ci.yml:212`. All four
  exact commands pass, including the MSRV test command under Rust 1.93.
- Contract, `.github/workflows/ci.yml:27`, `.github/workflows/ci.yml:36`, and
  `.github/workflows/ci.yml:47` add exactly the three focused jobs approved by
  the design. No release, publication, Python wheel, or future `rpptx-wasm`
  behavior is added.
- Panics, the change is declarative CI configuration and adds no indexing,
  slicing, arithmetic, or unchecked runtime operation
  (`.github/workflows/ci.yml:23`).
- OOXML, no parser, serializer, namespace, schema-order, whitespace, or
  unmodelled-subtree behavior is changed (`.github/workflows/ci.yml:1`).
- Tests, the no-default job runs the exact package test and passes
  (`.github/workflows/ci.yml:34`). The WASM job runs the exact target check and
  passes (`.github/workflows/ci.yml:45`). Both Python gates pass
  (`.github/workflows/ci.yml:52`).
- Structure, the diff changes one existing workflow file and adds no source
  file, module, crate, feature flag, trait, generic, wrapper, or indirection
  (`.github/workflows/ci.yml:15`).
- GitHub Actions semantics, the complete workflow parses as YAML with 12 unique
  job identifiers. Every new job checks out the repository, both Rust jobs
  select a toolchain, and no new job has an unmet dependency
  (`.github/workflows/ci.yml:27`).
- Target installation, the WASM job installs `wasm32-unknown-unknown` through
  the toolchain action before checking `rdocx-wasm`
  (`.github/workflows/ci.yml:40`).
- No-default behavior, the focused command disables the default
  `system-fonts` feature while retaining the bundled deterministic path, and
  its 62 tests plus 2 doctests pass (`.github/workflows/ci.yml:34`).
- WASM scope, `rdocx-wasm` is checked for the installed target and the future
  `rpptx-wasm` package remains outside this story
  (`.github/workflows/ci.yml:36`).
- Prose and generated skills, the two read-only gates are separate steps in one
  focused job and both pass (`.github/workflows/ci.yml:47`).
- Duplication, there is exactly one no-default job, one WASM job, and one prose
  and generated-skills job (`.github/workflows/ci.yml:27`). There are exactly
  four run commands containing `--all-features`, and all four contain both
  exact binding exclusions.
- Negative-path contract, the reviewed base has only the Test and MSRV
  all-feature commands without either exclusion and has none of the three
  focused job definitions. Reverting the diff therefore removes the new
  portability and repository-policy coverage proven at
  `.github/workflows/ci.yml:23` through `.github/workflows/ci.yml:53`.
- Hash behavior, the diff changes CI configuration only and contains no product
  or rendering behavior that could explain an output delta
  (`.github/workflows/ci.yml:1`).
