# F-178, correctness, pass 1

**Reviewed**: the F-178 working diff against `7bfa238`, 9 implementation and
test files with 2,294 additions and 8 deletions, plus the approved plan update
**Verdict**: 5 defects, 0 smells, 0 nitpicks

## Defects

### D1, path input can exceed the declared allocation bound
`crates/rdocx/src/html.rs:238`

`open_html` trusts one metadata length check and then calls unbounded
`read_to_end`. A special file with a zero metadata length, or a regular file
that grows after the metadata call, can allocate and read more than 64 MiB.
The path constructor therefore does not enforce the approved input bound before
allocation.

### D2, preformatted newlines are serialized as text instead of Word breaks
`crates/rdocx/src/html.rs:1191`

The `pre` branch sends newline characters to `Paragraph::add_run`, which emits
one `w:t`. Word line breaks require `w:br`. The source text accessor can still
echo the newline, so the current unit assertion passes even though the saved
DOCX does not preserve the preformatted line structure as required.

### D3, a table inside a list item is silently discarded
`crates/rdocx/src/html.rs:1025`

`collect_inline_children` skips both nested lists and tables. `list_models`
later revisits only nested lists at `crates/rdocx/src/html.rs:707`. A valid
table inside an `li` therefore loses all visible table content without either
projection or the stable diagnostic required for a safely skipped visible
construct.

### D4, the DOM node ceiling is applied only after comment-node allocation
`crates/rdocx/src/html.rs:313`

The construction preflight deliberately excludes markup beginning with `!`,
so repeated HTML comments do not increase `estimated_nodes`. `parse_document`
or `parse_fragment` then allocates every comment node before `validate_dom`
rejects the input. A sub-64-MiB input can consequently construct millions of
nodes despite the 100,000-node construction bound.

### D5, stylesheet relation matching is incorrectly case-sensitive
`crates/rdocx/src/html.rs:367`

HTML link relation tokens are ASCII case-insensitive. A head resource such as
`<link rel="STYLESHEET" href="external.css">` bypasses the external stylesheet
diagnostic even though it represents the same dropped resource as the lowercase
form.

## Smells

None.

## Nitpicks

None.

## Not found

- Contract: no unapproved public binding, network, file, module, or dependency
  expansion was found beyond the approved native facade API and private module.
- Panics: static-selector and checked-node expectations are internal invariants.
  No additional production panic or unchecked arithmetic path was found.
- OOXML: table properties, grids, rows, cells, paragraphs, grid spans, vertical
  merges, and numbering use the existing typed model in schema order.
- Tests: the six named gates fail if the additive API is reverted. The missing
  assertions correspond to D1 through D5.
- Structure: the approved private module has one concrete owner and introduces
  no trait, generic parameter, feature flag, dynamic dispatch, forwarding-only
  wrapper, crate, or second document model.
