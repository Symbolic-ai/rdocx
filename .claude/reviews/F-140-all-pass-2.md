# F-140, all, pass 2

**Reviewed**: the complete seven-file working diff, 421 insertions and 34 deletions, against the approved plan, progress notes, pass 1, and HLD 10, 12, 14, and 15
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, the testing HLD still assigns the setup-node pin to the wrong release

`docs/hld/12-testing-strategy.md:454`
`.github/workflows/ci.yml:95`

The workflow provenance correction now accurately identifies
`249970729cb0ef3589644e2896645e5dc5ba9c38` as setup-node v6.5.0, but the new
HLD paragraph says the same operative pin is setup-node v6.1.0. The upstream
tag refs resolve v6.5.0 to the approved SHA and v6.1.0 to
`395ad3262231945c25e8478fd5baf05154b1d79f`. HLD12 therefore contradicts the
reviewed workflow and does not describe current provenance accurately. Change
the HLD version label to v6.5.0.

## Smells

None.

## Nitpicks

None.

## Not found

- Pass-1 D1 is resolved. The shared preparation contract requires the exact
  13-member incubating group and separately fixes `rpptx-wasm` at explicit
  version 0.1.2, unpublished, absent from workspace dependency pins, present
  once in the lockfile, and carrying the exact incubating release metadata at
  `scripts/test_sprint_workflow.py:2654`. Independent family, tag, and version
  mutations at `scripts/test_sprint_workflow.py:2748` all fail the contract.
  The five focused positive and mutation regressions passed.
- Workflow trigger, privilege, action inputs, and control flow produced no
  finding. The structured contract at `scripts/test_sprint_workflow.py:472`
  requires the unconditional pull-request trigger, exact root read permission,
  exact job shape and step order, immutable action SHAs, exact Node and
  wasm-pack versions, locked checks for both packages, and both unfiltered Node
  suites. Its mutation table rejects conditions, package omissions,
  listing-only execution, failure swallowing, and early success.
- The operative commands at `.github/workflows/ci.yml:101` preserve ordinary
  failure propagation. Both target checks and both Node suites are separate
  ordered shell commands with no condition, `continue-on-error`, fallback, or
  environment override. The progress record reports the exact commands green
  and a real Node panic sensitivity restored byte-identically.
- Prior workflow and release authority produced no finding. The full diff does
  not touch the wheel or publication workflows, release commands, sprint
  ledgers, or binding test sources. The complete 32-test workflow file reached
  31 passing tests in this review environment. Its remaining test was blocked
  before execution because the sandbox refused its temporary directory under
  the repository, not because an assertion failed.
- Contract, dependencies, HLD scope, panics, OOXML, and structure produced no
  additional finding. Both completed WASM wrappers are present in the branch,
  exactly the four plan-listed HLD files changed, and the story adds no parser,
  serializer, trait, generic, module, publication action, baseline change, or
  unapproved file. Prose, generated-skill sync, and diff hygiene passed.
