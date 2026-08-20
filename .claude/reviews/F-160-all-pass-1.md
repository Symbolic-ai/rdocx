# F-160, all, pass 1

**Reviewed**: uncommitted working diff, 4 files, 1,335 changed lines, with 1,087 additions and 248 deletions
**Verdict**: 10 defects, 0 smells, 0 nitpicks

## Defects

### D1, unsupported complex fields lose their cached display during layout
`crates/rdocx-layout/src/engine.rs:922`

The complex-field projection replaces all source result runs with one synthetic
`RunContent::Field`. Layout then skips every field name except PAGE, NUMPAGES,
REF, and PAGEREF. A valid complex DATE, TIME, AUTHOR, or other field that used
to render its ordinary stored-result `w:t` now renders nothing. This changes
existing document output before F-161 is available and violates the stored
display and unchanged-harness contract.

### D2, a same-run separate and end marker panics on untrusted XML
`crates/rdocx-oxml/src/text.rs:925`

The projection records marker positions only as run indices. If one `w:r`
contains the field instruction, a `separate` marker, cached text, and the `end`
marker, `separate == run_index` and this expression slices
`runs[run_index + 1..run_index]`. Rust panics because the range starts after it
ends. Multiple run-content children in one run are legal, so opening such a
document must not panic.

### D3, changing a parsed field drops its unmodelled XML and run formatting
`crates/rdocx-oxml/src/text.rs:1906`

Raw source is reused only while every public field value is unchanged. Changing
the cached result or dirty flag takes the canonical writer path, which replaces
the complete source with bare generated runs. For example, the aliased simple
field in the test at line 3445 contains `q:custom` and run content, but changing
its result silently drops that subtree. Complex fields likewise lose the
producer run partition, result-run properties, proofing markers, and raw
neighbours absorbed into their source blob. F-162 necessarily changes the
cached result and dirty flag, so this violates the preservation contract on the
main planned mutation path.

### D4, complex-field discovery interprets markers inside unmodelled subtrees
`crates/rdocx-oxml/src/text.rs:810`

The event scan walks every descendant of the captured run and does not restrict
`w:fldChar` or `w:instrText` to direct children of `w:r`. A marker inside a
producer extension or an `mc:AlternateContent` branch is therefore treated as
part of the paragraph field stack. A later marker can make the parser invent a
typed field and absorb the unmodelled subtree into it, contrary to the rule to
parse only modelled content and preserve the rest verbatim.

### D5, namespace shadows leak across sibling run children
`crates/rdocx-oxml/src/text.rs:813`

`prefixes` is replaced for every start and empty event but never restored on an
end event. If an unmodelled child temporarily rebinds the Word prefix, a later
sibling `w:instrText` or `w:fldChar` is tested against the child's stale scope
instead of the run scope. The valid complex field remains untyped or is parsed
with missing instruction text. This fails the required namespace-shadow and
prefix-tolerant read behavior.

### D6, quoted backslash-leading operands are misclassified as switches
`crates/rdocx-oxml/src/text.rs:2866`

The lexer removes quotation boundaries before the parser decides whether a
token is a switch. For example, `INCLUDETEXT "\\\\server\\share\\file.docx"`
produces a token beginning with a backslash and this branch records it as a
switch rather than the required path argument. Empty quoted operands are also
dropped because the lexer emits a token only when its accumulated text is
nonempty. The AST therefore does not retain quoted arguments as required.

### D7, the argument-taking switch table omits approved field switches
`crates/rdocx-oxml/src/text.rs:2896`

Only `*`, `#`, `@`, `r`, `s`, and `d` can own an argument. The approved S49
subset also includes MERGEFIELD `\\b` and `\\f` arguments, while INCLUDETEXT
`\\c` has a converter argument that must be recognisable before F-161 can choose
its cached fallback. Those values are instead placed in the field's ordinary
argument list, so consumers cannot distinguish the field operand from the
switch operand.

### D8, malformed simple fields are exposed as typed fields
`crates/rdocx-oxml/src/text.rs:747`

A missing `w:instr` is replaced with an empty string and the function always
returns `Some(Field)`. Thus `<w:fldSimple><w:r><w:t>cached</w:t></w:r></w:fldSimple>`
becomes a typed field with an empty name instead of remaining opaque raw
content. The approved malformed fallback says malformed fields must be
preserved without inventing a typed projection.

### D9, public instruction edits serialize from different sources by field form
`crates/rdocx-oxml/src/text.rs:1931`

For simple fields the writer uses only `instruction.raw`, so changing the public
`name`, `arguments`, or `switches` marks the field changed but writes the old
instruction. For complex fields the writer starts from `name` and the parsed
vectors at line 1945, so changing the public `raw` string is ignored. The same
public AST therefore has two incompatible mutation semantics, and valid edits
can be silently absent from serialized XML.

### D10, the corpus gate does not verify the promised switch AST
`crates/rdocx-oxml/src/text.rs:3305`

For most corpus rows the test checks only that a switch name exists. It does not
assert which token became that switch's argument, and it omits the common
argument-taking MERGEFIELD and INCLUDETEXT switches. The current incorrect
switch table therefore passes the named unit gate. The gate also has no legal
same-run marker case, which leaves the panic in D2 unexercised.

## Smells

None.

## Nitpicks

None.

## Not found

Structure produced no additional findings. The diff adds no trait, generic
parameter, crate, module, file, or feature flag. The rdocx-html exhaustive-match
adaptations are scoped and consistent with the new enum shape.
