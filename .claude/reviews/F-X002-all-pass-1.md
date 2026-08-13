# F-X002, all aspects, pass 1

**Reviewed**: working-tree implementation, 6 files, 148 insertions and 16 deletions
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Not found

- Correctness. The runner requires exactly six `rust,no_run` fences at
  `scripts/readme_doctests.py:20`, builds the current workspace `rdocx` package
  with locked dependencies at `scripts/readme_doctests.py:36`, and rejects a
  failed build or an artifact set other than one rlib. The exact gate compiled
  all six tracked examples.
- Cargo artifact binding. The JSON filter at
  `scripts/readme_doctests.py:62` accepts only a compiler artifact named
  `rdocx` with a library crate type, then accepts only an rlib filename. The
  current Cargo record contains one such package artifact and one rlib. The
  dependency search path at `scripts/readme_doctests.py:95` handles both the
  unhashed target-directory rlib and a hashed `deps` rlib.
- Rustdoc contract. The direct invocation at
  `scripts/readme_doctests.py:98` supplies test mode, the 2024 edition,
  warning denial, the dependency search path, and the exact discovered
  `--extern rdocx` artifact. `no_run` keeps filesystem examples compile-only.
- README API. All six Rust fences use the exact marker. The read example at
  `README.md:87` uses total `row_count` and `row` access, then total
  `cell_count` and `cell` access. No public iterator was added to preserve the
  former invalid example.
- CI and verify wiring. The existing docs job calls the runner directly and
  unconditionally at `.github/workflows/ci.yml:265`. The canonical non-fast
  docs step calls the same owner at `.claude/commands/verify.md:50`, and the
  generated adapter hash is in sync at `.agents/skills/verify/SKILL.md:10`.
- Tests and sensitivity. The exact tracked gate passes six examples. The
  recorded invalid-iterator mutation fails with E0599 at
  `.claude/scratch/F-X002-progress.md:19`, then the byte-restored README passes.
  Reverting the fence changes also fails the exact fence validator before a
  false green is possible.
- Artifact hygiene. The rustdoc gate writes normal ignored Cargo artifacts
  only. The output scan recorded at `.claude/scratch/F-X002-progress.md:23`
  found no generated document or sample output, and the review rerun produced
  no non-target artifact.
- Contract and structure. The diff stays within the approved README, runner,
  CI, canonical verify, generated adapter, and plan paths. The runner is the
  single owner of artifact discovery and rustdoc arguments. No crate, module,
  trait, generic, wrapper, dependency, feature flag, binary fixture, or second
  snippet source was added.
- Panics and OOXML. The change handles subprocess and artifact failure through
  nonzero status and does not parse, serialize, reorder, or discard OOXML.
