# F-049, all, pass 2

**Reviewed**: uncommitted working diff, 23 files, 400 insertions and 127
deletions, plus the pass 1 review record
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Pass 1 resolution

- D1 is resolved. `/release` now selects exactly one stable or incubating
  family, applies the same reviewed HEAD, clean full verification, clean sprint
  review, exact family version and package, absent tag, separate final approval,
  requested-tag-only push, registry verification, owner verification, and
  GitHub release verification contract to both namespaces at
  `.claude/commands/release.md:17`. The stable seven-package boundary is at
  `.claude/commands/release.md:20`, and the incubating 12-package boundary is at
  `.claude/commands/release.md:29`.
- D2 is resolved. The helper isolates each named workflow step, compares its
  single predicate and complete command list exactly, checks dependency order
  from the manifests, and rejects any extra real publish command at
  `scripts/test_sprint_workflow.py:17`. The swapped-predicate and extra-package
  mutations are exercised at `scripts/test_sprint_workflow.py:372` and
  `scripts/test_sprint_workflow.py:394`.
- D3 is resolved. Each allowlist accepts only alternating bare
  `cargo publish -p <package>` and `sleep 60` commands, and it rejects
  `continue-on-error` at `scripts/test_sprint_workflow.py:55`. Dedicated
  negative mutations cover `continue-on-error` and a successful shell fallback
  at `scripts/test_sprint_workflow.py:411` and
  `scripts/test_sprint_workflow.py:425`.

## Not found

- Correctness: zero findings. The workflow predicates are disjoint and bind
  stable tags to seven packages and incubating tags to 12 packages at
  `.github/workflows/publish.yml:28` and `.github/workflows/publish.yml:44`.
  The real command order satisfies the current normal dependency graph, and
  every inter-package boundary retains a registry wait.
- Contract: zero findings. The 12 intended completed shared and PowerPoint
  packages are explicit candidates, while `rdocx-wasm` remains excluded at
  `crates/rdocx-wasm/Cargo.toml:13`. The candidate and exact allowlist
  regression is at `scripts/test_sprint_workflow.py:340`.
- Release safety: zero findings. The hash harness and workspace publication
  dry-run precede both real allowlists at `.github/workflows/publish.yml:20`.
  The release authority requires the dry-run to stage exactly the 19-package
  union, enforces the 10 MiB archive ceiling and required assets, and forbids
  any external mutation before a separate approval at
  `.claude/commands/release.md:40`. The dedicated `oxml-layout` CI archive
  inventory and size gate remains at `.github/workflows/ci.yml:141`.
- Publication boundary: zero findings. The implementation and tests perform
  only dry-run packaging. Real publication remains reachable only from a
  separately approved `/release` tag action described at
  `.claude/commands/release.md:72`. No tag, push, registry publication, or
  GitHub release occurred during this review.
- Adapter sync: zero findings. The generated adapter records the revised
  canonical release command hash at `.agents/skills/release/SKILL.md:8`, and
  `python3 scripts/sync_agent_skills.py --check` passes.
- Tests: zero findings. The focused seven-test release and workflow subset
  passes, including all four negative mutations. The exact global publish
  command comparison at `scripts/test_sprint_workflow.py:103` also prevents an
  extra publish step outside either named block.
- Panics: zero findings. No production Rust panic, unchecked index, slice, or
  arithmetic path is introduced. The Rust changes alter publication metadata
  assertions only.
- OOXML: zero findings. No parser, serializer, child order, namespace,
  whitespace, or unmodelled XML preservation behavior changes.
- Structure: zero findings. No trait, generic parameter, wrapper, feature
  flag, crate, module, or production source file is added. The change remains
  within the approved manifests, existing workflow, release command, generated
  adapter, and existing tests.
- Hash behavior: zero findings. The diff changes package and release mechanics,
  not generated OOXML or rendering behavior, matching the unchanged-harness
  contract at `.claude/plans/F-049-design.md:101`.
