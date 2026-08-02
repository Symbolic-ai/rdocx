# F-080, Modelled round-trip gate

**Status**: completed
**Sprint**: S19
**Size**: M
**Depends on**: F-079

## Problem

The corpus tests currently prove an opaque package save at
`crates/rpptx-oxml/tests/integration.rs:388` and isolated structural
round-trips for presentation, slide, layout, master, and notes roots. They do
not assemble those writers into one package-level modelled save and therefore
cannot show which exact deck and part a combined save changes.

The current HLD says the modelled save compares package parts against original
bytes at `docs/hld/06-presentationml-model.md:173`. That is not compatible with
the writer contract at `docs/hld/03-architecture.md`, "Crate-level
conventions": modelled roots read producer prefixes and write fixed prefixes.
For example, `CT_Presentation::to_xml()` deliberately omits the producer XML
declaration and writes fixed namespace order. Structural equality is the
correct gate for rewritten modelled XML. Byte equality remains exact for every
part the modelled save did not rewrite and for the expected rewritten bytes
after OPC save and reopen.

The first approved corpus run also found a producer-compatible theme whose
`a:fmtScheme` omits `@name`. The existing `CT_StyleMatrix` rejects that deck.
F-080 therefore includes the narrowly approved compatibility repair:
`CT_StyleMatrix.name` becomes `Option<String>`, and the parser and writer
preserve whether the attribute was absent.

The first native acceptance attempt also exposed canonical `a:blip` output
that retained `r:embed` while dropping the relationship namespace binding
declared on the producer element. F-080 therefore requires `a:blip` to declare
the fixed `r` namespace locally whenever it writes `r:embed` or `r:link`.

## Spec reference

- `docs/hld/03-architecture.md`, "Crate-level conventions".
- `docs/hld/04-opc-and-packaging.md`, "The package" and "Deterministic output".
- `docs/hld/05-drawingml-model.md`, "Theme" and "Preservation".
- `docs/hld/06-presentationml-model.md`, "Parts", "Hard structural
  constraints", and "Preservation strategy".
- `docs/hld/12-testing-strategy.md`, "The deck corpus".
- `docs/hld/14-development-backlog.md`, "F-080, Modelled round-trip gate".
- `docs/hld/15-build-and-toolchain.md`, "Publishing".

## Approach

Extend the existing `crates/rpptx/tests/integration.rs` entrypoint created by
F-079. Add `oxml-drawing` only as a dev dependency for direct theme-model
access. Do not create a second integration binary or a production module.

Update `crates/oxml-drawing/src/theme.rs` so an absent
`a:fmtScheme/@name` parses as `None` and writes without inventing the
attribute. Keep present names unchanged. No other theme field or validation
rule changes under this compatibility repair.

Update `crates/oxml-drawing/src/fill.rs` so canonical `a:blip` output declares
the relationship namespace whenever it writes a relationship attribute. Keep
the declaration local to `a:blip`, which makes independently serialised fills
namespace-valid without depending on an ancestor declaration.

For every content-type override in each verified corpus package, dispatch the
currently modelled root types:

- `CT_Presentation`
- `CT_Slide`
- `CT_SlideLayout`
- `CT_SlideMaster`
- `CT_NotesSlide`
- `CT_NotesMaster`
- `CT_OfficeStyleSheet`

For each such part, parse the original bytes, serialise the model, reparse the
serialised bytes, and require structural equality with deck and part context.
Replace that part in an in-memory expected package with the exact serialised
bytes. Require at least one instance of every modelled content type across the
corpus so a missing dispatch cannot turn into a silent pass.

Save the facade-owned presentation, slide, and notes model through
`Presentation::to_bytes()`. For layouts, masters, notes masters, and themes,
the test harness inserts their independently serialised model bytes into the
same expected package before the deterministic OPC write. Reopen the saved
package and compare it part by part against that expected in-memory package:

- content types and relationships are structurally equal
- every rewritten modelled part is byte-equal to its expected serialised bytes
- every unmodelled part is byte-equal to its original bytes
- part names and counts are unchanged

This interpretation catches package loss and unintended writes without
contradicting the fixed-prefix writer contract. Update the HLD to state this
boundary precisely. Do not weaken byte preservation inside captured unmodelled
subtrees.

When `RDOCX_PPTX_SAVE_DIR` is set, the same gate writes the 50 saved packages
to that ignored directory and emits a sorted filename and SHA-256 manifest to
the test log. Use `corpus/pptx-s19-modelled` for the S19 evidence run. The
manual acceptance protocol is then:

1. Confirm Microsoft PowerPoint 16.104 build 16.104.25121423.
2. Open every one of the 50 saved files in PowerPoint.
3. Record each filename as opened without a repair prompt, or stop on the first
   repair prompt with its filename and message.
4. Record the operator confirmation, deck count, PowerPoint build, output
   manifest, and date in the F-080 delivery evidence.

The manual result is never represented by an automated passing test. A missing
operator confirmation keeps F-080 incomplete and is classified as
`human-action` during sprint review.

## Rejected alternatives

- Require rewritten modelled XML to equal its producer bytes. This contradicts
  the approved fixed-prefix writer contract and would turn F-080 into a new
  lossless lexical XML architecture.
- Compare only the parsed trees. That misses dropped opaque package parts,
  renamed parts, and relationship damage.
- Treat `Presentation::to_bytes()` as opaque and never execute the typed
  writers. That would repeat the already completed raw OPC gate rather than
  establish the M8 modelled gate.
- Automate the PowerPoint result and call it manual. The testing strategy says
  this gate is not automatable and not skippable.
- Commit the 50 saved decks or their hashes. They are reproducible ignored
  evidence derived from the pinned corpus, not source fixtures.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `format_scheme_without_name_round_trips_without_inventing_attribute` | A producer theme without `a:fmtScheme/@name` parses, retains `None`, writes without the attribute, and reparses structurally |
| regression | `blip_relationship_prefix_is_declared_in_canonical_output` | Canonical `a:blip` output binds the fixed relationship namespace before writing `r:embed` |
| round-trip | `all_corpus_modelled_parts_reparse_structurally` | Every currently modelled root in all 50 decks parses, serialises, reparses, and remains structurally equal with nonzero coverage per content type |
| integration | `all_modelled_corpus_packages_match_expected_parts` | Saved packages retain exact part names and counts, exact unmodelled bytes, exact expected modelled bytes, and structurally equal content types and relationships |
| integration | `facade_saved_corpus_reopens_with_the_same_read_surface` | Every facade save reopens and retains slide ids, order, shapes, text, and notes |
| manual | `all_fifty_saved_decks_open_without_repair` | The recorded operator opens every generated deck in pinned PowerPoint without a repair prompt |

The backlog test gate is named explicitly: all 50 decks pass, and each opens in
PowerPoint without repair.

## HLD impact

- `docs/hld/05-drawingml-model.md`
- `docs/hld/06-presentationml-model.md`
- `docs/hld/12-testing-strategy.md`

## Risk routing

- Any parser or serialiser. Recheck fixed prefixes, `xsd:sequence`, exact
  captured-subtree preservation, and combined package integrity across all 50
  decks. Run the required corpus tests with
  `RDOCX_PPTX_CORPUS_REQUIRED=1` and an isolated target directory.
- A public data shape. `CT_StyleMatrix.name` changes from `String` to
  `Option<String>` because a current corpus producer omits the attribute. Run
  all `oxml-drawing` tests and compile every workspace consumer.
- An external oracle comparison. Pin the manual oracle to Microsoft PowerPoint
  16.104 build 16.104.25121423, record the resolved build and operator result,
  and refuse to count any skipped deck as a pass.

## Hash harness

Expected to be unchanged. The modelled PowerPoint corpus and ignored saved
decks do not participate in the 28 Word rendering hashes.

## Implementation checklist

- [x] Make `CT_StyleMatrix.name` optional in
  `crates/oxml-drawing/src/theme.rs`, preserving attribute absence on write.
- [x] Add
  `format_scheme_without_name_round_trips_without_inventing_attribute` and run
  the focused `oxml-drawing` regression.
- [x] Declare the fixed relationship namespace on canonical `a:blip` output
  with `r:embed` or `r:link`, and add the focused regression.
- [x] Add one combined dispatcher for every currently modelled presentation
  root to the existing `rpptx` integration test binary.
- [x] Parse, serialise, reparse, and structurally compare every modelled corpus
  part.
- [x] Save, reopen, and compare the expected package part by part with exact
  preservation for unmodelled parts.
- [x] Generate the ignored 50-deck S19 acceptance directory and checksum log.
- [x] Complete and record the manual pinned-PowerPoint no-repair protocol.
- [x] Update the HLD with the exact modelled versus unmodelled byte boundary.
- [x] Run the focused corpus gate, full verification, prose, and hash checks.

## Completion evidence

On 2026-08-02, a Codex-operated native acceptance run opened all 50 generated
decks in Microsoft PowerPoint 16.104, bundle build 16.104.25121423 and
AppleScript build 1214. Every deck began with zero open presentations, opened
as the only presentation at its expected absolute path, closed without saving,
and returned PowerPoint to zero open presentations. No repair prompt, timeout,
path mismatch, or presentation-count mismatch occurred in the completed run.

The ignored evidence directory is `corpus/pptx-s19-modelled`. Its 50 sorted
deck files have aggregate SHA-256
`19609644c12923fad63939656fc54681c667efa2e066fbd2a080bb717aa037fc`.

## Open questions

None. The approved gate uses structural equality for rewritten modelled XML,
exact expected serialised bytes after package save, and exact original bytes
for every unmodelled part. The approved compatibility repairs preserve absent
`a:fmtScheme/@name` and bind the fixed relationship namespace on canonical
`a:blip` output.
