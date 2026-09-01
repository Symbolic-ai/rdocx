# F-222 correctness pass 2

## Verdict

Ready. Found 0 defects, 0 smells, and 0 nitpicks.

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Pass 1 remediation

- D1 is fixed by charging worst-case escaped text before allocation and checking
  the XML cap after every shape and slide at
  `crates/rpptx/src/odp.rs:1120-1142` and
  `crates/rpptx/src/odp.rs:1386-1421`. The oversized-text regression fails
  closed before publication.
- D2 is fixed by classifying every direct Drawing or Presentation namespace
  child at `crates/rpptx/src/odp.rs:806-815`. The regression covers one stable
  import diagnostic and 10,002 safe unsupported nodes exhausting the 10,000
  diagnostic ceiling.

## Evidence

- Four ordinary F-222 integration tests pass. The pinned LibreOffice test is
  intentionally ignored in the ordinary suite.
- The pinned LibreOffice 26.2.5.2 two-direction structural and PDF record test
  passes when run explicitly outside the sandbox.
- `cargo clippy -p rpptx --all-targets --all-features -- -D warnings` passes.
- The hash harness remains 49 of 49 unchanged.
- `git diff --check` and the prose checker pass.
