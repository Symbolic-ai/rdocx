# F-160, all, pass 3

**Reviewed**: complete staged and unstaged implementation diff against `HEAD`, 11 tracked files, 2,887 changed lines, with 2,190 additions and 697 deletions. The untracked pass-1 and pass-2 reviews were read as open history and excluded from the implementation count.
**Verdict**: 6 defects, 0 smells, 0 nitpicks

## Defects

### D1, one synthetic run still cannot preserve a multi-run cached result's formatting
`crates/rdocx-oxml/src/text.rs:1067`
`crates/rdocx-oxml/src/text.rs:1100`

The pass-2 repair copies one result run's properties onto the synthetic field
run. A valid result split across a bold run and an italic run is still collapsed
to one string with one property set, so layout, HTML, and Markdown apply the
first selected formatting to the complete display. The selector also chooses
the run after `separate` whenever the end marker is later, even when cached text
already follows `separate` in the same run. Both cases change existing display
formatting despite retaining the original runs for serialization.

### D2, dirty-only serialization duplicates field-result tabs and breaks
`crates/rdocx-oxml/src/text.rs:2280`
`crates/rdocx-oxml/src/text.rs:2345`

Complex parsing now encodes `w:tab` and `w:br` as control characters inside
`cached_result`. A source-preserving rewrite writes that complete string into
the first `w:t`, empties later text nodes, and retains the original `w:tab` and
`w:br` elements through the generic event branch. Merely changing `dirty`
therefore makes a reparse observe each tab or break twice. Page and column
break sentinels are also written as XML-forbidden U+000C and U+000B characters
inside `w:t`, so a valid source can serialize as invalid XML.

### D3, the missing-text fallback appends to a nested cached result instead of replacing it
`crates/rdocx-oxml/src/text.rs:2246`

When an outer field result consists entirely of a nested field, `wrote_result`
remains false and the repair inserts the new outer cache immediately before the
outer end marker. It preserves the nested field and its old result, so reopening
the output produces `old nested result` plus the new cache rather than the new
`cached_result` alone. The focused regression uses an unmodelled producer node,
which does not contribute display text and therefore misses this original
pass-2 trigger.

### D4, projection inside a hyperlink loses the hyperlink's raw child boundaries
`crates/rdocx-oxml/src/text.rs:1136`
`crates/rdocx-oxml/src/text.rs:2961`

The projected field source collects paragraph-level `extra_xml` only, while
unmodelled children of an explicit `w:hyperlink` live in
`HyperlinkSpan::extra_xml`. After five field runs collapse to one, those
relative boundaries are not remapped. A raw child originally between early
field runs moves after the complete field, while children at later old
boundaries are never emitted because the collapsed hyperlink has only
boundaries zero and one. The new hyperlink test contains runs only and cannot
detect this preservation failure.

### D5, cache writers do not maintain significant result whitespace
`crates/rdocx-oxml/src/text.rs:2442`
`crates/rdocx-oxml/src/text.rs:2642`

Replacing an existing result copies its old attributes without adding
`xml:space="preserve"` when the new cache starts or ends with whitespace.
Inserted and canonical result writers likewise create bare `w:t`. A cache
mutation from `old` to ` value ` therefore serializes the requested string
without the OOXML whitespace marker Word needs to retain its boundary spaces.

### D6, a recursive structured edit on a simple-source field is silently dropped
`crates/rdocx-oxml/src/text.rs:2510`
`crates/rdocx-oxml/src/text.rs:2534`

The public grammar permits `FieldArgument::Nested` in every `FieldInstruction`,
but canonical instruction text includes text arguments only. A parsed simple
field whose public arguments or switch argument is changed to a nested field
keeps its private simple source form, then serializes only the incomplete raw
attribute. The nested operand disappears instead of selecting complex form or
rejecting an unrepresentable edit.

## Smells

None.

## Nitpicks

None.

## Not found

Panics and structure produced no additional findings. The pass-2 exporter,
basic formatting, control parsing, dirty-marker clearing, missing direct text,
canonical quoting, hyperlink-run discovery, and nested-malformation cases all
have focused repairs. The focused checks passed: 16 `rdocx-oxml` field tests,
the `rdocx-html` complex cached-display test, and the `rdocx-layout`
unsupported complex-field display test. `git diff --check` also passed. No new
trait, generic parameter, crate, module, or feature flag was introduced.
