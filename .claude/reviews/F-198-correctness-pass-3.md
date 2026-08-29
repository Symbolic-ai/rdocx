# F-198, correctness, pass 3

**Reviewed**: reconstructed working-tree diff against
`bc478f8a06d37268d06cd41598037df1d91b0611`, 17 tracked implementation, HLD,
and baseline files with 799 additions and 24 deletions, plus 2 restored
historical review records
**Verdict**: 4 defects, 0 smells, 0 nitpicks

## Defects

### D1, a self-closing settings root drops the authored toggle
`crates/rdocx-oxml/src/settings.rs:365`

`rewrite_automatic_hyphenation` handles only a start event at depth zero. A
valid parsed settings part written as `<w:settings/>` reaches the generic event
writer at line 420, so `set_automatic_hyphenation(true)` updates the typed value
but leaves `source_xml` without `w:autoHyphenation`. `to_xml` then returns those
unchanged source bytes and silently loses the requested authoring mutation.

### D2, creating settings can overwrite an unrelated existing package part
`crates/rdocx/src/document.rs:3725`

When an opened package has no settings relationship, the new setter always
claims `/word/settings.xml` without checking whether that part name is already
occupied. The next flush writes the authored settings bytes to that path at
`crates/rdocx/src/document.rs:2064` and changes its content type, destroying an
unrelated producer part. New settings authoring needs a collision-free target
while retaining an existing relationship-resolved target unchanged.

### D3, a retained malformed language child can move ahead of a modeled one
`crates/rdocx-oxml/src/properties.rs:1127`

A nonempty malformed `w:lang` is deliberately retained as raw XML, but its
position is always recorded as occurrence zero. If it follows a valid modeled
`w:lang`, the positioned-raw writer compares that zero at
`crates/rdocx-oxml/src/properties.rs:1661` and emits the malformed child before
the modeled child. This violates the raw child order contract for the exact
case the fallback is meant to preserve. The retained position must use the
current language occurrence.

### D4, the required low-level Rust source impact is absent from the HLD
`docs/hld/10-bindings-spec.md:93`

The added HLD paragraph documents only the additive facade methods. F-198 also
adds required public fields to `CT_RPr` and to `LayoutInput`, including
`crates/rdocx-layout/src/input.rs:114`, so existing external full struct
literals no longer compile. The approved risk routing explicitly requires this
pre-1.0 source impact to be stated. The current HLD therefore understates the
public compatibility contract.

## Smells

None.

## Nitpicks

None.

## Not found

No additional findings were found for line-breaking behavior, effective
language and paragraph suppression, generated source spans, fields, tables,
notes, drawing reflow, panic and error paths, schema child order, namespace
classification, deterministic fonts, golden and pinned Writer evidence, the
declared five-key hash delta, F-X062 restart behavior, F-X063 retained-font
matching, F-X066 run preservation, dependency direction, package archives, or
structural simplicity. The current package proof isolates registry
`oxml-layout@0.7.0`, while the immutable historical carrier regression retains
`rdocx-layout@0.10.1` resolving `oxml-layout@0.6.0`.
