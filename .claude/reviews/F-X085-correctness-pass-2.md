# F-X085, correctness, pass 2

**Reviewed**: remediated working-tree implementation diff against `4152ec2a`, 5 files, 383 changed lines
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Not found

Correctness, contract, panics, OOXML, tests, and structure produced no further
findings. Pass 1 defect D1 is resolved at the serialization boundary. A
test-only thread-local counter now observes every `restart_body_identity` call
made during a layout, including a direct call that bypasses the memo. The
layout resets that counter before each transaction and records it immediately
afterward. The tri-state memo, fingerprint prefilter, exact-byte comparison,
publication move, unserializable path, transient metrics, and persistent cache
accounting remain intact.
