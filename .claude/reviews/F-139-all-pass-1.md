# F-139, all, pass 1

**Reviewed**: the complete 19-file working diff from the F-139 claim state, 399 insertions and 1,217 deletions, plus the approved plan, progress notes, and HLD 03, 10, 12, 13, 14, and 15
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, the feature refactor disables system fonts for the native presentation renderer

`Cargo.toml:54`
`crates/rpptx-render/Cargo.toml:21`
`crates/rpptx-render/src/lib.rs:207`
`crates/oxml-layout/src/font.rs:149`
`docs/hld/08-rendering-spec.md:445`

The workspace now makes every inherited `oxml-layout` dependency default-off,
but only the Word facade family is given an explicit `system-fonts` forwarding
path. `rpptx-render` still inherits `oxml-layout.workspace = true`, so a normal
default `rpptx` build no longer activates `oxml-layout/system-fonts`,
`fontdb/fs`, or `fontdb/fontconfig`. Its normal `layout_presentation` path still
constructs `FontManager::new()`, whose host discovery is compiled only behind
that missing feature. Presentations that request an installed host font now
fall back to bundled or embedded fonts even though HLD08 requires the normal
entry point to retain system-font discovery. This also contradicts the approved
plan's native-defaults-unchanged assertion. The focused `cargo tree -p rpptx -e
features` observation contained no system-font feature, while the WASM tree was
correctly host-font-free. The native feature assertion covered `rdocx` only and
therefore missed this regression.

### D2, the Node gate bypasses the exported JavaScript binding

`crates/rdocx-wasm/src/lib.rs:214`
`crates/rdocx-wasm/src/lib.rs:217`
`.claude/plans/F-139-design.md:68`

The `wasm-bindgen-test` does execute as WebAssembly under Node, but its body
calls the Rust associated function and Rust method directly. It never obtains
the generated JavaScript class, invokes `fromBytes` or `toDocxBytes` by their
JavaScript names, or crosses the `Uint8Array` and `Vec<u8>` conversion boundary.
The test therefore remains green if either `js_name` attribute is removed or
renamed, despite stable JavaScript names being an explicit F-139 contract and
the plan describing this test as execution of the actual exported wrapper. The
native gate already proves the internal facade delegation. The Node gate must
add independent coverage of the generated JavaScript surface to prove the
binding contract it claims.

## Smells

None.

## Nitpicks

None.

## Not found

- Correctness and OOXML preservation: the facade-backed save retains the tested
  part inventory, content types, relationships, opaque bytes, image, header,
  and numbering semantics. The named native regression passed.
- Panics: no new panic on untrusted package input was found in the exported
  paths. Binding errors are converted to string-valued `JsValue`s.
- Structure and layering: `WasmDocument` owns the concrete facade directly,
  and no new trait, generic, forwarding-only wrapper, module, or source file was
  introduced.
- WASM isolation and dependency hygiene: the wasm32 feature tree contains no
  `fontdb/fs`, `fontconfig`, or `getrandom` path. The wasm32 check and the Node
  suite passed independently.
- Additive facade behavior: `document_text_preserves_body_and_table_order`
  passed, as did the `rdocx`, `rdocx-layout`, and `oxml-layout` no-default test
  suites.
- Formatting and artifacts: `cargo fmt --all --check` and `git diff --check`
  passed. The obsolete nested lock deletion and exact test dependency are in
  scope, and no unexpected generated artifact was present.
