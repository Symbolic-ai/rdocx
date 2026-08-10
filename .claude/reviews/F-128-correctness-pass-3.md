# F-128, correctness, pass 3

**Reviewed**: working-tree diff from `575a8b6330609ec929dce4a15d8c02658be71eff`, 18 files, 1,463 changed lines
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, the PDF image-object probe still admits raster-invalid JPEG previews
`crates/rpptx/examples/render_deck.rs:504`
`crates/rpptx/tests/integration.rs:4694`
`crates/oxml-pdf/src/image.rs:36`
`crates/oxml-pdf/src/raster.rs:780`

The compatibility probe is equivalent to PDF writer admission, but it is not
equivalent to raster backend admission for JPEG. The PDF path accepts a JPEG
after `oxml_media::probe` finds dimensions and then passes the original bytes
through as a DCT stream. The raster path performs an additional full JPEG
decode and returns without drawing when that decode fails. A JPEG with a valid
SOF header but corrupt or absent scan data therefore makes `render_to_pdf`
emit `/Subtype /Image`, enters the scoped media map, and suppresses the labelled
chart placeholder, yet the deterministic PNG backend draws nothing.

The same probe also does not establish a strict declared-type match in the
opposite direction from the test. `decode_image` selects JPEG whenever byte
sniffing reports JPEG, so valid JPEG bytes declared as `image/png` pass the
probe and the resolver's allowed-content-type check. This contradicts the HLD
requirement that content-type-mismatched media remain unresolved. The
end-to-end gate at `crates/rpptx/tests/integration.rs:489` covers corrupt PNG
and PNG declared as JPEG, but it has no JPEG corruption or JPEG declared as PNG
case. Pass-1 D2 therefore remains open for the JPEG branch.

## Smells

None.

## Nitpicks

None.

## Not found

Pass-1 D1, D3 and D4 remain closed. Missing preview targets reach the labelled
fallback, the integration fixture contains a real cached picture, and slide,
layout and master chart relationships are resolved through their owning
scopes.

No added API or source change exists in `oxml-pdf`. The pass-2 remediation adds
no public API in another crate. Every crate whose feature implementation adds
public surface remains `publish = false`, so the public API of a published
crate risk rider does not apply.

No additional namespace-resolution, raw-byte preservation, schema child-order,
deterministic-ordering, panic-path, dependency-direction, group-lowering,
font-manager, or structural-simplicity defect was found. The focused fallback
integration test, focused resolver test, existing PDF JPEG admission test,
dependency tree check, and `git diff --check` passed during this review.
