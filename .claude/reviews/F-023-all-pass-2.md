# F-023, all, pass 2

**Reviewed**: remediated working-tree diff against claim commit `e0816ce`, 4 files, 303 lines added and 0 lines removed
**Verdict**: 2 defects, 1 smell, 0 nitpicks

## Defects

### D1, valid SVG internal subsets can still terminate document-type scanning early
`crates/oxml-media/src/lib.rs:170`

The document-type scanner counts every `[` and `]` outside a quoted string,
including brackets inside comments and processing instructions in the internal
subset. A valid internal subset comment containing `]` can reduce
`subset_depth` to zero, after which the comment's `>` is mistaken for the end
of the document type. The remaining declaration text does not start with
`<svg`, so the valid SVG is still rejected. Pass-one D3 is therefore only
partially resolved.

### D2, an invalid slash after the SVG element name produces a false positive
`crates/oxml-media/src/lib.rs:140`

The detector accepts any bytes beginning `<svg/`, although `/` is valid at
that position only as part of the empty-element terminator `/>`. For example,
`<svg/not-an-image` is classified as SVG. Since sniffing takes precedence over
the filename extension, arbitrary bytes with that prefix override even a
trustworthy non-SVG extension.

## Smells

### S1, the truncation regression does not exercise the new SVG prolog parser
`crates/oxml-media/src/lib.rs:273`

The SVG truncation case remains the direct `<svg` signature. None of the new
processing-instruction, comment, document-type, quote, or internal-subset paths
are entered by the truncation loop. Add the remediated prolog fixture to the
signature list so bounds-safety regressions in those paths cannot leave the
test green.

## Nitpicks

None.

## Not found

Pass-one D1 is resolved by the registered `image/emf` and `image/wmf` mappings.
Pass-one D2 is resolved for standard nonplaceable WMF headers. Pass-one S1 is
resolved by a direct extension-second `resolve` assertion. No additional
findings were found in JPEG mappings, panic safety outside the unexercised test
path, structure, dependency isolation, publication isolation, workspace
manifest wiring, or lockfile accuracy.
