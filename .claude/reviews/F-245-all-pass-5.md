# F-245, all, pass 5

**Reviewed**: working-tree implementation diff excluding review records, 17 files, 2,347 additions and 40 deletions
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, custom licensing attributes are not declared ignorable

`crates/rdocx-oxml/src/font_table.rs:233`

The writer declares the private `rdocx` namespace and emits private attributes
on `w:embedRegular` and its siblings, but it does not declare the Markup
Compatibility namespace or add `rdocx` to `mc:Ignorable`. `CT_FontRel` models
only the standard relationship id, font key, and subset flag. A schema-aware
consumer that does not understand the private namespace therefore sees
unrecognized attributes rather than legal ignorable extension markup. The root
must carry a safe MCE declaration and an ignorable token that preserves any
producer-owned tokens already present.

### D2, exact license identity admits XML-normalized whitespace

`crates/rdocx/src/document.rs:8676`

The public boundary rejects an empty or oversized identity but accepts literal
tab, carriage return, and line feed characters. XML attribute normalization
changes those characters to spaces when the saved font table is reopened, so
the returned identity is not exact. The boundary must reject whitespace that
cannot round-trip unchanged. Reserved XML syntax is already escaped by the
string attribute conversion and is not part of this finding.

## Smells

None.

## Nitpicks

None.

## Not found

Pass 4 package-wide exclusivity, raw relationship retention, and sibling event
preservation are correct and covered by focused regressions. Correctness,
contract, panic safety, OOXML sequence order, namespace replay, test strength,
and repository structure were checked. No further issue was found in GUID and
enumeration validation, font-byte obfuscation, layout alias selection, cache
invalidation, or the pinned Word and LibreOffice comparison.
