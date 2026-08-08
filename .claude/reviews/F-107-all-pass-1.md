# F-107, all aspects, pass 1

**Reviewed**: working tree against
`9495ef9aa4752df968fe76852fdc3e1dd25698ed`, 5 files, 481 insertions and 6
deletions
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, the native acceptance test never invokes PowerPoint

`crates/rpptx/src/lib.rs:922`

The test named `three_added_slides_open_in_powerpoint_without_repair` only
serialises and reopens the deck through `Presentation::from_bytes`. It passes
without Microsoft PowerPoint and cannot detect the repair prompt that the
approved acceptance test and backlog gate require. Move this gate to an
explicit ignored native test, pin the application version and build, and make
that test open the generated three-slide deck in PowerPoint.

### D2, the relationship test does not prove a relocated layout target

`crates/rpptx/src/lib.rs:879`

The assertion checks only that the target lacks a leading slash. A hardcoded
`../slideLayouts/slideLayout1.xml` target passes against the bundled template,
so the test does not prove the approved requirement to compute the target when
a template stores layouts elsewhere. Exercise a layout record at a relocated
part name and assert that resolving the emitted relative target reaches that
exact part.

## Smells

None.

## Nitpicks

None.

## Not found

No additional correctness, contract, panic, OOXML preservation, schema order,
test, or structure findings were found. The implementation stages all fallible
work before package mutation, preserves placeholder type and index, excludes
the three latent placeholder types, emits the required slide child order, and
introduces no unnecessary trait, generic, wrapper, module, crate, or feature
flag.
