# F-134, Type stubs and py.typed

**Status**: completed
**Sprint**: S34
**Size**: M
**Depends on**: F-131, F-136

## Problem

The mixed `rdocx` package exposes native classes from `rdocx._rdocx`, but it
ships no stub or `py.typed` marker. The S34 package set also includes the new
`rpptx` binding from F-136, so typing only the Word package would leave the
sprint definition of done half complete. The backlog currently names only
F-131 even though `rpptx` cannot be typed before F-136 creates it.

## Spec reference

- `docs/hld/10-bindings-spec.md`, "Packaging".
- `docs/hld/12-testing-strategy.md`, "Binding tests".
- `docs/hld/14-development-backlog.md`, "F-134, Type stubs and py.typed".

## Approach

Hand-write one native-extension stub and add one `py.typed` marker inside each
mixed package. Keep the pure-Python units, enums, and exception hierarchies
inline typed rather than duplicating them in package-level stubs. Describe the
complete native runtime surface with concrete collection and handle types,
overloaded integer and slice access, typed iterators, path-like inputs, bytes
outputs, and optional values wherever `None` is meaningful.

Add one strict typing smoke program per package. Build and install fresh
cp39-abi3 wheels in an isolated environment, run `mypy==2.3.0 --strict`, run
`python -m mypy.stubtest` against both imports, and inspect the installed
packages to prove the marker and native stub were included.

## Rejected alternatives

- Generate stubs from PyO3. HLD10 explicitly requires hand-written stubs.
- Add package-level stubs for inline-typed Python modules. They would duplicate
  working annotations and add drift points.
- Check source-tree imports. That would not prove the wheel contains the typing
  contract.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| integration, gate | installed-wheel `mypy==2.3.0 --strict` and `stubtest` | Both package surfaces agree with their hand-written stubs |
| integration | installed package inventory | Each wheel contains its native stub and `py.typed` marker |
| regression | deliberate stub member and return-type mutations | The strict typing gate fails when a stub drifts |

The test gate is the backlog requirement that `mypy --strict` and `stubtest`
both pass.

## HLD impact

- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- WASM or PyO3 bindings. Retain both binding exclusions in workspace
  all-feature gates and check the existing WASM targets.
- New files. Obtain explicit approval for both native stubs, both markers, and
  both typing smoke programs.

## Hash harness

Expected unchanged. Typing metadata and tests do not affect OOXML or rendering.

## Implementation checklist

- [x] Add complete native stubs for `rdocx._rdocx` and `rpptx._rpptx`.
- [x] Add `py.typed` to both installed mixed packages.
- [x] Add one strict typing smoke program per package.
- [x] Prove installed-wheel inventory, strict mypy, and stubtest behavior.
- [x] Prove the gate fails under representative stub drift.

## Open questions

None. F-136 is an approved dependency, and the two native stubs, two markers,
and two typing smoke programs are explicitly approved.
