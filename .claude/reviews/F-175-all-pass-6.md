# F-175, all, pass 6

**Reviewed**: the complete remediated working tree on `work/f-175-codex`, 7
tracked feature files plus the approved new
`crates/rdocx/src/redaction.rs`, 2,205 additions and 4 deletions
**Verdict**: 3 defects, 0 smells, 0 nitpicks

## Defects

### D1, table-property-exception revision authors remain recoverable

`crates/rdocx/src/redaction.rs:730`

The revision-author allowlist delegates to `is_word_revision_container`, but
that set still omits `w:tblPrExChange`. This schema revision element carries
`w:author`. A valid table row containing
`<w:tblPrExChange w:author="sec&#114;et">` therefore receives no attribute
edit, and the raw scan does not see the entity-decoded literal. The candidate
commits with the sensitive author intact. The complete author-bearing revision
allowlist must include table property exception changes as well as the range
starts added in this pass.

### D2, legacy visible run blocks still allow false cross-node matches

`crates/rdocx/src/redaction.rs:673`

The expanded Word boundary list now includes note reference marks, but it
still omits other schema-defined visible run content: `w:annotationRef`,
`w:pgNum`, and the `w:dayShort`, `w:monthShort`, `w:yearShort`, `w:dayLong`,
`w:monthLong`, and `w:yearLong` date blocks. For example, `sec`, a
self-closing `w:pgNum`, and `ret` are joined as `secret` and removed even
though the rendered run contains a page number between the fragments. These
run elements need the same expanded-name boundary treatment as the newly
added footnote and endnote marks.

### D3, valid UTF-16 sensitive XML cannot be redacted

`crates/rdocx/src/redaction.rs:302`

The rewriter rejects every UTF-16BE or UTF-16LE sensitive XML part before
parsing it. UTF-16 is a valid OPC XML encoding, so a relationship-resolved
UTF-16 header, comment, chart, worksheet, or shared-string part containing the
selector makes `redact_text` fail rather than remove the value. This is not one
of the contract's malformed-input cases, and it prevents the operation from
covering valid documents even though the postcondition scanner already checks
UTF-16LE residual bytes.

## Smells

None.

## Nitpicks

None.

## Not found

- Pass-5 D1: `w:delInstrText` participates in the deleted-instruction flow,
  and expanded-name `w:fldSimple/@w:instr` is redacted as a sensitive
  attribute. The focused fixture covers entity-decoded simple-field
  instructions and split deleted instructions.
- Pass-5 D2: XML declarations are accepted only as the first event, and a
  document type is accepted at most once before the sole root. Misplaced or
  repeated declarations and document types fail closed.
- Pass-5 D3: `w:footnoteRef` and `w:endnoteRef` now form boundaries for both
  empty and paired producer spellings. D2 lists other remaining visible run
  blocks.
- Pass-5 D4 for the cited range starts: move-from, move-to, and all four custom
  XML range-start names redact `w:author` without entering the flow-boundary
  set. D1 identifies the remaining non-range author owner.
- Pass-5 D5: `w:rt` and `w:rubyBase` each start and end an independent flow, so
  phonetic guide text cannot join base text.
- Pass-5 D6: focused fixtures bind noncanonical prefixes to sensitive Word,
  WordprocessingDrawing, ChartML, and DrawingML namespaces.
- Pass-5 D7: the public regression performs a second redaction with selector
  `12.5` and requires positive ChartML-cache and embedded-workbook counts plus
  a raw package scan.
- Accepted and rejected revision projections: zero additional findings.
  Hidden descendants remain transparent, mutually exclusive branches stay
  isolated, and the alternating passes reach a shared fixed point.
- Raw text, CDATA, and attribute rewriting: zero findings beyond D3. Matching
  attributes patch exact raw value spans, changed CDATA becomes escaped text,
  and the output validator requires one root and bound expanded names.
- OPC resolution and package bounds: zero findings. Relationship targets
  resolve from their owning source, external workbooks and missing internal
  parts fail closed, and outer and nested packages use explicit limits.
- UTF-8 and UTF-16LE residual scanning: zero findings in the scanner itself.
  Every inflated outer and nested entry is checked in both forms. D3 concerns
  valid UTF-16 sensitive XML before that postcondition.
- Atomicity: zero findings. All mutation remains staged through serialization,
  scan, bounded reopen, and validation. Failure preserves package bytes, typed
  state, and all four cache or engine identities.
- Package preservation: zero findings. The regression compares every untouched
  outer part byte for byte, together with complete relationships and content
  types.
- Panic and error handling: zero findings. Production positions, slicing,
  indexing, and arithmetic are guarded or saturating.
- Public API isolation: zero findings. The additive method and report remain
  native to `rdocx`, with no Python, WASM, or CLI binding expansion.
- Structure: zero findings. The sole new file is explicitly approved, and no
  new trait, generic parameter, crate, feature flag, forwarding wrapper, or
  dependency-family edge appears.
- HLD and hash scope: zero findings. Exactly the four plan-listed HLD files
  change, and no sample or hash baseline file changes.
