# F-065, all aspects, pass 1

**Reviewed**: working-tree diff, 3 files, 1,416 inserted lines and 10 removed lines
**Verdict**: 1 defect, 1 smell, 0 nitpicks

## Defects

### D1, modelled string attributes are not XML-decoded

`crates/oxml-drawing/src/theme.rs:1277`

`required_attr` uses `get_attr`, which returns the encoded attribute bytes rather
than a decoded XML value. The root theme name at line 474 uses the same helper
directly. A valid typeface such as `A&amp;B` is stored as the literal text
`A&amp;B`, then the writer escapes its ampersand again. The first write produces
`A&amp;amp;B`, so parse, write, and reparse are not structurally equal. Decode
modelled attributes with the same normalized quick-xml path already used for
raw attributes, and add a regression assertion.

## Smells

### S1, private writers introduce one-way generic parameters

`crates/oxml-drawing/src/theme.rs:558`

The new private writer methods and helpers are generic over `W: Write`, but this
feature instantiates them only with `Writer<Vec<u8>>` from `to_xml`. The
repository structural rule requires two current instantiations before adding a
generic parameter. Use the concrete writer type throughout this private writer
chain. No public nested-writer API was requested by the design contract.

## Nitpicks

None.

## Not found

- Correctness: no other wrong logic, boundary error, or malformed-input panic.
- Contract: the public model, Office default, and oracle pin stay within F-065.
- Panics: the built-in static theme invariant is the only production panic.
- OOXML: required and duplicate children, schema order, fixed prefixes, and raw
  subtree positions are otherwise handled correctly.
- Tests: all named plan gates exist, and the pinned PowerPoint no-repair open was
  observed separately from the ignored-by-default suite.
- Structure: no new trait, crate, wrapper, feature flag, or dependency was added.
