# S71 sprint review, pass 3

**Reviewed**: sprint/s71 against fd069fd460fec11b27c2f6eb1004d4a9ee9bcade,
76 files, 26930 changed lines, crates: oxml-media, oxml-opc, rdocx-layout,
rdocx-oxml, rdocx, rpptx
**Verdict**: 0 blocking, 1 should-fix, 0 nice-to-have

## Blocking

None.

## Should-fix

### S1, Remove the duplicate theme-language setter

`crates/rdocx/src/document.rs:8546`

F-244 added `Document::set_theme_font_language` as the staged owner of the
settings mutation. F-245 then added `Document::set_language_defaults` as a
public method whose complete implementation forwards to that setter. The two
names expose the same value and behavior, and the later method violates the
repository rule against forwarding-only wrappers. Keep one public mutation
path, update the affected tests and current HLD surface description, and retain
the same atomic settings mutation and layout invalidation behavior.

## Nice-to-have

None.

## Milestone gate

The M23 end gate requires all five private references to be generated through
the blank public facade, reopen without repair, match package semantics and
reviewed visual thresholds, serialize identically, and report no unexplained
preservation-only fallback
(`docs/hld/14-development-backlog.md:2214`). That end-of-milestone gate does not
yet hold at this dependency-prefix boundary because F-246 through F-248 remain
unfinished. This pass does not assert otherwise.

The integrated prefix has direct interaction evidence. F-244's
`authored_settings_and_properties_survive_reopen` gate covers the complete
bounded settings and property surface. F-245's
`authored_theme_font_table_and_embedded_fonts_survive_reopen` and pinned
`public_authored_theme_and_fonts_match_pinned_word_resolution` gates exercise
theme language through the same settings owner while proving theme, font-table,
embedded-font, and deterministic layout behavior. The full gate passed at
`6be86bf60afc6dd5f460f3ddc7e475ef27ce9da0` with all 49 hash entries unchanged,
all package archives below 10 MiB, and the pinned LibreOffice, Poppler, and
python-pptx riders green.

## Not found

No blocking interaction, layering, harness, milestone-gate, documentation,
dependency, or unrelated public-surface defect was found. No manifest changed,
the `oxml-*` dependency direction remains intact, F-249 remains the single
identifier owner, and F-244 and F-245 use that staged package allocator. The
two reviewed F-249 hash changes reconcile with the baseline and AS_BUILT entry,
while F-244 and F-245 leave all 49 reviewed entries unchanged. Every HLD file
listed by the integrated feature plans changed.
