# S42 sprint review, pass 2

**Reviewed**: `sprint/s42` at `3937a36`, which is `d69dc5a` from pass 1 plus two
commits. The incremental delta is the pass 1 review file, the F-X025 filing, and
the release finalisation: two AS_BUILT entries, two plan status changes and the
tracker rows. **No product code changed**, confirmed by
`git diff --name-only d69dc5a..HEAD` returning nothing under `crates/`.
**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have

This pass exists because `close-preflight` requires the review to cover the
exact HEAD being merged, and the release finalisation landed after pass 1.

## Blocking

None.

## Should-fix

None outstanding. S1 from pass 1, that `/verify` does not run the release
regressions, is filed as **F-X025** with its own test gate. It is not fixed
here, deliberately: editing a command file during a release sprint is how the
gate that everything else depends on gets changed without review.

Worth recording that the gap did not bite twice. F-X023 ran
`python3 -m unittest scripts.test_sprint_workflow` directly as part of its own
gate, and this sprint's `/verify --full` ran it too.

## Nice-to-have

None outstanding. N1, the incubating train having no shared version key, and N2,
the F-X024 invariant test being unable to exercise its own crate, are both
carried forward in their stories' reviews and neither blocks anything.

## Milestone gate

S42 closes no milestone. All eight clauses of the sprint's definition of done
held at pass 1 and are re-confirmed here, with clauses 6 to 8 now carrying
registry evidence rather than preparation evidence:

- **Versions.** Fifteen incubating packages at 0.3.0, eleven workspace-version
  packages at 0.7.0, no `0.2.0` or `0.6.0` anywhere in the workspace.
- **Publication sets.** Exactly fourteen incubating and seven stable packages
  resolve from crates.io under owner `mantissaman`. The four unpublished
  Python and WASM packages inherited their version without publication
  authority, and `rpptx-wasm` stayed `publish = false`.
- **Approval.** Both annotated tags dereference to `ab52cd2`, the SHA pass 1
  reviewed. Nothing was tagged or published before that approval, and the two
  release F-IDs stayed `reviewed` until the registry could be checked, which is
  the release-preparation exception working rather than being worked around.

`/verify --full` passed all eleven steps at this HEAD, including the two only
`--full` reaches: the patched 21-package dry run with every archive under
10 MiB, and `cargo deny check` reporting advisories, bans, licenses and sources
ok. Recorded against `3937a36`.

## Not found

Re-checked after the finalisation, all clean: **interaction**, **duplication**,
**layering**, **deps**, **harness**, **surface**, **docs**. The delta is
documentation and ledgers, so none of these could have moved. The harness is 28
of 28, consistent with every AS_BUILT entry in the sprint.

## Exit

Zero blocking across two passes. The sprint is ready to merge.
