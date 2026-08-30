# F-217, all, pass 3

**Reviewed**: uncommitted working tree implementation diff, 10 files, 3,487 changed lines, with 3,474 additions and 13 deletions
**Verdict**: 3 defects, 0 smells, 0 nitpicks

## Defects

### D5, inherited fixed-prefix shadows still change raw comment meaning
`crates/rpptx-oxml/src/comments.rs:989`

The serializer now permits safe self-contained `p188` and `a` shadows, but it
unconditionally removes those namespace declarations from every modelled
comment shell. A raw child can validly depend on a declaration owned by its
modelled parent, for example an aliased `q:cm` with
`xmlns:p188="urn:producer"` and a preserved `<p188:producer/>` child. Rewriting
the comment removes the owner declaration and emits the unchanged child under
the writer's modern-comment namespace instead. Reopen succeeds, so the facade's
atomic candidate check does not detect the semantic corruption.

### D9, nested section sidecars are still not preserved verbatim
`crates/rpptx-oxml/src/presentation.rs:829`

The pass 2 remediation preserves the `p14:sectionLst` shell and its direct raw
events, but each typed `p14:section` still decodes unsupported attributes into
string pairs and drops direct comments, processing instructions, text, and
CDATA through the wildcard arm at
`crates/rpptx-oxml/src/presentation.rs:870`. `write_section` then recreates the
attributes at `crates/rpptx-oxml/src/presentation.rs:1182`. A section carrying
`x:flag='a&#x20;b'` or a producer comment therefore loses lexical bytes or the
entire event after `set_sections` or section membership removal. The same
problem applies inside the typed slide-id list.

### D12, dirty aliased section lists emit unbound fixed-prefix children
`crates/rpptx-oxml/src/presentation.rs:1158`

Dirty serialization replays the original `sectionLst` opening tag byte for
byte, then `write_section` always emits `p14:section` at
`crates/rpptx-oxml/src/presentation.rs:1175`. A valid input using only an alias,
such as `<q:sectionLst xmlns:q=".../powerpoint/2010/main">`, provides no `p14`
binding for those new children. The low-level writer produces namespace-invalid
XML, while facade `set_sections` fails its reopen and cannot mutate that valid
prefix-tolerant input.

## Smells

None.

## Nitpicks

None.

## Not found

D1 is remediated. Open rejects a comment part shared by multiple slides, and
commented-slide duplication fails without changing the presentation.

D2 is remediated. Section discovery requires the exact extension URI and a
direct `p:ext` parent.

D3 is remediated. Self-closing slide extension lists expand in place while
retaining their original start bytes.

D4 is remediated. Comment and reply status parsing and writing enforce the
declared enumeration.

D6 is remediated for comment, reply, author, and list boundaries. Direct raw
events are retained there.

D7 is remediated for comment attributes. Their source lexemes survive dirty
serialization.

D8 is remediated. A self-closing presentation extension list expands in place
when sections are added.

D10 is remediated. Public author, comment, and reply identifiers and timestamps
are revalidated during serialization.

D11 is remediated. The facade round-trip gate now checks notes and handout
header-footer values plus final-index reply movement after reopen.

Correctness, contract, panics, OOXML, tests, and structure were all checked.
Panics, smells, and structure produced no additional findings. The only new
module was explicitly approved, and the diff adds no trait, generic, crate,
feature, builder, forwarding wrapper, or production dependency.
