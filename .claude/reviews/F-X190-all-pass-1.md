# F-X190, all, pass 1

**Reviewed**: Frozen working diff against claim Base `2cea3992ae176c4d63e433210fb59a292d8f54f3`, 10 tracked files, 689 insertions and 43 deletions. F-X190 is in progress. Read the approved plan, cited HLD, canonical workflow, current progress and complete diff.

**Verdict**: 1 defect, 0 smells, 0 nitpicks.

## Defects

### D1, The shared envelope promotes first-run local bindings into later siblings

`crates/rdocx/src/document.rs:6658`

The complex-field envelope writes every binding from `scope` onto a new parent
of all source runs. That scope includes declarations local to the first run.
The bulk scope collector applies the element's own declarations before storing
the scope (`crates/rdocx/src/document.rs:6562`), and bulk snapshots retrieve
that scope at `item.scan.start` (`crates/rdocx/src/document.rs:17312`). A local
binding that ends with the first run therefore incorrectly governs later runs
inside the new envelope.

A concrete input uses `q` for the Word namespace and `x` for `urn:producer`
on the document. Its otherwise ordinary five-run PAGE field begins with
`<q:r xmlns:x="http://schemas.openxmlformats.org/wordprocessingml/2006/main">`
and a q-qualified begin marker. Instruction, separator and end markers are
q-qualified. A later cached-result run is
`<q:r><x:t>SHADOW</x:t><q:t>7</q:t></q:r>`. In the physical source, that `x:t`
is foreign producer content because the first run's local binding has ended.
The new envelope instead binds `x` to Word across every sibling, so it exposes
`SHADOW7` rather than the direct source read's `7`.

This is not excluded by the existing typed field admission boundary. The
qualified physical scan finds the complete q-qualified field. The admission
projection itself obtains the first-run scope and puts it on its paragraph
wrapper (`crates/rdocx/src/document.rs:9257`). It then compares retained raw
field source rather than requiring source text semantics to match
(`crates/rdocx/src/document.rs:9286`). The typed reader accepts complete
begin, separator and end events, and retains original source runs
(`crates/rdocx-oxml/src/text.rs:4437` and
`crates/rdocx-oxml/src/text.rs:4627`). This run-only construction crosses no
unsupported internal-hyperlink discovery shape.

Direct text reads scan the actual source (`crates/rdocx/src/document.rs:1410`)
and classify foreign nodes by their resolved namespace
(`crates/rdocx/src/document.rs:10271`). Bulk text uses the new envelope
(`crates/rdocx/src/document.rs:17315`). The changed binding therefore violates
both direct/bulk parity and the approved authoritative foreign-shadow
contract. The inverse binding case can hide genuine later Word text.

Use the namespace context inherited by the run sequence, retaining each run's
local declarations only on its original element. Do not merely remove local
prefixes from the envelope when that would lose the ancestor binding they
shadow. Add direct/bulk and Python controls with first-run local bindings
that conflict with later siblings' inherited bindings, including a foreign
lookalike and the reverse shadow. Assert exact source bytes and unchanged
locations. The present matrix tests aliases, defaults and redundant local
Word declarations, but not this lifetime boundary
(`crates/rdocx/tests/regression_test.rs:7127`).

This finding is proved by source inspection. No construction or test was
executed during this review, and no retained runtime log is claimed to
reproduce D1.

## Smells

None found.

## Nitpicks

None recorded.

## Not found

- Correctness: No additional finding beyond D1. The marker interval and checked forward and inverse offsets retain the intended isolation. Clearing external complex ancestor offsets is consistent with their discovery-only use.
- Contract: D1 is the remaining namespace and parity defect. No new public snapshot API, field evaluator or expanded typed admission was introduced. Existing location, XML, ordering and read-only revision contracts otherwise remain intact.
- Panics: Zero findings. The former nested ancestor subtraction is removed. The new coordinate mapping checks subtraction, addition, bounds and interval order before returning spans (`crates/rdocx/src/document.rs:6679` and `crates/rdocx/src/document.rs:6729`). No new unchecked production indexing or unwrap was introduced.
- OOXML: D1 changes qualified identity inside an ephemeral projection. No additional finding. Original source bytes and declarations remain copied verbatim, and the envelope is not serialized into the package. Single-root projection and global scanners remain unchanged.
- Tests: D1's conflicting declaration-lifetime case is missing. Otherwise the tests exercise the two real compiled Base failures, six supported story-owner cases, nested fields, cached text grammar, exact locations and fingerprints, source preservation and Python held handles. The private real-link span test is correctly identified as mechanism proof rather than public admission proof.
- Structure: Zero findings. No new production file, module, crate, trait, generic, dependency or feature. One shared private projection directly serves existing text and link consumers. Tests extend existing entrypoints, and HLD updates match the approved impact list.

## Evidence checked

Authenticated `/private/tmp/fx190-final1-source-binder/binder.json`, SHA256
`022ec1bd226fa7d42d9dbb9ed2edadfbbbb12363b276b7829f7cb7af842b1836`.
All 10 live and captured source bindings, current progress, 12 records and
32 logs match. Log authentication is distinct from reading result contents.
The named before, admitted nested before, expanded after and final native and
binding result contents were examined.

The genuine compiled exact-Base named gate reports empty sibling-run bulk
text while direct text is7, SHA256
`31af2334d239318e7d47fff49513cf0ce25ef8302c12ba9acc974da9fd191ad8`.
The separately admitted run-only nested case executes four underflow panics
through bulk items, inner direct links, story links and bulk links, SHA256
`dff1173770a8161deb98eececaf66c5d6db13f5966a9fffcc10451cec766a0ba`.
The original internal-hyperlink fixture discovers no public field and proves
no offset panic. Compilation failures are not behavior proof. The corrected
non-complex enclosing hyperlink expectation preserves literal7yes, while
actual outer and inner complex fields retain yes and7. The original global
scanner policy was not changed to satisfy that expectation.

Final1 scoped receipt SHA256
`119531bf462c4e977be3f314ec25eb9559d8a616868820366f141183fbd71b95`
records all 13 steps green. Affected native suites report1820 passed and
22 existing ignored. Fresh current Python reports101 passed, strict mypy
checks seven files and stubtest checks six modules. Both actual imported and
retained extension copies authenticate to current provenance. Actual and
retained publication dry-run archives authenticate, and all25 archived source
and test members were opened and compared with current bytes. The archive is
below10MiB, with exact member bytes and count. Its two-byte compression
measurement variation is within the existing64-byte tolerance. All49 hash
entries remain unchanged.

These are scoped feature gates, not final integrated sprint verification or
publication approval. Only this review file was written. No source, tests,
plan, HLD, progress, Git or Cargo mutation occurred. No tests were run during
review. Return to the orchestrator for distinct remediation of D1.
