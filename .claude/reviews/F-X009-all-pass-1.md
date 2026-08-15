# F-X009, all, pass 1

**Reviewed**: Working-tree implementation against `e65fffc`, 51 files, 768 insertions and 65 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, eleven crate examples show installation but not use

`crates/oxml-core/README.md:13`

The `Example` sections for `oxml-core`, `oxml-drawing`, `oxml-layout`,
`oxml-media`, `oxml-opc`, `oxml-pdf`, `oxml-py-support`, `rpptx-chart`,
`rpptx-layout`, `rpptx-oxml`, and `rpptx-render` contain only a Cargo dependency
declaration. They do not demonstrate a type, function, command, or consumer
operation from the crate. This does not satisfy the approved plan's requirement
for one concrete example suited to each package surface or the user's request
that every crate README explain use with examples. The inventory gate permits
this because it accepts any TOML fence as an example without package-specific
use assertions for these crates at `scripts/readme_doctests.py:298`.

Replace each dependency-only block with a small real usage example, retaining
the dependency declaration separately where it helps. Compile Rust examples
when the crate has a normal linkable library surface. For internal support
crates, show an honest repository-local consumer operation and label the
boundary explicitly.

## Smells

None.

## Nitpicks

None.

## Not found

No defects were found in the exact 26-package manifest inventory, distinct
README path enforcement, crates.io archive byte checks, existing stable crate
examples, CLI commands, Python examples, JavaScript examples, publication
labels, relationship descriptions, version requirements, HLD scope, prose,
formatting, or hash-harness evidence.
