# F-217, all, pass 9

**Reviewed**: uncommitted working tree implementation diff, 10 files, 4,826 changed lines, with 4,811 additions and 15 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D26, inherited default section namespaces are lost during dirty rewriting
`crates/rpptx-oxml/src/presentation.rs:1589`

The namespace carry-forward helper unconditionally rejects the empty prefix.
A valid section extension can therefore declare the P14 namespace as the
default on its `p:ext` parent and use unprefixed `sectionLst`, `section`,
`sldIdLst`, and `sldId` descendants. Initial parsing recognizes those elements
through the complete live scope, but dirty rewriting rebuilds the scope without
the inherited default binding. It then classifies every original section as raw
content and writes the caller's replacement beside it. Facade `set_sections`
reopens that well-formed result successfully and commits both the old and new
sections, so the method returns success without replacing the section
collection. This violates the prefix-tolerant parser contract and the public
replacement semantics.

## Smells

None.

## Nitpicks

None.

## Prior finding status

- D1 is remediated. Open rejects shared comment-part ownership, and commented
  slide duplication is refused atomically.
- D2 is remediated. Section discovery requires the exact extension URI and a
  direct `p:ext` parent.
- D3 is remediated. A self-closing slide extension list expands in place and
  retains its opening bytes.
- D4 is remediated. Comment and reply status values are validated on parse and
  write.
- D5 is remediated. All three model-prefix candidates, inherited bindings,
  descendant uses, and shell-attribute uses either retain their meaning or fail
  closed.
- D6 is remediated. Direct text, CDATA, comments, processing instructions, and
  other captured events survive at comment model boundaries.
- D7 is remediated. Unsupported comment-shell attributes retain their lexical
  source bytes.
- D8 is remediated. An actually self-closing presentation extension list
  expands in place.
- D9 is remediated. Section, slide-id-list, and slide-id attributes and direct
  raw events retain their lexical bytes during supported dirty writes.
- D10 is remediated. Public author, comment, and reply identifiers and
  timestamps are revalidated during serialization.
- D11 is remediated. The facade round-trip gate checks reply movement plus
  notes-master and handout-master header-footer values after reopen.
- D12 is remediated for prefixed section lists. Generated children carry a
  model namespace binding and reparse. D26 is the remaining inherited default
  namespace case.
- D13 is remediated. Section insertion recognizes a self-closing extension-list
  root from its terminal lexical form.
- D14 is remediated. Sidecar-free alias-only section lists clear without
  retaining an invalid empty typed shell.
- D15 is remediated. Author boundaries reconcile original author ids, so an
  appended author remains before the original trailing raw sidecar.
- D16 is remediated. The named facade round-trip gate asserts both saved
  collaboration content-type overrides.
- D17 is remediated. Inherited and local fixed-prefix bindings used by
  descendants or shell attributes survive, and exhausted candidates fail
  clearly.
- D18 is remediated. Slide-list raw boundaries reconcile against original
  slide-id identity for removal and reorder.
- D19 is remediated for inherited nonempty aliases used by direct section
  children. D26 is the distinct default-namespace gap in that carry-forward
  path.
- D20 is remediated. Text-body root namespace declarations and unsupported
  root-attribute lexemes survive unchanged serialization and safe dirty writes.
- D21 is remediated. An unlinked conventional comment part carrying the
  matching modern-comment MIME blocks mutation atomically.
- D22 is remediated. A valid owned `comment1.xml` no longer triggers the
  collision guard, and a second commented slide allocates `comment2.xml`.
- D23 is remediated. Clearing a section list with direct raw payload now fails
  before mutation instead of retaining an invalid empty typed list.
- D24 is remediated. Unchanged text bodies keep their raw bytes, while a dirty
  body that depends on a producer-owned inherited `a` prefix fails closed.
- D25 is remediated. Slide removal serializes and reopens its staged graph
  before publishing the section mutation.
- The pass-5 nitpick remains remediated. Author-list parsing computes and
  stores the raw attribute sidecar once.

## Not found

Correctness, contract, panics, OOXML, tests, and structure were all checked.
Smells produced zero findings. Nitpicks produced zero findings. Panics produced
zero findings. Structure produced zero findings. The only new module was
explicitly approved, and the diff adds no trait, public generic, crate,
feature, builder, forwarding wrapper, or production dependency. The four exact
pass-8 focused regressions pass. The full `rpptx-oxml` integration binary passes
109 tests. The `rpptx` integration binary passes 92 tests with 7 ignored after
the two sandbox-only oracle failures are rerun with a writable cache. No
additional defects were found in comment-part allocation, fixed-prefix comment
shells and text bodies, collaboration graph ownership, relationship resolution,
content-type creation, section clearing, slide-removal staging, notes-master or
handout-master schema order, legacy-comment opacity, or header-footer mutation.
