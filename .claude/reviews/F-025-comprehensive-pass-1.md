# F-025, comprehensive, pass 1

**Reviewed**: working diff against
`c4d7a25919b7f70c013435126b00808a4c185383`, 1 file with 141 additions and 1
deletion
**Verdict**: 1 defect, 1 smell, 0 nitpicks

## Defects

### D1, Root-directory scans can allocate an occupied part name

`crates/oxml-media/src/lib.rs:56`
`crates/oxml-media/src/lib.rs:65`

`normalize_directory("/")` stores `/`, but splitting the valid root part name
`/image1.png` at its last slash produces an empty directory. The comparison
therefore rejects every existing root part. A namer scanned with `/`, `image`,
and `/image1.png` returns `/image1.png` again instead of `/image2.png`, which
breaks the collision-free public contract for a directory shape that the output
path handles explicitly.

## Smells

### S1, The regression gate does not distinguish normalized input or caller extensions

`crates/oxml-media/src/lib.rs:1071`
`crates/oxml-media/src/lib.rs:1094`

Every naming assertion passes the already canonical `/word/media` directory and
the `png` extension. The gate would remain green if directory normalization
were removed or if `next_part_name` ignored its extension argument and always
emitted PNG. Add distinct inputs for leading and trailing directory slashes and
a non-PNG caller extension. A root-directory case should pin the defect above.

## Nitpicks

None.

## Not found

No defect was found in positive decimal parsing for the covered media shape.
Empty, signed, zero, nonnumeric, wrong-stem, and wrong-directory names are
ignored.

No defect was found in maximum selection, occupied-value skipping, repeated
allocation, or integer-boundary handling. `usize::MAX` wraps through checked
addition to 1, zero is never selected, and the occupied set advances the loop
to a free positive suffix.

No defect was found in caller-extension rendering for valid extension strings.
The implementation uses the supplied extension verbatim.

No panic path was found beyond allocation failure. Matching uses checked
parsing and allocation advances with checked arithmetic.

No OOXML ordering, namespace, whitespace, or subtree-preservation concern
applies to this package-part naming change.

No structural violation was found. The public API adds the approved concrete
type in the existing module, and its iterator input has the two current
instantiations required by the design plan.

No rdocx consumer, dependency, HLD, sprint ledger, release configuration, or
publication change is present in the working diff. `oxml-media` remains at
version `0.0.0`, has no dependencies, and has publication disabled.
