# F-181, integration, pass 1

**Reviewed**: the staged F-181 squash integration against `ebee604`, 23 files,
7,377 insertions and 12 deletions, plus the combined F-180, F-181, and F-182
native exports and reconciled HLD contracts
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, the dependency policy still says base64 is absent from ordinary and binding graphs

`docs/hld/15-build-and-toolchain.md:545`

The dependency policy groups `base64` with the default-off digital-signature
dependencies and says that group does not enter ordinary, Python, WASM, or CLI
graphs. The integrated SVG contract makes `base64` an unconditional direct
`rdocx` runtime dependency at `crates/rdocx/Cargo.toml:26`, and both `rdocx-py`
and `rdocx-wasm` therefore receive it transitively. The later SVG paragraph at
`docs/hld/15-build-and-toolchain.md:565` records the direct consumer but does
not correct the earlier claim. The reconciled HLD does not yet describe the
current combined dependency graph faithfully.

## Smells

None.

## Nitpicks

None.

## Verification

- `cargo test --locked -p rdocx --lib epub::tests`: 33 passed, 1 ignored.
- `cargo test --locked -p rdocx --lib odt::tests`: 36 passed.
- `cargo test --locked -p rdocx --lib svg::tests`: 19 passed.
- The public ODT integration gate, public SVG integration gate, and SVG facade
  regression each passed.
- The EPUBCheck CI workflow regression passed. The external ignored EPUBCheck
  invocation was not repeated in this integration review.
- `cargo fmt --all --check`, `git diff --cached --check`, and
  `python3 scripts/prose_check.py --staged` passed.

## Not found

No fresh correctness, contract, panic, OOXML or EPUB packaging, test-gate,
public-export, or structural defect was found in the staged code integration.
The `rdocx` facade exports all three approved native result types without
changing Python, WASM, CLI, Presentation, or public `oxml-pdf` APIs.
`docs/hld/10-bindings-spec.md` retains the approved ODT and SVG contracts and
adds the completed EPUB contract without semantic conflict. No smells or
nitpicks were found.
