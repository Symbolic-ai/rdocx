# F-215, all, pass 2

**Reviewed**: complete working diff against the F-215 worker base, 10 files,
3,159 additions and 9 deletions, plus the approved design, cited HLD sections,
progress notes, pass-1 findings, and every default microscope aspect
**Verdict**: 3 defects, 0 smells, 0 nitpicks

## Defects

### D1, valid trim offsets are rejected as an ordered range

`crates/rpptx-oxml/src/picture.rs:995`

`crates/rpptx-oxml/src/timing.rs:521`

`crates/rpptx/src/lib.rs:2805`

The `st` and `end` values of `p14:trim` are independent amounts removed from
the beginning and end of the media. They are not endpoints on one ordered
timeline. A valid clip can therefore remove 875 milliseconds from the start
and 125 milliseconds from the end. Picture parsing and both mutation
validation paths reject that valid input because they require `st <= end`.
Only the media duration can determine whether the combined offsets leave a
valid playable interval, and that duration is not available here. Existing
tests use only smaller-start, larger-end examples, so they do not exercise the
inverse valid case.

### D2, valid linked precedence is rejected when both media attributes exist

`crates/rpptx-oxml/src/picture.rs:1028`

`crates/rpptx-oxml/src/picture.rs:1034`

The Office 2010 media extension permits both `r:embed` and `r:link`, with the
linked relationship taking precedence. `p14_media_source` instead rejects the
element whenever both attributes are present. A valid producer picture using
the declared precedence therefore fails parsing rather than projecting the
linked target while retaining the original XML.

### D3, the public timing insertion API silently discards trim arguments

`crates/rpptx-oxml/src/timing.rs:351`

`crates/rpptx-oxml/src/timing.rs:365`

`crates/rpptx-oxml/src/timing.rs:560`

`CT_Timing::add_media` publicly accepts and validates `trim_start_ms` and
`trim_end_ms`, then calls `authored_media_xml` without passing either value.
The method returns success even though neither argument can affect the written
timing XML. The facade happens to write trim through `CT_Picture`, but direct
users of the published `rpptx-oxml` API receive a successful result that has
dropped their requested settings. The timing API must not claim inputs it
cannot serialize.

## Smells

None.

## Nitpicks

None.

## Pass-1 follow-up

- D1 moved trim parsing and authorship to `p14:media/p14:trim`, retains the raw
  lexical XML, and exposes rounded checked integer values. The incorrect range
  rule remains as pass-2 D1.
- D2 now derives playback triggers through timing ancestry.
- D3 and D4 now insert only into a namespace-validated PresentationML timing
  list and place a new list before later schema children.
- D5 and D6 now scope replacement to the selected picture and preserve shared
  relationship records still referenced by retained slide XML.
- D7 now requires bytes, extension, and content type to agree before explicit
  media part reuse.
- D8 now uses token-checked MIME validation from `oxml-media`.
- D9 now asserts the pinned decks' expected media bytes, relationship types and
  targets, content types, poster ownership, playback settings, and retained
  metadata.
- S1 moved format-neutral MIME and container-signature classification into the
  dependency-free `oxml-media` crate and documented the boundary.

## Not found

- Correctness beyond the three defects: targeted replacement, extraction,
  removal, part pruning, and duplicate-slide remapping have regression
  coverage for shared relationships and package-wide byte reuse.
- Contract beyond the findings: additions require a valid poster, linked media
  retains exact external targets without fetching, and unsupported codec bytes
  remain packaged, extractable, and diagnosable.
- Panics: added production indexing and `expect` sites remain dominated by
  validated slide indices, fixed local construction, or parser-established
  roots.
- OOXML beyond the findings: prefix aliases are resolved by namespace URI,
  authored timing children follow schema order, and unmodelled subtrees retain
  their raw serialization source.
- Tests beyond the noted gaps: the corpus gate now compares against explicit
  producer expectations rather than self-comparing pre-save and post-save
  projections.
- Structure: no new trait, generic, feature, crate, module, file, dependency,
  forwarding wrapper, or builder was introduced. The `rpptx-layout` change is
  limited to exhaustive diagnostics for the existing media timing variants.
