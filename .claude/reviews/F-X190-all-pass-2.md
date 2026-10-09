# F-X190, all, pass 2

**Reviewed**: Frozen working diff against claim Base `2cea3992ae176c4d63e433210fb59a292d8f54f3`, 10 tracked files, 889 insertions and 47 deletions. F-X190 remains in progress. Read the approved plan, cited HLD, canonical workflow, current progress, complete Base diff and immutable pass1. This pass includes every production consumer of the repaired batched namespace collector.

**Verdict**: 0 defects, 0 smells, 0 nitpicks.

## Defects

None found.

## Smells

None found.

## Nitpicks

None recorded.

## Prior finding

D1 is resolved. The existing batched collector records the inherited context
before applying the requested element's own declarations
(`crates/rdocx/src/document.rs:6562`). Ancestor bindings are retained rather
than deleting prefixes shadowed by the first run. The original run bytes keep
their own declarations, so a local alias or default binding expires at the
end of that run instead of governing later siblings. The envelope consumes
that inherited context (`crates/rdocx/src/document.rs:6659`).

All production batched consumers were traced. Item and link excerpts use the
shared projection. Owner inventories close the original single root before
scanning. Rich-comment paragraph replay closes the original paragraph
(`crates/rdocx/src/document.rs:17221`). Single-root closure explicitly keeps
existing local declarations authoritative, adding only missing bindings
(`crates/rdocx/src/document.rs:7002`). Consequently the lifetime correction
does not change the meaning of a locally bound single root. The singular
namespace helper and general typed discovery admission remain unchanged.

The native control checks all four combinations of alias or default binding
and Word or foreign ancestor, requiring direct/bulk agreement, exact XML and
source preservation, and stable reopened locations
(`crates/rdocx/tests/regression_test.rs:7238`). Python checks the same four
cases, stable revisions, held handles and exact producer bytes
(`crates/rdocx-py/tests/test_core.py:2733`). Single-root and empty-element
scope behavior has an explicit private control
(`crates/rdocx/src/document.rs:31036`). The private actual-link mechanism test
also shadows the relationship prefix only on the first run and verifies the
later qualified link's exact source endpoints
(`crates/rdocx/src/document.rs:31075`). Its marker start uses the actual first
opening-tag end rather than a fixed tag length.

The genuine current-before native run executes all four admission cases and
reports bulkSHADOW7 versus direct7, or bulk7 versus direct87. The separately
authenticated current-before Python runtime reports four corresponding
failures after preservation and read-only assertions have executed. These
are later-current D1 proofs, distinct from the two original claim Base proofs.
Fresh focused and full final2 runs pass these controls.

## Not found

- Correctness: Zero findings. Sibling runs share inherited namespace context, while local declarations retain their original lifetimes. Nested ancestor offsets remain discovery-only, and checked coordinate translation restores physical link spans. The complete Base implementation and bounded remediation have no additional finding.
- Contract: Zero findings. Existing bulk snapshots, direct text reads and Python snapshots expose stored caches without evaluation or a new public API. Supported run-only nested fields preserve instruction and result visibility. The public opaque internal-hyperlink discovery boundary remains explicit, with private qualified-span mechanism proof and separate supported public link controls.
- Panics: Zero findings. Checked source slicing, interval arithmetic and inverse endpoint bounds replace the external ancestor underflow path (`crates/rdocx/src/document.rs:6680` and `crates/rdocx/src/document.rs:6731`). No new unchecked production indexing or unwrap was introduced.
- OOXML: Zero findings. Namespace bindings remain authoritative across aliases, defaults, local shadows and foreign nodes. Original run bytes, field XML, unrelated package members and schema ordering remain preserved. The inert envelope is private reader state and is never saved. Single-root local declarations remain authoritative.
- Tests: Zero findings. The genuine compiled Base sibling-cache and admitted nested-underflow failures have current regression coverage. D1 now has native and actual Python before/after controls. Supported owner, namespace, cache grammar, exact location, source byte, read revision and public/private hyperlink distinctions remain covered. Non-complex enclosing-link text preserves established literal7yes compatibility, while complex outer and inner fields retain yes and7.
- Structure: Zero findings. No new production file, module, crate, dependency, feature flag, trait or generic. The existing batched collector and one shared private projection serve concrete current consumers. Tests extend existing entrypoints. HLD updates match the approved impact list.

## Evidence checked

Authenticated `/private/tmp/fx190-final2-source-binder/binder.json`, SHA256
`22621621ae5bff3b9a2487f7e991cb201bc9c57368840e889a461b4333842f0f`.
All 10 live and captured source bindings, current progress, 15 records and
42 logs match. Pass1 authenticates unchanged. Log authentication is distinct
from reading result contents. Current-before native and Python, focused after,
link interval and final native and binding result contents were examined.

The current-before native log SHA256 is
`197ac80c5a75901fa41182d0346bfeab1d8a5558a0b181256990bb19e05b5a4f`.
The current-before Python log SHA256 is
`01cbc9eb71c0d868b92874ab4c6b42bace4e100794f713675852e112099f7d86`.
Only the two declared original compiled failures are exact claim Base
evidence. Compilation failures and the original unadmitted hyperlink fixture
remain excluded from behavior-failure claims.

Final2 scoped receipt SHA256
`d8de3bfb123e6a6ae252730bab48ea8a342ce46de9b8d0381983fd558dab42ca`
records all 13 steps green. Affected native suites report1822 passed and
22 existing ignored. Fresh rebuilt Python reports105 passed, strict mypy
checks seven files and stubtest checks six modules. Both imported and retained
extension copies authenticate to current provenance SHA256
`fadb1b1925fb54c9f9129233e440d2bd3b740e7cff21a2ca10f09e49b7322421`.

Actual and retained verified publication dry-run archives authenticate. All25
archived source and test members were opened and compared with current bytes.
The archive remains below10MiB. Member bytes and count match the recorded
measurement, with a one-byte compressed size variation inside the existing
64-byte tolerance. All49 hash entries remain unchanged. Workflow reports
140 tests with two skips, prose reports zero violations and all26 adapters
remain synchronized.

These are scoped feature gates, not final integrated sprint verification or
publication approval. Only this review file was written. No source, tests,
plan, HLD, progress, Git or Cargo mutation occurred, and no tests were run
during review. This pass ends here and returns control to the orchestrator.
