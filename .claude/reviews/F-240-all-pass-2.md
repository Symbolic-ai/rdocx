# F-240, all aspects, pass 2

**Reviewed**: working diff from `539f37a1`, 8 files, 584 insertions and 35 deletions
**Verdict**: 3 defects, 0 smells, 0 nitpicks

## Defects

### D1, partial and unsupported classifications are not structurally enforced

`scripts/test_sprint_workflow.py:8050`

Pass 1 added a consistency rule for `complete`, but the inverse remains open.
A `partial` row can contain only complete cells, and an `unsupported` row can
claim creation is available, without failing the classification gate. The test
must prove that partial rows contain at least one partial or unavailable
applicable cell and that unsupported rows cannot claim creation.

### D2, duplicate placement outside S70 through S80 is invisible

`scripts/test_sprint_workflow.py:8111`

The placement scan is limited to the expected sprint range. If F-243 is also
placed in S81 or another sprint, the collected S70 through S80 placement still
has length one and the test passes. One sprint placement must be proved over the
whole canonical sprint plan before checking that its value is S70 through S80.

### D3, a private DOCX tracked outside the configured directory passes

`scripts/test_sprint_workflow.py:8217`

The privacy regression checks only tracked paths below
`corpus/private-docx`. Moving or adding one of the private binary documents at
another path would evade this assertion. This repository has no tracked DOCX
fixtures and constructs test fixtures in code, so the regression can reject any
tracked DOCX path without weakening an existing exception.

## Smells

None.

## Nitpicks

None.

## Not found

The three pass 1 defects are resolved. No additional correctness, contract,
panic, OOXML, test, or structure findings were found outside the gate gaps
listed above.
