# F-105, all, pass 1

**Reviewed**: working tree against `27130e7`, five files, 142 changed text
lines plus the 31,022-byte `default.pptx` asset
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Not found

- Correctness: `Presentation::new()` uses the existing fallible parser path at
  `crates/rpptx/src/lib.rs:91`, and the template graph assertions cover the
  specified dimensions, master, layouts, themes, auxiliary parts, notes
  master, and zero slides at `crates/rpptx/tests/integration.rs:207`.
- Contract: the default-enabled feature at `crates/rpptx/Cargo.toml:14` gates
  only the bundled constructor and asset. The crate remains unpublished and no
  dependency was added.
- Panics: production code adds no panic path. Test-only `expect` and `panic`
  calls provide fixture context and do not process caller input.
- OOXML: the asset is loaded through `OpcPackage` and the existing model
  parser, then deterministically serialized and reopened at
  `crates/rpptx/tests/integration.rs:192`. Native PowerPoint 16.104 opened the
  emitted deck without repair.
- Tests: reverting the constructor fails
  `new_presentation_uses_the_bundled_zero_slide_template`, while removing or
  changing the asset fails `bundled_template_has_the_documented_part_graph`.
  The opt-out build and Cargo package-list riders also passed.
- Structure: the only new file is the story-mandated crate-local asset. No
  trait, generic, wrapper, source module, dependency, or speculative API was
  introduced.
