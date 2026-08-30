# F-217, all, pass 8

**Reviewed**: uncommitted working tree implementation diff, 10 files, 4,653 changed lines, with 4,640 additions and 13 deletions
**Verdict**: 4 defects, 0 smells, 0 nitpicks

## Defects

### D22, an existing valid comment part blocks comments on every later slide
`crates/rpptx/src/lib.rs:1090`

The pass-7 collision guard rejects any package containing the conventional
`comment1.xml` part before the numeric allocator runs. After the API adds a
comment to slide 1, that valid owned part exists. Adding a different valid
comment to slide 2 therefore returns `CollaborationPartCollision` instead of
allocating `comment2.xml`. The MIME-matching unlinked collision from D21 is now
blocked, but the broad check also prevents the ordinary one-comment-part-per-
slide graph required by the facade.

### D23, clearing sections fails when the section-list shell has raw payload
`crates/rpptx-oxml/src/presentation.rs:1076`

When `set_sections(Vec::new())` clears a list that has a direct preserved
attribute, child, comment, or processing instruction, this branch retains the
`sectionLst` shell and writes it with zero typed sections. The same parser then
rejects that result because `sectionLst` requires a typed section at
`crates/rpptx-oxml/src/presentation.rs:817`. The facade candidate reopen returns
an error, so a valid caller request cannot clear sections whenever the approved
raw sidecar contract has something to preserve. Only sidecar-free clearing is
covered by the current tests.

### D24, a producer-owned DrawingML prefix inside a typed text body changes meaning
`crates/rpptx-oxml/src/comments.rs:1214`

If a comment-list ancestor owns `xmlns:a="urn:producer"`, a typed text body can
use another alias for its modelled DrawingML children and contain a preserved
raw `<a:producer/>` child. The list correctly retains the producer binding, but
`write_text_body` unconditionally installs the DrawingML URI on the text-body
shell when it sees that inherited shadow. The raw child bytes survive under the
new URI, so their expanded name changes. Reopen still succeeds because
`CT_TextBody` dispatches these children by local name. D20's root-attribute
sidecar is fixed, but the byte-preserved descendants under that root remain
namespace-unsafe.

### D25, slide removal publishes a section state that can no longer serialize
`crates/rpptx/src/lib.rs:1321`

`remove_slide` clones and mutates the presentation, including removing the
producer slide id from every section, then assigns the clone without serializing
or reopening it. A package whose dirty section writer must reject, such as the
covered valid input where all three fixed model-prefix candidates are
producer-owned, therefore returns success from `remove_slide` and replaces the
live facade. Its next `to_bytes` fails with `no unshadowed section model prefix
is available`. The approved contract requires typed-root changes to commit only
after serialization and reopen succeed.

## Smells

None.

## Nitpicks

None.

## Prior finding status

- D1 is remediated. Open rejects shared comment-part ownership, and commented-slide duplication is refused atomically.
- D2 is remediated. Section discovery requires the exact extension URI and direct `p:ext` parent.
- D3 is remediated. A self-closing slide extension list expands in place and retains its opening bytes.
- D4 is remediated. Comment and reply status values are validated on parse and write.
- D5 is remediated for all three model prefixes, inherited bindings, descendant uses, and shell-attribute uses. D24 is a distinct nested text-body descendant failure.
- D6 is remediated. Direct text, CDATA, comments, processing instructions, and other raw events survive at comment model boundaries.
- D7 is remediated. Ordinary unsupported comment-shell attributes retain their lexical source bytes.
- D8 is remediated. An actually self-closing presentation extension list expands in place.
- D9 is remediated. Section, slide-id-list, and slide-id attributes and direct raw events retain their lexical bytes during supported dirty writes.
- D10 is remediated. Public author, comment, and reply identifiers and timestamps are revalidated during serialization.
- D11 is remediated. The facade round-trip gate checks reply movement plus notes and handout header-footer values after reopen.
- D12 is remediated. Generated children under a dirty aliased section list carry a model namespace binding and reparse.
- D13 is remediated. Section insertion recognizes a self-closing extension-list root from its terminal lexical form.
- D14 is remediated for sidecar-free alias-only section lists. D23 is the remaining direct-sidecar clearing case.
- D15 is remediated. Author boundaries reconcile original author ids, so an appended author remains before the original trailing raw sidecar.
- D16 is remediated. The named facade round-trip gate asserts both saved collaboration content-type overrides.
- D17 is remediated. Inherited and local bindings used by descendants or shell attributes survive, and exhausted fixed candidates fail clearly.
- D18 is remediated. Slide-list raw boundaries reconcile against original slide-id identity for removal and reorder.
- D19 is remediated. Dirty section replacement and clearing retain inherited alternate aliases used by original direct section children.
- D20 is remediated for text-body root namespace declarations and unsupported root-attribute lexemes. D24 is a distinct inherited fixed-prefix failure for a raw descendant inside that typed root.
- D21 is remediated for an unlinked conventional part carrying the matching modern-comment MIME. D22 is the regression where the same broad guard rejects a valid linked part owned by another slide.
- The pass-5 nitpick remains remediated. Author-list parsing computes and stores the raw attribute sidecar once.

## Not found

Correctness, contract, panics, OOXML, tests, and structure were all checked.
Panics produced zero findings. Structure produced zero findings. Smells produced
zero findings. Nitpicks produced zero findings. The only new module was
explicitly approved, and the diff adds no trait, public generic, crate, feature,
builder, forwarding wrapper, or production dependency. The four exact pass-7
focused regressions pass. The full `rpptx-oxml` integration binary passes 107
tests. The `rpptx` integration binary passes 90 tests with 7 ignored after the
two sandbox-only oracle failures are rerun with a writable cache. No additional
defects were found in fixed-prefix shell-attribute handling, comment graph open
validation, relationship resolution, content-type creation, notes-master or
handout-master schema order, legacy-comment opacity, or header-footer mutation.
