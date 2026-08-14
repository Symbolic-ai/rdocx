# F-X009, README coverage for every workspace crate

**Status**: completed
**Sprint**: S39
**Size**: L
**Depends on**: none

## Problem

Cargo metadata resolved a README for only eight of the 26 workspace packages,
and only seven manifests declared the path explicitly.
Crates.io renders the seven stable package READMEs correctly at 0.5.0, but the
remaining internal, incubating, binding, and WASM package boundaries lack the
same explanation and examples.

## Spec reference

- `docs/hld/12-testing-strategy.md`, "README example correctness".
- `docs/hld/14-development-backlog.md`, "F-X009, README coverage for every
  workspace crate".
- `docs/hld/15-build-and-toolchain.md`, "Package inventory" and "The two
  release families".

## Approach

Add one crate-local README for each of the 18 workspace members that currently
has no README metadata. Retain the root README as the `rdocx` package README,
retain and audit the seven existing crate-local documents, and improve the
minimal `oxml-sml` document. Add explicit `package.readme` metadata to all 19
manifests that lacked it, including the automatically discovered `oxml-sml`
file. Every README must state purpose, direct-use
guidance, neighbouring package relationships, publication status, and one
concrete example suited to its Rust, CLI, Python, or JavaScript surface.

Extend the existing README runner rather than adding another script. It will
derive the exact 26-package inventory from Cargo metadata, require one declared
README per package, validate common sections and package-specific snippets,
compile Rust examples where the package can be linked as a normal library, and
verify every publishable archive contains exactly one README.

## Rejected alternatives

- One generic README copied across packages. Package boundaries need distinct
  ownership and usage guidance.
- Claim that every example can be a Rust doctest. PyO3, WASM, and CLI packages
  need examples in their real consumer surface.
- Add another documentation script. The existing runner already owns README
  correctness and inventory.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | exact 26-package README inventory | Every workspace package declares one existing README |
| integration | `python3 scripts/readme_doctests.py` | Rust examples compile and non-Rust examples satisfy exact package and command contracts |
| integration | publishable archive inventory | Every crates.io package carries exactly one intended README |
| regression | missing manifest README mutation | Removing one README declaration fails the exact inventory gate |

The **test gate** is `python3 scripts/readme_doctests.py` over the exact 26
workspace packages.

## HLD impact

- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- New files. The user explicitly authorized README coverage for all 26
  workspace crates. Add only the 18 missing crate-local documents.
- WASM and PyO3 bindings. Keep publication flags unchanged, retain required
  workspace test exclusions, and run both WASM target checks.
- Release metadata and package inventory. Inspect every manifest README diff,
  run the exact patched workspace dry run, and confirm no version, tag, or
  publication allowlist changes.

## Hash harness

Expected to be unchanged. Documentation and manifest README metadata must not
change generated outputs.

## Implementation checklist

- [x] Record the exact 26-package baseline and real missing-README red gate.
- [x] Add the 18 authorized README files and 19 manifest declarations.
- [x] Audit and strengthen all eight existing README sources.
- [x] Extend the existing runner to enforce exact inventory and example quality.
- [x] Compile applicable Rust examples and validate CLI, Python, and JavaScript examples.
- [x] Verify every publishable archive contains exactly one README.
- [x] Run full verification, package riders, and the unchanged hash harness.
- [x] Obtain a clean independent microscope review.

## Open questions

None. The user explicitly requested README coverage for all 26 workspace
packages.
