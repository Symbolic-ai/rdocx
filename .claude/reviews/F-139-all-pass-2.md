# F-139, all, pass 2

**Reviewed**: the complete 24-file working diff, 460 insertions and 1,224 deletions, against the approved plan, cited HLD sections, progress notes, and pass 1
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, the approved Word feature-isolation regression is missing

`.claude/plans/F-139-design.md:75`
`crates/rdocx/Cargo.toml:19`
`crates/rdocx-layout/Cargo.toml:18`
`crates/rdocx-wasm/Cargo.toml:22`
`crates/rpptx-render/src/lib.rs:3502`

The approved test plan requires regression assertions that direct native Word
defaults include system fonts and that the WASM graph excludes host font
features. The implementation wires those contracts correctly in the three Word
manifests, but the only new manifest regression reads and asserts the
presentation manifests. Removing `system-fonts` from either Word default list
does not break the CLI or Python consumers because they opt in explicitly, and
the native and Node round-trip tests do not render. They therefore remain green
while direct `rdocx` or `rdocx-layout` defaults silently lose host discovery.
The recorded `cargo tree` inspection proves the current working tree, but it
does not provide the regression test promised by the plan. Add a focused
assertion for the active Word default-forwarding chain and the defaults-off
`rdocx-wasm` edge, with sensitivity to each relevant manifest mutation.

## Smells

None.

## Nitpicks

None.

## Not found

- Pass-1 D1 remediation: native `rpptx` and `rpptx-render` now forward
  `system-fonts`, the binding and development consumers opt in, and the focused
  manifest regression passed. The native feature tree contains
  `oxml-layout/system-fonts`, `fontdb/fs`, and `fontdb/fontconfig`.
- Pass-1 D2 remediation: the Node test obtains the generated class constructor,
  resolves `fromBytes` and `toDocxBytes` by their JavaScript names, passes a
  `Uint8Array` in, and asserts a `Uint8Array` out. `wasm-pack test --node
  crates/rdocx-wasm` passed its one test with wasm-pack 0.13.1.
- Correctness and OOXML preservation: the exact native R-class gate passed and
  retains part inventory, content types, package and part relationships, opaque
  bytes, header text, image presence, numbering, and reopenability.
- Contract and scope: `WasmDocument` owns the concrete `rdocx::Document`, keeps
  the approved JavaScript names, adds only the planned `Document::text` facade
  surface, and does not add browser PDF scope.
- Panics: no new panic reachable from untrusted exported input was found.
  Binding failures map to string-valued `JsValue`s. Test-only assertions and
  fixture construction contain the new `unwrap` and `expect` calls.
- OOXML: no new parser or serializer was introduced. The binding delegates to
  the existing package-preserving facade and the gate covers the complete
  constructed package graph.
- Structure: no new trait, generic parameter, module, source file, forwarding
  wrapper, or speculative bundled-font feature was introduced. The obsolete
  nested lock deletion matches workspace ownership.
- Dependency hygiene: the inspected wasm32 normal feature tree contains
  `rdocx` and excludes `fontdb/fs`, `fontdb/fontconfig`, and `getrandom`. No
  forbidden `oxml-*` outward dependency was found.
- HLD discipline: the six plan-listed HLD files describe the facade wrapper,
  local Node gate, feature forwarding, and resolved carried defect. No unlisted
  HLD file was edited.
- Formatting and artifacts: `git diff --check` passed. Focused review builds
  used isolated target directories after the shared worker target lock was not
  writable, and no tracked generated artifact appeared.
