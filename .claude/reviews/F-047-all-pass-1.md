# F-047, all, pass 1

**Reviewed**: working-tree diff, 2 files and 74 changed lines
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Not found

- Correctness: the manifest makes `oxml-layout` packageable while preserving
  the explicit source, font, licence, and notice includes at
  `crates/oxml-layout/Cargo.toml:11`.
- Contract: the CI inventory names exactly 20 TTFs and the four required legal
  files at `.github/workflows/ci.yml:111` and `.github/workflows/ci.yml:141`.
- Panics: no Rust code or untrusted indexing changed.
- OOXML: no parser, serializer, namespace, or schema-ordering code changed.
- Tests: deleting any expected font or legal file makes the corresponding
  inventory diff fail at `.github/workflows/ci.yml:137` or
  `.github/workflows/ci.yml:151`.
- Structure: the change adds no trait, generic parameter, wrapper, feature
  flag, crate, module, or helper script.
- GitHub Actions shell correctness: both added steps select Bash explicitly at
  `.github/workflows/ci.yml:104` and `.github/workflows/ci.yml:155`. The two
  shell bodies pass ShellCheck, and command, pipeline, and comparison failures
  propagate to the job.
- Dirty and clean behavior: the committed CI commands intentionally omit
  `--allow-dirty` at `.github/workflows/ci.yml:112` and
  `.github/workflows/ci.yml:157`, so the job validates a clean checkout. Local
  review of the working diff used Cargo's dirty-tree allowance only as review
  evidence.
- Archive path and size logic: verified packaging precedes archive discovery,
  absence is rejected, GNU `stat` matches the Ubuntu runner, and the limit is
  exactly 10 MiB at `.github/workflows/ci.yml:157` and
  `.github/workflows/ci.yml:164`.
- Publish eligibility scope: only the `oxml-layout` guard is removed. No
  publication command, release tag, or workflow allowlist changes are added.
  Restoring the prior `publish = false` line is rejected at
  `.github/workflows/ci.yml:106`.
- Package inventory: the current package list contains every expected asset,
  and the checked legal files carry the Caladea Apache 2.0 and Carlito and
  Liberation OFL material required by the contract.
