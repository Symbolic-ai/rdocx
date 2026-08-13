# F-137, all, pass 10

**Reviewed**: complete current working diff from claim base `6ade43a`, 7 feature
files and 1,939 added plus 12 removed lines, the approved contract and HLD, all
nine earlier reviews, progress evidence, and all pass-9 remediation
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Not found

Pass-9 D1 is resolved. Both positive tests load the workflow with `read_bytes()`
at `scripts/test_sprint_workflow.py:732` and
`scripts/test_sprint_workflow.py:738`. The shared contract hashes those raw
bytes as its first operation, compares the reviewed SHA-256, and only then
decodes strict UTF-8 at `scripts/test_sprint_workflow.py:111`. An independent
raw-byte check reproduced the current digest
`db89119b10d04baee6011f21513f9e7a191e39bb8b2f7715857a52217f6325ac`.
The LF-to-CRLF variant reproduced
`773677700cf561368e3ca6abe17ab0ad50a0d668bbe995033c4c922353ec3f93`
and was rejected before decoding. A forced matching digest with invalid UTF-8
reached the strict decoder and raised `UnicodeDecodeError`.

The explicit CRLF mutation is constructed from the raw workflow bytes at
`scripts/test_sprint_workflow.py:1699`. The corpus contains 100 explicit
mutations, 15 generated shortcut mutations, and 40 generated control-flow
mutations, exactly 155 in total. Instrumentation observed one unchanged
positive call followed by 155 unique byte mutations. Every mutation differs
from the source bytes and is rejected at
`scripts/test_sprint_workflow.py:1706`.

The unchanged workflow passes the digest and continues through all structural
semantic assertions beginning at `scripts/test_sprint_workflow.py:117`. The
matrix, native and musl installation, pytest and typing gates, setup-python
maps, exact scripts, action revisions, checkout defaults, upload graph,
publication counts, tag predicate, permissions, environment, and OIDC boundary
remain correct in `.github/workflows/wheels.yml:3`.

The approved plan now records the exact raw-read, hash-first, strict-decode
order and the complementary semantic checks at
`.claude/plans/F-137-design.md:45`. Its dependencies, HLD impact, risk riders,
unchanged hash-harness expectation, checklist, and no-publication boundary are
intact. The four HLD updates remain within the approved file list and accurately
describe the current workflow.

No fresh correctness, contract, test-vacuity, digest-maintenance, YAML, GitHub
expression, runner, PyO3 abi3, maturin, action revision, OIDC, secret, artifact,
release metadata, HLD, scope, formatting, prose, hash, or generated-artifact
finding was found. The focused two-test gate and complete 26-test workflow
module passed. Prose, generated-skill, and diff checks also passed. The
full-workspace LibreOffice rider remains the cleanly stopped environmental
attempt recorded in progress and is not a focused feature-gate failure.
