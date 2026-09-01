# F-224, correctness, pass 1

**Reviewed**: working tree implementation across 5 files, 2,109 inserted lines and 2 deleted lines
**Verdict**: 3 defects, 0 smells, 1 nitpick

## Defects

### D1, the browser gate omits declared shared image input and exact run formatting
`crates/rpptx/tests/integration.rs:71`

The approved differential contract supplies the same source-built HTML, PNG,
and bundled font bytes to Chrome and the importer, then compares selected run
formatting exactly. The current source contains no image, passes an empty image
resource list at line 120, and only checks text at line 129. An image-resource
or run-formatting regression can therefore pass the named gate.

### D2, the preservation test does not prove byte preservation
`crates/rpptx/src/html.rs:1840`

The routed parser and serializer rider requires a byte-for-byte comparison of
an opaque template subtree. This test checks validation, relative child order,
and the presence of a presentation element, but it never captures template
bytes before conversion or compares them with the reopened output. A rewrite
of an unmodelled master, layout, or theme subtree can pass this test.

### D3, the all-limits test omits the DOM depth and CSS rule limits
`crates/rpptx/src/html.rs:1655`

The test claims to reject every declared resource limit, but its cases cover
neither `Limits::depth` nor `Limits::css_rules`. Regressions that stop enforcing
either bound can pass the named contract test.

## Smells

None.

## Nitpicks

- `crates/rpptx/src/html.rs:1101`, `emitted` is assigned `true` twice in the same branch.

## Not found

No additional findings in contract, panics, OOXML, tests, or structure. The
remaining `expect` calls assert locally established invariants or static
selectors. The implementation adds no trait, generic parameter, builder,
feature flag, crate, or integration test binary.
