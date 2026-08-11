---
description: Release an already prepared and reviewed stable or incubating Rust family. The only command that creates and pushes v* or rpptx-v* release tags or starts crates.io publication.
---

# /release {vX.Y.Z | rpptx-vX.Y.Z}

Release the exact reviewed sprint SHA for one Rust package family. This is the
only command allowed to create or push a stable `v*` tag, create or push an
incubating `rpptx-v*` tag, or start crates.io publication. It never merges to
`main` and never creates an `sNN` sprint tag.

The version preparation is committed through its F-ID before this command
runs. This command does not edit versions, create a release commit, repair a
red gate, or switch a release from one family to the other.

## Family contract

Choose exactly one family from the requested tag before running any check.

### Stable family

For `vX.Y.Z`, `[workspace.package].version` must be exactly `X.Y.Z`. The exact
seven-package stable set is `rdocx-opc`, `rdocx-oxml`, `rdocx-layout`,
`rdocx-html`, `rdocx-pdf`, `rdocx`, and `rdocx-cli`, each at `X.Y.Z` through
workspace inheritance and matching `[workspace.dependencies]` pins.
`rdocx-wasm` may inherit the workspace version, but it remains
`publish = false` and is not in the crates.io set.

### Incubating family

For `rpptx-vX.Y.Z`, every selected package manifest and its corresponding
`[workspace.dependencies]` pin must be exactly `X.Y.Z`. The exact 12-package
incubating set is `oxml-core`, `oxml-opc`, `oxml-media`, `oxml-layout`,
`oxml-drawing`, `oxml-pdf`, `oxml-sml`, `rpptx-oxml`, `rpptx-chart`,
`rpptx-layout`, `rpptx-render`, and `rpptx`. Stable packages are not in this
set. Binding, WASM, and unimplemented CLI support crates remain outside it.

## Preconditions

Refuse before any tag or push if one check fails:

1. The argument is exactly `vX.Y.Z` or `rpptx-vX.Y.Z`, and the selected family
   satisfies its complete version and package contract above. Reject any other
   prefix, suffix, mixed family, or partial version preparation.
2. The current branch is the active `sprint/sNN` branch and the tree is clean.
3. The release F-ID named `Tag <requested-tag>` is `reviewed` in the sprint run
   state, remains `in-progress` in both delivery trackers, and every dependency
   is completed.
4. The latest recorded `/verify --full` passed at the current HEAD with the
   declared hash-harness result.
5. The latest recorded `/sprint-review SNN` is clean at the current HEAD, and
   its review file reports zero blocking findings.
6. `cargo metadata --no-deps` at the reviewed HEAD confirms the exact selected
   family, its versions, publication eligibility, and internal version pins.
   The other family's packages must not appear in the selected workflow
   allowlist.
7. `cargo publish --workspace --dry-run` passes from the clean tree. A dry-run
   uploads nothing. It must stage exactly the 19-package union of the two
   family sets because all are publication candidates, and every archive must
   remain below 10 MiB. The `rdocx-layout` and `oxml-layout` archives must
   contain their complete bundled TTF and licence inventories. The `rpptx`
   archive must contain `assets/default.pptx`.
8. `.github/workflows/publish.yml` binds the stable predicate to exactly the
   stable set and the incubating predicate to exactly the incubating set, each
   in dependency order. Every real publish command is the bare verified form
   `cargo publish -p <package>`, failures propagate, and registry waits remain
   between dependency layers.
9. Fetch the remote release-tag namespaces. The exact requested tag must be
   absent locally and from `origin`. Refuse a conflicting or already-published
   version rather than treating it as success.

## Final approval

Report the exact HEAD SHA, requested tag, selected family, selected package
set, version, remote, and workflow that will run. Ask for a separate explicit
go or no-go immediately before the first external mutation. Approval given
earlier in the feature or sprint does not count at this boundary.

## Release

After approval, preserve this order:

1. Push the active `sprint/sNN` branch at the reviewed HEAD.
2. Create one annotated tag for the requested argument at that exact HEAD with
   message `Release <requested-tag>`.
3. Push only that requested tag. The tag starts
   `.github/workflows/publish.yml`, whose matching predicate publishes only the
   selected family with verification and then creates the GitHub release.
4. Watch the workflow through completion. A failed job is a failed release.
   Do not rerun blindly and do not convert an authentication, network,
   compilation, duplicate-version, or registry failure into success.
5. Verify `cargo info <package>@X.Y.Z` for every package in the selected set,
   verify the owner for each registry entry, and inspect the GitHub release tag
   and target SHA. Do not claim the unselected family was published.

If the branch push succeeds but tag push fails, report that exact state. If the
tag push succeeds but publication fails, retain the tag and report the failed
package and workflow. Do not delete or move a published release tag.

## Finalise the release F-ID

Only after every selected registry version and the matching GitHub release are
verified:

1. Create the F-ID's `AS_BUILT.md` entry with the release evidence.
2. Complete its sprint tracker and backlog records, clear its owner, and set
   its design plan to completed.
3. Record the release F-ID completed in sprint state.
4. Re-run the sprint's ledger checks and continue `/run-sprint` to its final
   review and `/close-sprint` handoff.

## Refused situations

- A version bump or uncommitted change is still required.
- Verification or sprint review covers a different SHA.
- The requested tag and prepared family do not match exactly.
- A local dry-run is offered as a substitute for successful publication.
- Any command would merge to `main`, create an `sNN` tag, or create both
  release-family tags.
- The user has not given the separate final approval.
