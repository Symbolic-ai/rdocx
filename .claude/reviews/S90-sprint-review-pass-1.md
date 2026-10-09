# S90 sprint review, pass 1

**Reviewed**: sprint/s90 at69f900a1a26747dda643d8e654bd8cec053fe18e against
merge Base20888b7a2c636e322ad23dc611f494beaac1c09b,185files,
405444added and7436removed lines. Crates: oxml-chart, oxml-cli-support,
oxml-core, oxml-drawing, oxml-layout, oxml-media, oxml-opc, oxml-pdf,
oxml-sml, rdocx, rdocx-cli, rdocx-html, rdocx-layout, rdocx-opc,
rdocx-oxml, rdocx-pdf, rdocx-py, rdocx-wasm, rpptx, rpptx-chart,
rpptx-cli, rpptx-layout, rpptx-oxml, rpptx-py, rpptx-render and rpptx-wasm.
Root owns this review record. Independent reviewers audited interaction,
duplication, surface and docs, and separately harness, gate, layering and deps.
This pass changes no implementation or delivery files.

**Verdict**: 0 blocking, 1 should-fix, 0 nice-to-have.

## Blocking

None found.

## Should-fix

### SF1, Document the new exhaustive WordStory match case

`CHANGELOG.md:34`
`crates/rdocx-layout/src/lib.rs:25`
`crates/rdocx-layout/src/lib.rs:39`
`.claude/plans/F-279-design.md:97`
`docs/hld/10-bindings-spec.md:1882`

Classification: fix-now, in-scope documentation gap.
The Word Compatibility notes explain new exhaustive FieldKind cases but omit
WordStory::TextBox. WordStory is public and not non_exhaustive, so existing
exhaustive downstream matches require the new case to compile. F-279 approves
the variant and HLD10 documents it. Add a precise migration sentence to the
Word Compatibility notes. No API removal or runtime correction is required.

## Nice-to-have

None recorded.

## Integrated interactions and evidence

`crates/rdocx-oxml/src/text.rs:825`
`crates/rdocx/src/field.rs:13533`
`crates/rdocx-layout/src/engine.rs:600`
`crates/rdocx-layout/src/engine.rs:3624`
`crates/rdocx/src/document.rs:6559`
`crates/rdocx/src/field.rs:18277`
`crates/rdocx/src/bibliography.rs:1849`
`crates/rdocx/src/document.rs:27039`
`crates/rdocx/src/document.rs:20739`
`crates/rdocx/tests/regression_test.rs:61781`

Field registration, update and layout share recursive preorder. Sequence data
is derived from current input. Namespace scopes do not leak across sibling
runs. Generated-table staging performs one paginator call before validated
publication. Bibliography and ownership mutations retain staged reopen and
atomic publication boundaries. Scoped replacements prove physical correspondence,
and whole-story edits preserve live identities while validating reopened output.
The Issue292 positive main-body fragment control includes note and text-box
review dependencies. No concrete cross-feature defect was found in these paths.

The actual full generic gate at769202b0 passed. Formatting and strict workspace
lint are bound in oz2q3i3z, resumed remaining commands in hmo1bqu5, receipt
caf22a0b. Native5708passes/58existing ignores include all1340APA prefix controls,
current field/table/comment/numbering tests, dense form, fixed/autofit/nested
tables and grid/vertical controls. Workflow139passes/2existing registry skips,
rustdoc, README examples, no-default layout, dual WASM, packaging and supply
chain passed. Earlier stopped cache-path and offline-source attempts remain
failures with authenticated environmental recovery, not aggregate passes.

Final integrated installed Word runtime178per Python3.9/3.12 and PowerPoint
75normal plus2exact Cargo-oracle cases per interpreter passed. Strict typing
and stub checks passed. Receipts9df89453/de0636df/eaa2808e bind actual installed
wheel and native bytes. WASM receipt63ec6acb binds11stages, Node2Word/1PPT,
actual size1test at684723gzip bytes and byte-equal local npm installs/imports.
Actual22-package publication dry run used local patches without allow-dirty.
Archive receipt0b7de36f binds324members/258current sources and clean VCS metadata.
Golden receipt2304d0fb proves7page-one decoded pixel buffers at150DPI with
Poppler26.01.0. Current Word primary captures, compiled comparisons and retained
hash/golden pairs were independently authenticated. Queue self-snapshot
advisories remain qualified against matching final primary records.

`docs/sprints/AS_BUILT.md:18438`
`scripts/hash_baseline.json:1`

X182 five and X183 fourteen labelled events overlap on14unique final keys.
The remaining35of49keys are untouched. Current49hashes match their reviewed
baseline. No unexplained delta or new golden baseline was accepted.
External registry dependency records remain equivalent to sprint Base.
Internal version pins advance within the exact7Word/15shared-PPT families.
Neutral crate manifests add no format-specific dependency edge.

## Milestone gate

`docs/hld/14-development-backlog.md:2605`
`docs/sprints/CURRENT_SPRINT.md:9`
`docs/sprints/CURRENT_SPRINT.md:45`
`docs/sprints/BACKLOG.md:40`

The M24 end gate requires "the approved capability matrix has no unexplained
partial row" and broad strict/transitional, public authoring, preservation,
rendering and binding checks without Word repair. S90 does not close M24.
Thirty M24 stories remain pending. No broad manual without-Word-repair or full
catalogue conclusion is asserted. Delivered scope is supported by the actual
checks above. F-283 and remaining bibliography F-X192 belong to S91, F-X178
is explicitly carried for the Issue264 exclusion, and Issue281 remains open.

The69f900 ledger commit changes only four delivery files. Its18done/2carried
and backlog470done/0in-progress/53pending/4archived counts reconcile. Source
checks at769202 remain exact source evidence, but final-HEAD full verification
must repeat after this review and any remediation commits. The exact pushed-SHA
hosted12wheel/2sdist build-only rehearsal also remains a separate pre-close
gate. No hosted artifact, tag, registry upload or release approval is claimed.

## Not found

Interaction: zero blocking or should-fix findings. Duplication: zero findings.
Layering: zero findings. Harness: zero findings. Gate: zero findings within
delivered scope and explicitly retained closure obligations. Deps: zero findings.
Surface: zero unauthorized API findings. Docs: one should-fix SF1 above, no
other findings. No findings were manufactured for categories with zero results.

The review pass ends here. Remediation is a distinct implementing phase.
