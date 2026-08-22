# F-166, all, pass 2

**Reviewed**: Uncommitted working diff, 4 files and 1,166 changed lines, with
1,119 additions and 47 deletions
**Verdict**: 4 defects, 0 smells, 0 nitpicks

## Defects

### D1, Generated bookmark names can capture unrelated existing references

`crates/rdocx/src/field.rs:381`

`crates/rdocx/src/field.rs:400`

The allocator reserves existing bookmark names but not existing hyperlink
anchors or `REF` and `PAGEREF` targets. If the source has bookmark `Target` and
an intentionally unresolved `REF MailMerge1`, remapping the second record
renames `Target` to `MailMerge1`. The unrelated field is not changed because it
is not keyed by `Target`, but it now resolves to the generated bookmark. The
same capture can redirect an existing hyperlink anchor. D1 from pass 1 is only
partly resolved because numeric and declared bookmark collisions are remapped,
while generated names can still change valid source semantics.

### D2, Nested story projection loses ancestor namespace bindings

`crates/rdocx/src/field.rs:1624`

`crates/rdocx/src/field.rs:1637`

Each nested header or footer paragraph is parsed in a synthetic document that
copies only namespaces declared on the story root. A valid paragraph can inherit
its WordprocessingML prefix from an intermediate table or content control. The
extracted paragraph then has an unbound prefix, projection fails, and the caller
silently skips that entire story. Two records with different values therefore
retain equal raw bytes and section mode accepts the record-varying non-body
field. D2 from pass 1 remains for valid locally scoped namespace aliases.

### D3, Footnote round trips still discard unmodelled note content

`crates/rdocx/src/document.rs:715`

`crates/rdocx-oxml/src/footnotes.rs:206`

The relationship target is now respected, but every candidate still serializes
the complete typed `CT_Footnotes` projection. That projection writes only direct
paragraphs. A valid footnote table or other unmodelled child is dropped from
each separate output even when it contains no merge field. Section mode also
returns the lossy first candidate when non-body outputs compare equal. The
approved contract requires every unrelated package part to remain
byte-preserved, so D3 from pass 1 is not fully resolved by correcting the part
path alone.

### D4, Nested story support changes the ordinary field APIs

`crates/rdocx/src/field.rs:87`

`crates/rdocx/src/field.rs:159`

The nested header and footer projection runs for both values of the mail-merge
policy flag. As a result, existing `evaluate_fields` and `update_fields` calls
now discover and rewrite fields inside header tables and content controls,
including non-merge field families. The approved plan adds two opt-in mail merge
operations and a merge-local missing-value policy. It does not authorize a
behavioral expansion of the existing public field traversal. The nested scan
must be confined to the merge operation or approved as a separate behavior
change.

## Smells

None. S1 from pass 1 is resolved by using one shared `clone_for_staging`
implementation from both mail merge and template rendering. No new trait,
generic, module, forwarding wrapper, or duplicate staging helper remains.

## Nitpicks

None.

## Not found

S2 from pass 1 is resolved. The round-trip test now matches both producer raw
subtrees exactly rather than counting only surviving tokens.

Panics: the empty-record guard still makes candidate indexing and final-index
subtraction safe. The remapping scanners use checked allocation and bounded
slice searches, with no new untrusted-input unwrap found.

OOXML child order and fixed output prefixes: no additional finding beyond D2
and D3. Section properties continue through the existing schema-ordered
serializers, and the final body `sectPr` remains schema-final.
