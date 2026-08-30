# S59 sprint review, pass 1

**Reviewed**: sprint/s59 against 40e21eb9608e1705fb95fafe55130f5069d66683, 39 files, 7,828 lines, crates: oxml-opc, rpptx-oxml, rpptx
**Verdict**: 1 blocking, 0 should-fix, 0 nice-to-have

## Blocking

### B1, the signed-current-state gate omits every F-217 mutation surface
`crates/rpptx/tests/integration.rs:227`

The signature invalidation regression signs a presentation and exercises slide,
shape, text, core-property, and slide-graph mutations through line 276. S59 also
adds comment-author, comment, reply, section, notes-master, and handout-master
mutation paths at `crates/rpptx/src/lib.rs:1235`,
`crates/rpptx/src/lib.rs:1282`, `crates/rpptx/src/lib.rs:1335`,
`crates/rpptx/src/lib.rs:1435`, `crates/rpptx/src/lib.rs:1445`, and
`crates/rpptx/src/lib.rs:1456`, but no signed fixture exercises any of them.
This leaves the sprint definition of done, which requires every relevant
mutation to invalidate rather than falsely preserve signature validity,
untested at the exact interaction boundary between F-217 and F-221. Extend the
signed-current-state regression with a source-built presentation carrying the
F-217 package roots. Exercise each newly serialized collaboration and
navigation part, then prove the retained signature remains inspectable and is
reported cryptographically invalid after each mutation.

## Should-fix

None.

## Nice-to-have

None.

## Milestone gate

The M21 gate is: "one representative modern deck round-trips its comments,
sections, SmartArt, media, animation timeline, signatures, and package variant
without repair. Its static frames, animated export, notes, and handouts match
the pinned PowerPoint oracle at their declared fidelity boundaries."

The milestone gate does not yet hold, as expected for the first M21 sprint.
S59 supplies only the comments, sections, notes and handout settings,
encryption, and signature slices. SmartArt, media, animation, package variants,
animated export, and the combined representative-deck oracle remain assigned
to later M21 stories.

The completed S59 evidence is otherwise positive. The named F-217 gate
`modern_comments_replies_sections_and_handout_settings_survive_ordered_mutation_save_and_reopen`
passed in the integrated all-feature `rpptx` binary. Encryption round trip,
atomic encrypted-save failure, trusted-certificate signing and coverage,
producer-byte preservation, the existing signed-mutation cases, ordinary
notes staging, feature isolation, and all 50 corpus structural gates passed.
The ignored PowerPoint gate passed against PowerPoint 16.104 build
16.104.25121423 and candidate SHA-256
`a0d33171c63ec084231daeef3b35718f5a2d709a5c92c9c0e2017ccaf9fa52d6`
with both recorded observations. The correct password opened the file and a
wrong password was rejected. Both no-default `rpptx` graphs compiled, and the
hash harness matched 49 of 49 entries. The missing F-217 signed-mutation gate
keeps the S59 slice from satisfying its own integrated definition of done.

## Not found

Duplication produced zero findings. Layering produced zero findings. The
`oxml-opc` changes add constants only and no lower-layer dependency points to a
format crate. Harness produced zero findings, with the declared unchanged
result reproduced at 49 of 49. Docs produced zero findings outside the gate
gap above, and the exact union of both approved HLD impact lists was updated.
Dependencies produced zero findings. The only manifest changes are the two
default-off `rpptx` feature forwards to existing `oxml-opc` capabilities, with
the native `Presentation` facade as their named consumer. Surface produced zero
findings. The additive public APIs match the two approved design plans.

The post-integration amendment at 26191147 was inspected separately. Ordinary
save, render, slideshow, and atomic facade commit paths retain canonical
modelled-part staging. Encryption and signature operations preserve unchanged
producer bytes, while an ordinary signed save selects preservation from the
retained signature-origin relationship. The ordinary notes staging regression,
the untouched producer-signature regression, and the current-state mutation
regression all passed. No additional interaction finding was found in that
split.
