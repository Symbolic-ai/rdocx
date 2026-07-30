# F-025, comprehensive, pass 2

**Reviewed**: working implementation diff against
`c4d7a25919b7f70c013435126b00808a4c185383`, 1 file with 153 additions and 1
deletion
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Not found

Pass-one D1 is resolved at `crates/oxml-media/src/lib.rs:65`. A root part now
maps its empty split prefix back to `/`, so scanning `/image1.png` allocates
`/image2.webp` rather than reusing the occupied name. The regression at
`crates/oxml-media/src/lib.rs:1116` pins that behavior.

Pass-one S1 is resolved at `crates/oxml-media/src/lib.rs:1113`. The test now
proves leading and trailing directory slash normalization and caller-controlled
non-PNG extensions. The root case separately proves normalized root matching.

No correctness or contract defect was found in directory and stem matching,
positive decimal suffix parsing, malformed near-match rejection, caller
extension output, maximum selection, occupied-value skipping, or repeated
allocation. The boundary tests at `crates/oxml-media/src/lib.rs:1121` and
`crates/oxml-media/src/lib.rs:1131` prove safe wrap after `usize::MAX` without
selecting zero or reusing an occupied positive suffix.

No panic path was found beyond allocation failure. Suffix parsing is checked,
and allocator advancement uses checked addition at
`crates/oxml-media/src/lib.rs:41`.

No OOXML ordering, namespace, whitespace, or subtree-preservation concern
applies to this package-part naming change.

No test-gate gap remains. The four approved F-005 regressions are present, and
the normalization, root-directory, and caller-extension assertions exercise
the remediated paths. All 22 `oxml-media` tests and strict crate clippy pass.

No structural violation was found. The approved concrete type remains in the
existing module, introduces no trait or wrapper, and the planned iterator input
has current slice and array iterator instantiations.

No rdocx consumer, dependency, HLD, sprint ledger, release configuration, or
publication change is present in the implementation diff. `oxml-media` remains
at version `0.0.0`, has no dependencies, and has publication disabled.
