# F-215, all, pass 6

**Reviewed**: complete working diff against the F-215 worker base, 10 files,
4,081 additions and 9 deletions, plus the approved design, cited HLD sections,
progress notes, passes 1 through 5, and every default microscope aspect
**Verdict**: 4 defects, 0 smells, 0 nitpicks

## Defects

### D1, timing id allocation can collide with schema-owned unsupported nodes

`crates/rpptx-oxml/src/timing.rs:527`

`crates/rpptx-oxml/src/timing.rs:1306`

The remediated allocator walks only typed `TimingNode` common values, so every
schema-owned timing kind that projects as `TimingNode::Unsupported` contributes
no id. For example, a direct `p:animClr` in `p:tnLst` can retain a valid
`p:cBhvr/p:cTn` with id 5 while the greatest typed id is 4. `add_media` then
authors ids 5 and 6, creating a duplicate time-node id and ambiguous timing
references. The new regressions correctly ignore `p:cTn` lookalikes inside
foreign raw wrappers, but allocation must still account for real common time
nodes owned by unsupported PresentationML timing kinds.

### D2, standard media replacement retargets unrelated raw picture references

`crates/rpptx-oxml/src/picture.rs:222`

`crates/rpptx-oxml/src/relmap.rs:75`

The standard attachment is discovered at the exact direct
`p:nvPr/a:audioFile` or `a:videoFile` slot, but replacement passes the complete
picture XML to the general relationship-id rewriter. If retained producer XML
inside that picture also references the old standard relationship id, every
matching relationships-namespace attribute is changed to the new payload. The
old relationship can then be pruned even though the unrelated raw subtree was
meant to keep it. Replacement must splice only the direct standard attachment
attribute, just as the Office extension replacement is structurally scoped.

### D3, supported-looking unknown media commands fail instead of staying explicit

`crates/rpptx-oxml/src/timing.rs:1513`

`crates/rpptx-oxml/src/timing.rs:1522`

The command projection has an `Other(String)` variant, but any lexical value
that starts with `playFrom(` or `seek(` and ends with `)` is forced through the
numeric parser. A producer command such as `seek(bookmark)` is a valid retained
command string but makes the complete timing parse fail rather than becoming
`MediaCommandKind::Other`. This violates the approved requirement that unknown
commands remain explicit and byte-preserved.

### D4, MIME casing bypasses known-container signature validation

`crates/oxml-media/src/lib.rs:90`

`crates/oxml-media/src/lib.rs:129`

`crates/rpptx/src/lib.rs:2856`

The safe MIME validator accepts uppercase token characters, as it should, but
the audio and video classifier matches known media types case-sensitively.
Media type and subtype names are case-insensitive, so `Audio/MPEG` denotes the
same supported type as `audio/mpeg`. With the current split, arbitrary bytes
supplied under `Audio/MPEG` produce `None` from the classifier and bypass the
facade's `Some(false)` rejection. The same input is later diagnosed as an
unsupported type instead of receiving the required MP3 signature check.

## Smells

None.

## Nitpicks

None.

## Pass-5 follow-up

- D1 is fixed. The parser records attributes from the direct,
  namespace-validated PresentationML `p:cTn` and derives loop and display only
  from that attribute set (`crates/rpptx-oxml/src/timing.rs:1402`). The
  adversarial foreign nested `cTn` test proves the direct values win
  (`crates/rpptx-oxml/tests/integration.rs:116`).
- D2 is fixed. Removal enters the direct PresentationML timing list, traverses
  only supported node ownership slots, and stops at unsupported nodes
  (`crates/rpptx-oxml/src/timing.rs:731` and
  `crates/rpptx-oxml/src/timing.rs:779`). The focused regression preserves the
  nested media-shaped raw payload byte for byte while deleting the direct owned
  audio and command (`crates/rpptx-oxml/tests/integration.rs:188`).
- D3 is fixed for foreign and foreign-wrapper time-node lookalikes. Allocation
  now derives its maximum from the typed timing tree
  (`crates/rpptx-oxml/src/timing.rs:515`), and both adversarial inputs remain
  raw while ids continue after the typed maximum
  (`crates/rpptx-oxml/tests/integration.rs:212` and
  `crates/rpptx-oxml/tests/integration.rs:237`). The schema-owned unsupported
  timing-node collision is recorded as pass-6 D1.

## Not found

- Correctness beyond D1 through D4: inverse trim offsets, dual-source linked
  precedence, Office source replacement, standard-only replacement, shared
  relationship retention, metadata-compatible deduplication, and candidate-only
  part pruning remain correctly implemented.
- Contract beyond the findings: additions require a validated poster, linked
  targets remain exact and external without fetching, unsupported embedded
  bytes remain packaged and extractable, and format-neutral media checks remain
  in `oxml-media`.
- Panics: the added production indexing and `expect` sites are dominated by
  checked slide indices, parser-established roots, or fixed local construction.
- OOXML beyond the findings: direct media settings are namespace-aware, timing
  removal respects modelled ownership slots, picture extension discovery uses
  the exact schema path and URI, new timing nodes follow schema order, and trim
  lexemes remain raw.
- Tests beyond the four gaps: the six focused pass-5 media regressions pass and
  directly cover foreign setting lookalikes, raw-wrapper removal preservation,
  typed id allocation, timing insertion order, and trigger ancestry. The
  pinned-deck gate continues to compare exact producer expectations.
- Structure and scope: no new trait, generic, feature, crate, module, file,
  dependency, forwarding wrapper, or builder was introduced. The public change
  remains within the approved pre-1.0 crates, and `rpptx-layout` only diagnoses
  the new retained timing variants.
