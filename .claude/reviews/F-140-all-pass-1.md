# F-140, all, pass 1

**Reviewed**: the complete seven-file working diff, 372 insertions and 34 deletions, against the approved plan, progress notes, and HLD 10, 12, 14, and 15
**Verdict**: 1 defect, 0 smells, 1 nitpick

## Defects

### D1, release preparation sensitivity protects the group count but not the wrapper's lockstep metadata

`scripts/test_sprint_workflow.py:2592`
`scripts/test_sprint_workflow.py:2617`
`scripts/test_sprint_workflow.py:2676`
`scripts/test_sprint_workflow.py:2696`
`docs/hld/15-build-and-toolchain.md:206`

The new exact 13-member assertion classifies manifests only by
`shared-version`, while the existing regressions that require the incubating
`tag-name` and explicit 0.1.2 version still enumerate only the 12 published
packages. Changing only `crates/rpptx-wasm/Cargo.toml` from
`tag-name = "rpptx-v{{version}}"` to `tag-name = "v{{version}}"` leaves the new
13-member contract and its one shared-version mutation test green. No other
workflow regression mentions the wrapper's release metadata. A wrapper version
mutation is likewise outside the explicit incubating version test. The HLD now
states that all 13 preparation-group manifests carry the incubating tag
template at explicit version 0.1.2, so the regression does not prove the
reconciled contract it records. Include `rpptx-wasm` in equivalent tag and
version checks, with its unpublished and no-workspace-dependency differences
handled explicitly, and make representative tag or version drift fail.

## Smells

None.

## Nitpicks

- `.github/workflows/ci.yml:95`, the pinned setup-node commit is
  `249970729cb0ef3589644e2896645e5dc5ba9c38`, which upstream tags as v6.5.0,
  while the provenance comment says v6.1.0. Upstream v6.1.0 points to
  `395ad3262231945c25e8478fd5baf05154b1d79f`. The operative SHA is the exact
  commit approved by the plan, so this does not weaken execution, but the
  review annotation is factually wrong.

## Not found

- Workflow trigger, privilege, and control flow produced no finding. The root
  permission remains exactly `contents: read`, the pull-request trigger is
  unconditional, the WASM job has no condition, and every command step uses
  ordinary failure-propagating bash without `continue-on-error`, fallback
  success, or an early successful exit.
- Action and tool pins produced no operative finding. Checkout v6.0.2, the
  stable Rust action commit, rust-cache v2.9.1, and the approved setup-node
  commit resolve to the exact full SHAs in the job. Node is exactly 24.11.1,
  and wasm-pack is installed as exact 0.15.0 with its published lockfile.
- Target and behavioral execution produced no finding. Both exact locked
  wasm32 checks passed. Both exact `wasm-pack test --node` commands passed and
  executed one non-vacuous package-preservation test per wrapper. The updated
  HLD correctly leaves the presentation render-profile and optimized-size
  gates local.
- Structured workflow sensitivity produced no finding. The positive and
  declared mutation tests passed. Additional probes confirmed that install
  conditions, false `continue-on-error`, an extra test filter, a test-step
  environment override, and job-level write permission are rejected. Exact
  step and command ordering prevents either package gate from being omitted or
  converted to listing-only execution.
- Dependency routing produced no finding. The inspected default WASM trees
  contain neither PyO3 nor `getrandom`. The only cross-family edge visible in
  the presentation tree is the documented `oxml-drawing` to `rdocx-oxml`
  theme-adapter exception.
- HLD and file scope produced no finding. Exactly the four plan-listed HLD
  files changed, and their PR-time default-profile claims match the workflow.
  No new file, module, release action, publication authority, baseline change,
  or unlisted specification edit was introduced. Prose, generated-skill sync,
  and diff hygiene passed.
