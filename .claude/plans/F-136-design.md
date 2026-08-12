# F-136, rpptx-py

**Status**: completed
**Sprint**: S34
**Size**: L
**Depends on**: F-129, F-116

## Problem

The workspace has no `rpptx-py` crate or Python package. Shared paths remain
Word-only, and the presentation facade lacks the total immutable text and
placeholder accessors needed to re-resolve Python handles. The official
python-pptx Getting Started examples define a bounded compatibility surface,
but their import namespace conflicts with the required `rpptx` package name.

## Spec reference

- `docs/hld/03-architecture.md`, "Three families, one workspace" and "Facade conventions".
- `docs/hld/06-presentationml-model.md`, "Public facade boundary".
- `docs/hld/10-bindings-spec.md`, "The chosen design", "Python API shape", and "Packaging".
- `docs/hld/12-testing-strategy.md`, "Binding tests".
- `docs/hld/14-development-backlog.md`, "F-136, rpptx-py".
- `docs/hld/15-build-and-toolchain.md`, "Feature flags", "Packaging", and "Release process".

## Approach

Create an unpublished mixed-layout `rpptx-py` crate at workspace version 0.4.1
with an off-by-default `extension-module` feature and cp39-abi3 PyO3. A
`PyPresentation` owns `rpptx::Presentation` and a `RevisionCounter`. Path-only
slide, shape, text, table, and cell handles re-resolve through additive total
facade accessors on every operation. Extend `PathSeg` with `Slide(usize)` and
repeatable `Shape(usize)`. Successful structural mutations bump exactly once,
failed mutations and scalar writes do not, and stale errors report captured and
current revisions with a complete recovery path.

Implement the seven python-pptx 1.0.2 Getting Started examples: Hello World,
bullet slide, textbox, picture, preset shapes, table, and extract all text.
Preserve each example body after the `pptx` to `rpptx` namespace substitution
and the minimal public re-fetches required after structural writes by the
global revision contract. Provide lazy layouts, slides, shapes, placeholders,
text frames, paragraphs, runs, columns, and cells. Provide pure-Python
`Length`, `Inches`, `Pt`, and the required `MSO_SHAPE` members. Mirror the
package-specific error hierarchy. Generate the tiny PNG input in code and
compare normalized public structure against pinned `python-pptx==1.0.2`, never
package bytes. Compare the normalized rpptx-authored and python-pptx-authored
records directly so writer-only drift fails the gate.

Keep each behavior-bearing class family in its owning source file. Add no
resolver trait, generic abstraction, binary fixture, forwarding-only module, or
runtime dependency on either Python oracle package.

## Rejected alternatives

- Hold Rust facade borrows in pyclasses. They cannot satisfy `'static` and
  would alias after mutation.
- Reach into `rpptx-oxml`. The Python binding belongs above the facade.
- Ship a `pptx` alias package. It would collide with the pinned oracle.
- Implement the whole python-pptx user guide or chart object graph. The named
  seven examples are the bounded L-sized contract.
- Compare `.pptx` bytes. The oracle comparison is structural.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| integration, gate | `python_pptx_getting_started_examples_run_with_global_revision_refetches` | All seven documented workflows execute through `rpptx` with only namespace substitution and required post-structural-write re-fetches |
| integration | lazy collection and stale-handle suite | Index, slice, iteration, global revision, and exact recovery behavior is loud and total |
| differential | two-reader and two-writer normalized example records | rpptx and python-pptx 1.0.2 agree structurally, including direct cross-writer equivalence |
| regression | facade totality and immutable resolution | Binding reads do not mutate and every path can re-resolve |

The test gate is the seven pinned python-pptx Getting Started workflows with
the package namespace changed and the minimal re-fetches required after
structural writes by strict global revision invalidation.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/06-presentationml-model.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- Unit conversion. Preserve truncation toward zero and pin positive and
  negative fractional `Inches` and `Pt` cases.
- Crate dependency graph. Inspect normal and development trees and the inverse
  PyO3 tree. Keep every shared edge inward.
- Public API of published `rpptx`. State additive semver impact, run the full
  publication dry run, and assert archive sizes.
- WASM or PyO3 bindings and a new feature. Keep `extension-module` off by
  default, name maturin as its consumer, retain binding exclusions, run both
  existing WASM and binding-isolation checks, and run the layout
  no-default-features gate. `rpptx-wasm` remains deferred to F-142, so this
  story runs the existing `rdocx-wasm` target plus inverse dependency proof
  that PyO3 does not leak into format crates.
- New crate, modules, and files. Obtain explicit approval for the exact crate
  tree listed in the open questions.
- External oracle comparison. Pin `python-pptx==1.0.2`, assert the version, and
  compare normalized public trees.
- Release and version strings. Inspect workspace metadata, lockfile,
  `pyproject.toml`, release-family counts, and keep the binding unpublished.

## Hash harness

Expected unchanged. New bindings and additive facade accessors do not affect
the existing sample generators.

## Implementation checklist

- [x] Add `Slide` and repeatable `Shape` path segments with stale tests.
- [x] Add only the total additive rpptx facade accessors required by the seven examples.
- [x] Create the mixed-layout crate and path-only Python handle graph.
- [x] Add pure-Python units, shape enums, errors, and top-level exports.
- [x] Execute all seven examples and both-reader structural checks.
- [x] Run every dependency, PyO3, WASM, publication, and hash rider.

## Open questions

None. The seven-example inventory, namespace substitution plus required global
revision re-fetches, mirrored error names, additive facade surface, and exact
new crate tree are explicitly approved.
