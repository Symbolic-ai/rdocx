# F-X018, correctness, pass 1

**Reviewed**: the uncommitted working tree. Twelve call sites across six files
in `rdocx-oxml`, three new tests, and one spec bullet.
**Verdict**: 0 defects, 0 smells, 2 nice-to-have

## Defects

None.

The change is mechanical and its risk is uniformity: a site left strict would
still fail the open. All twelve moved, and the document-level regression covers
eight of the nine enumerations end to end, failing against a single reverted
site with the offending value named.

## Smells

None.

## Nice-to-have

### N1, `StyleType` is not covered by the document regression
`crates/rdocx-oxml/src/styles.rs:22`

The story names nine enumerations. The document-level regression exercises
eight, because `StyleType` is reached from `styles.xml` rather than
`document.xml`, so a `CT_Document::from_xml` fixture cannot carry it.

`StyleType::from_str` is still covered by the strictness unit test, and its
call site was reviewed by hand. Adding a styles-part fixture would close the
gap properly and is a small, separable piece of work rather than something to
graft onto this regression.

### N2, an unmodelled value is silently lost on save
`crates/rdocx-oxml/src/properties.rs` and five other files

A document carrying an unmodelled value now opens, and the field is `None`, so
the serialiser writes nothing and the value is gone on save. Before this story
the document could not be opened at all, so nothing regresses.

It is the accepted cost of the chosen rule, recorded in the design plan, in the
spec bullet and in AS_BUILT. Preserving the raw string would need the same
`raw_xml` machinery that unmodelled *elements* already use, extended to
attributes, which is a much larger story.

## Not found

Checked and produced nothing:

- **correctness**. Two site shapes, handled differently on purpose.
  `Option`-typed fields become `None`, which means "not specified" and lets the
  style chain supply the value. The three `borders.rs` locals keep the default
  they were initialised with, since `None` is not available there.
- **uniformity**. No `?` remains on any of the nine parsers at a property
  parsing site. Verified by grep, not by memory.
- **strictness preserved**. `the_parsers_still_reject_an_unknown_value` pins
  that each `from_str` still returns `Err`, so tolerance lives at the call site
  and a future caller that wants strictness has it.
- **panics**. `.ok()` and `.unwrap_or(val)` cannot panic. No indexing added.
- **ooxml**. Attribute values only. No element name, namespace, prefix or child
  ordering touched, and the serialiser is unchanged.
- **structure**. No new type, trait, module or feature flag.
- **contract**. Matches the plan. The plan cited the wrong spec file for the
  tolerance rule and has been corrected in place rather than quietly fixed.

## Hash harness

**Unchanged, 28 of 28.** Expected: every corpus document already opens, so none
carries an unmodelled value. A delta would have meant a value the model does
enumerate started parsing differently.
