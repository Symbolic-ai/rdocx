# F-X171, Pin LibreOffice for macOS Python acceptance

**Status**: completed
**Sprint**: S85
**Size**: S
**Depends on**: F-X168

## Problem

The S84 main CI run failed the original Issue 158 deck acceptance case at
`crates/rpptx-py/tests/test_documented_examples.py:4656`: the macOS Python job
in `.github/workflows/ci.yml:210` has Poppler but no LibreOffice. The same
suite passed locally with exact LibreOffice 26.2.5.2. A green local gate cannot
substitute for the required hosted CI result before contribution closure.

## Spec reference

`docs/hld/12-testing-strategy.md`, “What CI runs” and the Issue 158 criterion
ledger. `docs/hld/14-development-backlog.md`, “F-X171, Pin LibreOffice for
macOS Python acceptance.”

## Approach

Add one macOS-only step before the presentation binding suite in the existing
`python-bindings` job. Download the official aarch64 26.2.5 image, verify its
SHA-256 `c99fb4fe574437fc4cb820a4ca15271bca325920861f7139858b36d7f9df78ad`,
mount it read-only, and verify the executable reports the exact reviewed
26.2.5.2 build. Export its path through `GITHUB_ENV` and `GITHUB_PATH` so the
Issue 158 fixture test and other viewer consumers resolve the same image.
Keep Linux jobs and the existing Python matrix unchanged. Extend the workflow
contract test to require this setup before the full suite and reject removal,
skipping and success short circuits.

## Rejected alternatives

- `brew install --cask libreoffice`: its latest version can move without a
  reviewed oracle change.
- Skip the fixture when `soffice` is absent: that would erase the acceptance
  gate that discovered the setup defect.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| workflow | `python3 -m unittest scripts.test_sprint_workflow` | The macOS viewer step is pinned, ordered before the binding suite and cannot be bypassed. |
| local oracle | `soffice --version` and full `rpptx-py` pytest directory | The pinned 26.2.5.2 image and all 77 local binding cases pass. |
| hosted integration | `Python bindings (rpptx)` and aggregate `CI gate` on final main SHA | The original Issue 158 deck test and all selected hosted jobs pass at sprint closure. |

The backlog's **workflow regression** test gate is the F-ID completion
criterion. Hosted CI is the S85 closure criterion after the reviewed main push.

## HLD impact

- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

External oracle comparison: read `.claude/skills/differential-testing.md`.
Pin and check LibreOffice's exact build and archive digest, then run the Issue
158 deck fixture with that oracle. No output baseline is expected to change.

## Hash harness

Unchanged, all 49 entries must match.

## Implementation checklist

- [x] Add the SHA-pinned macOS viewer setup to `python-bindings`.
- [x] Make the workflow assertion fail if setup is removed or bypassed.
- [x] Update the HLD CI table and run scoped verification.

## Open questions

None. The local image digest and version were checked against the official
download headers and mounted executable on 3 October 2026.
