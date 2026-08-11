# F-129, all, pass 2

**Reviewed**: working implementation diff from claim base `aba870d`, 8 files,
322 changed lines, with 298 additions and 24 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D2, The new workspace member has no release-family metadata
`crates/oxml-py-support/Cargo.toml:4`
`scripts/test_sprint_workflow.py:363`
`docs/hld/15-build-and-toolchain.md:183`

The new crate inherits the stable workspace version but its manifest has no
`package.metadata.release` table. The release-safety regression iterates every
workspace member and reads that table, so it now fails with `KeyError:
'metadata'` before it can verify the family counts. The HLD also still states
that exactly eight packages inherit the workspace release version. F-129 must
choose and encode this unpublished crate's release-preparation family, keep the
regression consistent with that choice, and list the affected HLD file before
completion.

## Smells

None.

## Nitpicks

None.

## Not found

- D1 remediation: the shared error now stores caller-supplied recovery
  guidance, includes it in `Display`, and the gate test pins the complete
  paragraph recovery message. D1 is resolved.
- Correctness: no additional wrong path ordering, revision comparison, counter
  behavior, or unit conversion was found.
- Contract: apart from D2, the implementation matches the approved Word-only
  path inventory, stale-domain ownership split, conversion delegation, and
  listed HLD changes.
- Panics: no reachable panic, unchecked index, slice, or untrusted arithmetic
  issue was found. Revision overflow still requires exhausting the private
  monotonic `u64` counter.
- OOXML: this diff adds no parser, serializer, namespace, child-order, or raw
  XML behavior.
- Tests: all five crate tests, the focused all-targets check, clippy, rustfmt,
  prose check, and generated-skill check passed. The release-safety regression
  failed as described in D2.
- Structure: no unjustified trait, generic, dynamic dispatch, forwarding
  wrapper, feature flag, or format-specific dependency was found. The normal
  dependency tree contains only `oxml-core`, `smallvec`, and `thiserror`.
