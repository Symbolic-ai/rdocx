# Current Sprint, S85

**Milestone**: X, hosted CI repair and contribution closure.

**Goal**: restore the required hosted CI gate on the S84 main result and
reconcile the 32 reviewed PRs and nine issue contracts after that gate passes.
The published `s84` tag stays fixed. Planned M24 feature work starts in S86.

## Spec references

- `docs/hld/12-testing-strategy.md`, for the pinned CI toolchain, source-built
  Python suite, reporter deck fixture and criterion-level closure evidence.
- `docs/hld/14-development-backlog.md`, for F-X171's hosted integration gate.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-X171 | Pin LibreOffice for macOS Python acceptance | S | done | - |

## Sequencing note

F-X171 follows completed F-X168. First repair the macOS CI job and its
workflow assertion. Then verify the integrated branch, review it, merge through
`/close-sprint`, wait for a green hosted `main` gate, and post the prepared
individual PR and issue dispositions. Issue 158 closes last.

## Definition of done for this sprint

- The macOS presentation Python job downloads a SHA-256-pinned LibreOffice
  26.2.5 image, verifies build 26.2.5.2 and runs all binding tests with the
  pinned `soffice` available. The Linux pinned-viewer jobs remain green.
- The workflow assertion detects a missing or bypassed macOS viewer setup.
- The full integrated local gate, hash harness and sprint review pass, then
  the required hosted CI gate passes on pushed `main`.
- Each of the 32 S84 PRs and nine issues receives its specific contributor or
  reporter note and closes only after its full criteria hold on `main`.
