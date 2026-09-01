# F-220, correctness, pass 11

**Reviewed**: pass 10 remediation against carried head `594bf7c`, one changed
production and test file with 133 additions. The review also rechecked the full
F-220 contract, pass 10 findings, pinned-resource profiles, required external
differential, and prior budget and cycle safeguards.
**Verdict**: 0 defects, 0 smells, 0 nitpicks. Ready for completion.

## Defects

None. Count: 0.

## Smells

None. Count: 0.

## Nitpicks

None. Count: 0.

## Pass 10 remediation verification

- **Exact occurrence and order**: the non-cycle path now completes the existing
  readable field, owner, topology, and cardinality checks with an exact semantic
  profile at `crates/rpptx/src/diagram.rs:336`. The profile includes every
  instruction kind, variable identity, sorted attribute pair, ordered child,
  and child occurrence at `crates/rpptx/src/diagram.rs:493`. Same-kind reorder
  and replacement therefore cannot preserve the accepted profile.
- **Portable profile encoding**: every field, attribute count, and child count
  uses fixed-width `u64` little-endian length encoding at
  `crates/rpptx/src/diagram.rs:495`. Native and WASM targets therefore compute
  the same digest. Attribute order is normalized without normalizing values or
  child order.
- **Fail-closed identity boundary**: each of the five readable authentic
  identities has one distinct semantic profile at
  `crates/rpptx/src/diagram.rs:534`. A mismatch returns a stable family-named
  diagnostic before evaluation at `crates/rpptx/src/diagram.rs:554`. The
  separate exact raw-byte, identity, instruction, and three-node cycle profile
  remains unchanged.
- **Discriminating mutations**: the installed-resource regression now swaps two
  same-kind constraints, substitutes the equally family-valid right-aligned
  parameter tuple at the selected list occurrence, duplicates a valid
  constraint without changing count, adds a family-valid style to the wrong
  owner, and swaps two accepted relationship conditions at
  `crates/rpptx/src/diagram.rs:5144`. Every mutation reaches a fail-closed
  diagnostic. The unchanged pinned resources remain executable.

## Validation observed

The required six-family PowerPoint 16.104 common-source differential passed
with unchanged geometry and masked SSIM metrics. The sensitivity gate rejected
the 1.01-point displacement and the decorative mutation at masked SSIM
`0.555302336`. Full `rpptx` passed 33 unit tests and 189 integration tests with
the documented ignores. `rpptx-oxml` passed 15 unit tests and 155 integration
tests. Scoped Clippy, WASM, no-default layout, dependency tree, formatting,
prose, skill sync, diff hygiene, and the unchanged 49 of 49 hash harness passed.

## Not found

- **Correctness**: no surviving same-kind reorder, occurrence substitution,
  duplicate semantic replacement, condition swap, or wrong-owner attribute
  path was found.
- **Contract**: the change remains private, validates the exact six approved
  resources, and adds no public API, dependency, feature, module, file, trait,
  generic, package asset, or second SmartArt model.
- **Panics and bounds**: the existing depth and total-work checks run before the
  semantic profile. Fixed-width length conversion is safe for supported 32-bit
  and 64-bit targets. No indexing, arithmetic, allocation, or recursion bypass
  was introduced.
- **OOXML and preservation**: the profile consumes the already namespace-aware
  typed projection. It neither reparses nor serializes XML, and untouched source
  bytes remain owned by the original raw model.
- **Tests and structure**: the new helper is private and directly used by the
  one supported validation path. The mutation cases fail against the pass 10
  source and pass only with the remediation.
