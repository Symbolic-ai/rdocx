# F-X176, Repair unified release inventory and respin PowerPoint

**Status**: completed
**Sprint**: S88
**Size**: M
**Depends on**: F-X175

## Problem

The first `rpptx-v0.13.0` tag built all six CLI archives and both sets of Python artifacts, then its unified asset job failed before either registry job. `.github/workflows/wheels.yml:520` runs `cargo publish --workspace --dry-run`, while the next step at line 544 expects selected `.crate` files in `target/package`. A clean Rust 1.97.1 dry run leaves unpacked package directories but removes its `.crate` files. The tag is immutable and cannot be used for a repaired release. The PowerPoint version carriers and release notes still name 0.13.0.

## Spec reference

`docs/hld/14-development-backlog.md`, F-X176, defines the release regression gate. `docs/hld/15-build-and-toolchain.md`, the unified family publication section, defines the prepublication package inventory and family boundary. `.claude/commands/release.md`, Preconditions and Release, defines immutable tags and separate final approvals.

## Approach

Validate the unpacked package directories from the successful workspace dry run. Put the exact selected-family comparison in `scripts/sprint_workflow.py` and call it from `wheels.yml`, replacing the `.crate` file assumption. Reject missing, extra, or malformed selected-version package directories. Test this function against an empty fresh target and an exact populated target, including both family versions.

Move all 15 shared OOXML and PowerPoint crate manifests, their internal pins and the `rpptx-py` binding and Python distribution carriers to the new approved release version. Refresh current-version README examples, lockfile entries, release policy assertions, and the PowerPoint changelog section. Preserve external dependency version strings and historical release records. Keep seven stable Word crate versions and Python `rdocx` at 0.15.0, updating only shared dependency pins they consume. Prepare both tag contracts and notes from the same source tree.

## Rejected alternatives

- Moving or deleting the failed tag would make the reviewed release history mutable.
- Rerunning the failed workflow would repeat the deterministic clean-target failure.
- Treating a successful dry run as proof that `.crate` files remain would keep a false gate.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | Fresh-directory package inventory tests in `scripts/test_sprint_workflow.py` | Exact selected package directories pass. Missing, extra, and malformed directories fail for both families |
| version | `unified-release-family` and `release-notes --check` for both tags | All manifest, Python and lockfile carriers agree with each tag and reviewed notes |
| package | Patched `cargo publish --workspace --dry-run` and selected inventory command on a clean target | 22 publishable crates package. Selected 15 and 7 directories match, with sizes below 10 MiB |
| artifact | Manual `wheels.yml` build-only dispatch at the reviewed sprint SHA | Both six-wheel and sdist sets, metadata, installed runtime, typing and stubs pass without publication |
| release regression | Hosted Release regressions and Docs jobs at the final sprint SHA | Version and workflow contracts pass before close |
| integrated | `/verify --full`, hash harness and clean sprint review | Complete gate passes with 49 unchanged hashes |

The backlog test gate is release regression.

## HLD impact

- `docs/hld/14-development-backlog.md`, F-X176 test gate and final release version.
- `docs/hld/15-build-and-toolchain.md`, current selected-version carriers and dry-run package inventory mechanism.

## Risk routing

The release scripting and version strings row applies. Read `.claude/commands/release.md` and `docs/hld/15-build-and-toolchain.md`, inspect every manifest, lockfile and README version diff, run the complete package dry run and full gate, and require separate final approval before each new tag. The WASM or PyO3 bindings row also applies to binding version carriers: run the WASM target check and exclude both Python bindings from the workspace test command.

## Hash harness

Expected unchanged, with all 49 entries matching. The release workflow and version metadata do not change document output.

## Implementation checklist

- [x] Reproduce and test clean-target inventory behavior.
- [x] Replace the tag job's archive-file assertion with the selected package-directory contract.
- [x] Refresh PowerPoint version carriers, notes, README examples and policy tests.
- [x] Keep stable Word 0.15.0 and update its shared pins.
- [x] Complete scoped verification and a zero-finding microscope review.

## Open questions

None. The user selected 0.13.1 for the replacement PowerPoint version and
retained 0.15.0 for Word.
