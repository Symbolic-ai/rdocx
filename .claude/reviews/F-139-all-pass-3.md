# F-139, all, pass 3

**Reviewed**: the complete 24-file working diff, 484 insertions and 1,224 deletions, against the approved plan, all cited HLD sections, progress notes, and passes 1 and 2
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, the Word isolation regression omits the authoritative workspace defaults

`Cargo.toml:54`
`Cargo.toml:67`
`Cargo.toml:68`
`crates/rdocx-wasm/src/lib.rs:271`
`crates/rdocx-wasm/src/lib.rs:272`
`.claude/plans/F-139-design.md:75`

The pass-2 remediation locks the feature lists and dependency declarations in
the `rdocx-layout`, `rdocx`, and `rdocx-wasm` member manifests, but it never
reads the root workspace manifest. Cargo cannot turn off defaults in an
inheriting member when the corresponding `[workspace.dependencies]` entry is
default-on. The root `oxml-layout`, `rdocx-layout`, and `rdocx` entries are
therefore part of the executable isolation contract, not incidental duplicate
configuration. Removing `default-features = false` from any of those root
entries can activate `oxml-layout/system-fonts` in the `rdocx-wasm` graph while
`word_native_defaults_and_wasm_isolation_are_manifest_contracts` still passes
all of its current assertions. The live feature trees are correct today, but
the approved regression and pass-2 remediation require that state to remain
mutation-sensitive. Include the authoritative workspace entries in the gate
and prove the relevant root-manifest mutations fail it.

## Smells

None.

## Nitpicks

None.

## Not found

- Pass-1 D1 behavior: the current native Word and presentation feature trees
  activate system fonts, while the current `rdocx-wasm` wasm32 tree excludes
  `oxml-layout/system-fonts`, `fontdb/fs`, `fontdb/fontconfig`, and `getrandom`.
  The presentation manifest regression passed independently.
- Pass-1 D2: the Node gate reflectively resolves generated `fromBytes` and
  `toDocxBytes`, crosses the `Uint8Array` boundary in both directions, and
  preserves the package. `wasm-pack test --node crates/rdocx-wasm` passed.
- Pass-2 D1 current-state wiring: the new member-manifest test passed, and its
  three recorded member-manifest mutations cover the assertions they change.
  The defect above is limited to the untested workspace-level inheritance.
- Correctness, contract, and OOXML preservation: the facade remains the sole
  document and package authority. No fresh loss of parts, relationships,
  content types, opaque XML, images, headers, or numbering was found.
- Panics and binding errors: no new panic reachable from untrusted exported
  input was found. Concrete facade failures remain string-valued `JsValue`s at
  the binding boundary.
- Structure and scope: no new trait, generic, module, source file, speculative
  feature, or browser PDF surface was introduced. The additional presentation
  manifests and tests match the revised approved feature-topology contract.
- Focused evidence: `cargo check --target wasm32-unknown-unknown -p
  rdocx-wasm`, the exact Word and presentation manifest tests, `cargo fmt --all
  --check`, `python3 scripts/prose_check.py`, and `git diff --check` passed.
