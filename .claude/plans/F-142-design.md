# F-142, rpptx-wasm

**Status**: approved
**Sprint**: S35
**Size**: M
**Depends on**: F-116

## Problem

No `rpptx-wasm` crate exists. The settled `rpptx::Presentation` facade already
owns complete package round trips, but the presentation renderer's package to
`RenderInput` assembly remains embedded in a corpus example. A wrapper that
copied that logic would create the same mini-model drift F-139 removes.

The required small default profile also lacks a real feature boundary. `rpptx`
has no implemented `render` feature despite HLD15 describing one, and its
current graph unconditionally pulls rendering-adjacent layout and chart paths.

## Spec reference

- `docs/hld/03-architecture.md`, "Three families, one workspace" and dependency rules.
- `docs/hld/08-rendering-spec.md`, "The final facade boundary".
- `docs/hld/10-bindings-spec.md`, "WASM".
- `docs/hld/12-testing-strategy.md`, WASM and binding tests.
- `docs/hld/14-development-backlog.md`, "F-142, rpptx-wasm".
- `docs/hld/15-build-and-toolchain.md`, "Feature flags", "Packaging", and "CI job matrix".

## Approach

Create unpublished `crates/rpptx-wasm` with `WasmPresentation { inner:
rpptx::Presentation }`. The default API is constructor, `fromBytes`, `toBytes`,
`slideCount`, and `addSlide`. The default profile includes the existing bundled
template but excludes rendering. An opt-in `render` feature exposes only
`toPdf` and enables the real facade rendering stack.

Implement the `rpptx` render feature as a real boundary while preserving native
defaults. Move package-to-render-input assembly out of the corpus example into
the owning `rpptx` facade and expose the smallest honest deterministic PDF seam.
Update the example to call that seam so package interpretation exists once.

Add inline native and wasm-bindgen tests. Measure the final optimized `.wasm`
for the normal default profile after wasm-bindgen and wasm-opt, then deterministic
`gzip -n -9`. The exact gate is less than 1,000,000 compressed bytes. The render
profile is built and behavior-tested but is not subject to the small-profile
limit.

## Rejected alternatives

- Build a mini PresentationML model in WASM. The real facade already owns the
  package and mutation rules.
- Copy the corpus renderer into the binding. That creates a second package
  interpretation path.
- Always ship rendering dependencies. It makes the bounded default profile
  impossible.
- Add separate wrapper crates per profile or a runtime render flag. Cargo
  features are the existing compile-time consumer boundary.
- Add npm metadata or publication. F-146 owns that external surface.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| round-trip, gate | `default_profile_is_under_one_megabyte_and_round_trips_a_deck` | Optimized default `.wasm` is under 1,000,000 gzip bytes and preserves a nontrivial deck package |
| integration | `wasm_presentation_uses_the_real_facade_in_node` | Exported bytes, slide count, and mutation execute through the facade in Node |
| integration | `render_profile_returns_a_complete_pdf` | Opt-in render profile uses the real deterministic presentation renderer |
| regression | example and facade rendering parity | Moving assembly out of the example preserves existing render output and diagnostics |

Sensitivity enables render in defaults or adds padding and proves the exact size
gate fails, then restores byte-identically. A disposable minimal-package save
must fail the round-trip gate. Removing the facade dependency must fail the
source and dependency contract.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/08-rendering-spec.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- New crate, module, and files. Obtain explicit approval for
  `crates/rpptx-wasm/Cargo.toml` and `crates/rpptx-wasm/src/lib.rs`. Inline tests
  avoid another file or binary.
- Crate dependency graph. Read HLD03. Inspect default and render wasm32 trees,
  prove no forbidden shared-to-format edge, and prove the default omits the
  renderer, PDF backend, rasterizer, and host font discovery.
- WASM binding. Read HLD10. Run both profile target checks and Node suites with
  required PyO3 workspace exclusions.
- New feature and default behavior. Existing native rendering and the new WASM
  render profile are concrete consumers. Preserve native defaults and run all
  no-default gates.
- Bundled fonts and layout. Read HLD08 and HLD15. Use deterministic fonts,
  retain exact legal inventory, and make no incidental baseline change.
- Public API of published `rpptx`. State additive semver impact, run the exact
  publication dry run, and enforce archive size.
- Version and lock changes. Inspect workspace metadata, release-family counts,
  root lock, and both profile dependency trees. Do not publish or add npm
  metadata.

## Hash harness

Expected unchanged. The render refactor is behavior-preserving, and the new
binding does not participate in the 28 native sample outputs.

## Implementation checklist

- [ ] Create the approved unpublished wrapper crate with inline tests.
- [ ] Add the real `rpptx` render feature while preserving native defaults.
- [ ] Move package-to-render-input assembly into the facade and update the example.
- [ ] Implement default facade APIs and render-only `toPdf`.
- [ ] Gate exact optimized and deterministic compressed default size.
- [ ] Run both profile, Node, dependency, publication, and hash riders.

## Open questions

None. The two new crate paths with inline tests, decimal 1,000,000-byte gate,
template-bearing non-render default, bounded JavaScript API, render-only
`toPdf`, and exact wasm-pack and wasm-opt measurement toolchain are approved.
