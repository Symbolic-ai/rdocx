# F-198, correctness, pass 5

**Reviewed**: reconstructed working-tree diff against
`bc478f8a06d37268d06cd41598037df1d91b0611`, 17 tracked implementation, HLD,
and baseline files with 1,051 additions and 44 deletions, plus 4 restored prior
review records with 182 lines
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Not found

Pass 4 D1 is closed. All four related-story regressions again construct 700
paragraphs and preserve their original note-clean restart, uncached-tail
endnote completion, header and footer, invalidation, page-numbering, and exact
warm-versus-fresh claims at `crates/rdocx-layout/src/engine.rs:8963`,
`crates/rdocx-layout/src/engine.rs:9007`,
`crates/rdocx-layout/src/engine.rs:9066`, and
`crates/rdocx-layout/src/engine.rs:9102`. The complete 186-test
`rdocx-layout` library suite passes at the reviewed state, including those four
workloads and the unchanged 8 MiB acceptance boundary documented at
`docs/hld/12-testing-strategy.md:408`.

The compact restart body retains a fingerprint only as a prefilter and compares
authoritative canonical serialized bytes afterward at
`crates/rdocx-layout/src/engine.rs:703`. Canonical identity creation is fallible
at `crates/rdocx-layout/src/engine.rs:790`, and a failed or incomplete identity
drops the whole candidate at `crates/rdocx-layout/src/engine.rs:1775` rather
than publishing partial retained state. The identity payload, body vector, and
view-specific note-reference payload are all charged at
`crates/rdocx-layout/src/engine.rs:747` and
`crates/rdocx-layout/src/engine.rs:3067`. The focused coarse-fingerprint
collision case covers the Latin, East Asian, bidi, and retained foreign
`w:lang` state at `crates/rdocx-layout/src/engine.rs:8552`.

Revision view, automatic hyphenation, styles, related stories, fonts, and every
other reusable context input remain exact at
`crates/rdocx-layout/src/engine.rs:469`. The retained note sequence is computed
through the active revision view and compared before restart at
`crates/rdocx-layout/src/engine.rs:802` and
`crates/rdocx-layout/src/engine.rs:1384`. Body prefix and suffix discovery uses
the same authoritative identity at `crates/rdocx-layout/src/engine.rs:1397`,
so cached-tail attachment and body completion cannot disagree about the body
state they reuse.

Pass 3 D1 through D4 remain closed. Self-closing settings retain their root
QName and attributes while gaining a valid fixed Word binding at
`crates/rdocx-oxml/src/settings.rs:354`. Settings allocation checks parts,
relationship owners, and content-type overrides at
`crates/rdocx/src/document.rs:1284`. Malformed retained `w:lang` content keeps
its modeled occurrence and repeated-serialization position at
`crates/rdocx-oxml/src/properties.rs:1112`. The intentional pre-1.0
`CT_RPr` and `LayoutInput` full-literal source breaks remain explicit at
`docs/hld/10-bindings-spec.md:100`.

No additional correctness, contract, panic or error-path, OOXML namespace or
schema-order, raw-preservation, public compatibility, test, HLD, dependency,
or structural findings were found. Automatic-hyphenation invalidation,
paragraph suppression, inherited and mixed run language, fields, notes,
tables, drawings, exact conditional-hyphen source spans, warm and fresh output,
and F-X062, F-X063, and F-X066 interactions remain coherent. The declared hash
movement is still limited to five `feature_showcase` keys, with deterministic
golden and pinned Writer evidence. Current carrier documentation keeps the
0.7.0 workspace proof separate from the immutable
`rdocx-layout@0.10.1` to `oxml-layout@0.6.0` registry regression at
`docs/hld/12-testing-strategy.md:1131`.
