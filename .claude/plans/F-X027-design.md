# F-X027, Wire the golden-PNG gate into something

**Status**: completed
**Sprint**: S44
**Size**: S
**Depends on**: none

## Problem

`scripts/golden_png_harness.py` generates deterministic sample PDFs,
rasterises page one at 150 DPI with pinned Poppler, and compares decoded pixels
with `scripts/golden_pixel_manifest.json`. The manifest pins
`pdftoppm version 26.01.0`, but neither `.claude/commands/verify.md` nor
`.github/workflows/ci.yml` invokes the harness. The gate therefore runs only
when someone remembers it.

The omission is specified in `docs/hld/14-development-backlog.md:2048` and was
recorded by `.claude/reviews/S43-sprint-review-pass-1.md`, finding N2. The byte
and structure hash harness does not replace this decoded-pixel comparison.

## Spec reference

- `docs/hld/12-testing-strategy.md`, "The golden-PNG gate" and "What CI runs".
- `docs/hld/15-build-and-toolchain.md`, "Deterministic rendering" and "CI job
  matrix".
- `docs/hld/14-development-backlog.md`, "F-X027, Wire the golden-PNG gate into
  something".

## Approach

Reuse the existing `test` job in `.github/workflows/ci.yml`. It already runs on
Ubuntu 24.04, builds pinned Poppler 26.01.0, and compiles the workspace. Add one
unconditional named step after the workspace suite that runs exactly
`python3 scripts/golden_png_harness.py --check`. This reuses the reviewed oracle
and Rust build artifacts without creating another Poppler consumer.

Add a workflow-contract helper and a mutation-sensitive regression to the
existing `scripts/test_sprint_workflow.py`. Assert exactly one check invocation,
its placement inside `test`, pinned-Poppler installation before it, and normal
failure propagation. Mutations will remove the command, move it before Poppler,
drop `--check`, and add a success short circuit. Do not edit the harness, hash
baseline, or golden pixel manifest.

## Rejected alternatives

- Add it to `/verify`. That would make the portable local gate depend on an
  expensive externally built raster oracle.
- Add it to `hash-harness`. That job deliberately has no Poppler installation,
  and the two harnesses answer different questions.
- Add a dedicated golden job. It would duplicate the pinned source build and
  Rust compilation for no stronger failure semantics.
- Add it to presentation fidelity or MSRV. Those jobs own unrelated product or
  compiler-compatibility concerns.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `test_ci_runs_the_golden_png_gate_in_the_pinned_poppler_environment` | The exact check runs once after the pinned Poppler install and propagates failures, with negative mutations proving sensitivity |
| regression | `GoldenPngHarnessTests.test_one_pixel_offset_is_rejected` | The harness rejects a one-pixel decoded-image change |
| regression | Clean and injected end-to-end harness runs | A clean tree passes with Poppler 26.01.0 and `--inject-one-pixel proposal` fails for the named sample |

The backlog test gate is **regression**: a deliberate rendering change fails
the gate wherever the story puts it, and a clean tree passes it.

## HLD impact

- `docs/hld/12-testing-strategy.md`, "The golden-PNG gate" and "What CI runs".
- `docs/hld/15-build-and-toolchain.md`, "CI job matrix".

## Risk routing

- **An external oracle comparison**. Read
  `.claude/skills/differential-testing.md`. Keep the Poppler oracle pinned in
  the harness and CI, record its exact 26.01.0 identity, and assert installer
  ordering. Run the clean and one-pixel injected checks in the pinned
  environment. Deterministic font mode remains mandatory for the generated
  PDFs.

## Hash harness

Expected unchanged at 49 of 49. No renderer, generator, hash baseline, golden
manifest, or harness implementation changes. Any delta blocks completion.

## Implementation checklist

- [x] Record the pre-change clean golden check and Poppler identity.
- [x] Add the unconditional named step after the workspace suite.
- [x] Add the CI contract helper and mutation-sensitive regression in the
  existing Python module.
- [x] Run the whole workflow regression module and the golden harness
  self-test.
- [x] Run the clean and expected-failing one-pixel checks.
- [x] Update only the listed HLD sections.
- [x] Confirm both manifests and the hash baseline remain byte-identical.
- [x] Run microscope and contribute the oracle evidence to the integrated full
  gate.

## Open questions

None.
