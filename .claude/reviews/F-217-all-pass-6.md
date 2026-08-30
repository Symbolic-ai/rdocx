# F-217, all, pass 6

**Reviewed**: uncommitted working tree implementation diff, 10 files, 4,283 changed lines, with 4,270 additions and 13 deletions
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D5, the final comment fallback prefix is still dropped or reused as model content
`crates/rpptx-oxml/src/comments.rs:1189`
`crates/rpptx-oxml/src/comments.rs:1100`

The dependent-shadow scanner restores producer-owned declarations for `p188`
and `p188m`, but it does not handle `p188model`, even though
`model_attributes` removes that declaration. An aliased comment-list or model
shell with `xmlns:p188model="urn:producer"` and a preserved
`<p188model:producer/>` descendant therefore loses the declaration. The writer
either leaves the raw descendant unbound or binds it to the modern-comment
namespace. Reopen can succeed while its expanded name changes. If all three
fixed candidates are producer-owned, prefix selection also falls back to the
already occupied `p188model` name. The pass-5 regression occupies only `p188`
and `p188m`, so it does not exercise the remaining fallback.

### D17, inherited and exhausted p14 shadows still corrupt dirty typed sections
`crates/rpptx-oxml/src/presentation.rs:1264`
`crates/rpptx-oxml/src/presentation.rs:1382`

The remediation retains a producer-owned `p14` declaration only when the
individual section, slide-id list, or slide-id shell owns it. A declaration
owned by `sectionLst` or an ancestor is not copied into the typed sidecar. For
example, an aliased list can own `xmlns:p14="urn:producer"`, while an aliased
typed section contains a preserved `<p14:producer/>` descendant. Dirty writing
locally rebinds `p14` to the model namespace on the generated section and
replays the raw descendant unchanged, so its expanded name changes and reopen
succeeds. A shell that owns and uses all three fixed candidates has a second
failure. Prefix selection reuses the occupied `p14model` fallback and emits
conflicting declarations instead of returning an error or selecting a free
prefix. The pass-5 regression covers separately shell-owned `p14` shadows, not
either trigger.

## Smells

None.

## Nitpicks

None.

## Prior finding status

- D1 is remediated. Open rejects shared comment-part ownership, and commented-slide duplication is refused atomically.
- D2 is remediated. Section discovery requires the exact extension URI and direct `p:ext` parent.
- D3 is remediated. A self-closing slide extension list expands in place and retains its opening bytes.
- D4 is remediated. Comment and reply status values are validated on parse and write.
- D5 remains open as cited above. `p188m` and inherited reply DrawingML coverage pass, but the final `p188model` fallback remains unsafe.
- D6 is remediated. Direct text, CDATA, comments, processing instructions, and other raw events survive at comment model boundaries.
- D7 is remediated. Ordinary unsupported comment attributes retain their lexical source bytes.
- D8 is remediated. An actually self-closing presentation extension list expands in place.
- D9 is remediated. Section, slide-id-list, and slide-id attributes and direct raw events retain their lexical bytes during dirty writes.
- D10 is remediated. Public author, comment, and reply identifiers and timestamps are revalidated during serialization.
- D11 is remediated. The facade round-trip gate checks reply movement plus notes and handout header-footer values after reopen.
- D12 is remediated. Generated children under a dirty aliased section list carry a model namespace binding and reparse.
- D13 is remediated. Section insertion recognizes a self-closing extension-list root from its terminal lexical form.
- D14 is remediated. Alias-only section lists clear without retaining an invalid empty typed shell.
- D15 is remediated. Author boundaries reconcile original author ids, so an appended author remains before the original trailing raw sidecar.
- D16 is remediated. The named facade round-trip gate asserts both saved collaboration content-type overrides.
- D17 remains open as cited above. Locally owned `p14` shadows survive, but inherited and fully occupied fallback bindings do not.
- D18 is remediated. Slide-list raw boundaries reconcile against original slide-id identity for removal and reorder.
- D19 is remediated. Dirty section replacement and clearing retain inherited alternate aliases used by original direct section children.
- The pass-5 nitpick is remediated. Author-list parsing computes and stores the raw attribute sidecar once.

## Not found

Correctness, contract, panics, OOXML, tests, and structure were all checked.
Smells produced zero findings. Nitpicks produced zero findings. Panics produced
zero findings. Structure produced zero findings. The only new module was
explicitly approved, and the diff adds no trait, public generic, crate,
feature, builder, forwarding wrapper, or production dependency. The four exact
pass-5 focused regressions pass. No additional defects were found in comment
graph ownership, relationship resolution, section identity-boundary
reconciliation, inherited alternate section discovery, notes-master or
handout-master schema order, atomic facade staging, or collaboration content
type creation.
