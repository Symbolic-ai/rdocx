# S59 sprint review, pass 2

**Reviewed**: sprint/s59 against 40e21eb9608e1705fb95fafe55130f5069d66683, 39 files, 7,929 lines, crates: oxml-opc, rpptx-oxml, rpptx
**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have

## Blocking

None.

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
S59 supplies the comments, sections, notes and handout settings, encryption,
and signature slices. SmartArt, media, animation, package variants, animated
export, and the combined representative-deck oracle remain assigned to later
M21 stories.

The S59 slice now satisfies its integrated gate. The pass-1 B1 remediation
adds `collaboration_mutations_never_report_the_retained_signature_as_valid` at
`crates/rpptx/tests/integration.rs:281`. Its `fresh` and `signed` closures at
lines 285 to 290 construct and sign a separate presentation for every case.
The cases cover comment-author mutation at line 329, comment mutation at lines
333 to 337, reply mutation at lines 339 to 346, section mutation at lines 348
to 359, notes-master header-footer mutation at lines 361 to 366, and
handout-master header-footer mutation at lines 368 to 373. The shared assertion
at lines 292 to 298 requires one retained report and requires every retained
signature to be cryptographically invalid. The exact regression passed at
commit 3e6e83c.

The named F-217 round-trip gate remains
`modern_comments_replies_sections_and_handout_settings_survive_ordered_mutation_save_and_reopen`
at `crates/rpptx/tests/integration.rs:5947`. The trusted-certificate signing,
encrypted round trip, wrong-password, atomic failure, producer-byte
preservation, ordinary staging, and feature-isolation gates remain positive.
Pinned Microsoft PowerPoint 16.104 build 16.104.25121423 opened candidate
SHA-256
`a0d33171c63ec084231daeef3b35718f5a2d709a5c92c9c0e2017ccaf9fa52d6`
with the correct password and rejected a wrong password. The hash harness
matched 49 of 49 entries at the reviewed head.

## Not found

Interaction produced zero findings. Pass-1 B1 is remediated by commit 3e6e83c.
Each collaboration mutation signs a separately constructed package, retains
the signature report, and makes it cryptographically invalid. The ordinary
staging amendment also remains correct. `Presentation::to_bytes` selects
producer-byte preservation only when retained signature-origin infrastructure
exists at `crates/rpptx/src/lib.rs:655`, while ordinary rendering and facade
commits select canonical modelled-part staging at
`crates/rpptx/src/lib.rs:711` and `crates/rpptx/src/lib.rs:1505`. Encryption,
verification, and signing select preservation at
`crates/rpptx/src/lib.rs:669`, `crates/rpptx/src/lib.rs:681`, and
`crates/rpptx/src/lib.rs:696`. The ordinary notes staging regression at
`crates/rpptx/tests/integration.rs:398` and untouched producer-signature
regression at `crates/rpptx/tests/integration.rs:203` both passed.

Duplication produced zero findings. The two stories share the existing facade
staging and package-security implementation without adding parallel helpers.
Layering produced zero findings. The `oxml-opc` delta adds only PresentationML
relationship and content-type constants, and no `oxml-*` crate gained a format
crate dependency. Harness produced zero findings. Both design plans and both
AS_BUILT entries declare an unchanged result, reproduced at 49 of 49.

Gate produced zero sprint-scope findings. The S59 definition of done has
executable evidence for collaboration round trips, retained-signature
invalidation across both stories, feature isolation, and the pinned manual
PowerPoint password observations. The broader M21 gate remains open for the
later stories named above.

Docs produced zero findings. The changed HLD set is exactly the union of the
two approved impact lists: 02, 03, 04, 06, 10, 12, and 15. The collaboration,
security staging, native-only API, testing, dependency, and portability
contracts agree with the integrated implementation.

Dependencies produced zero findings. The only manifest additions are the two
default-off `rpptx` feature forwards at `crates/rpptx/Cargo.toml:22` and
`crates/rpptx/Cargo.toml:24`, with native `Presentation` as their named
consumer. Inspected ordinary, Python, WASM, and CLI graphs contain none of the
added cryptographic dependencies, while the explicitly enabled native graph
contains them.

Surface produced zero findings. The additive pre-1.0 public APIs match the two
approved design plans. F-217 adds the approved comments module and the bounded
collaboration and navigation model. F-221 adds only the feature-gated native
encryption and signature facade surface. No unrequested binding, CLI, WASM,
trait, generic, crate, or production dependency was added.
