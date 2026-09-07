# F-X085, correctness, pass 1

**Reviewed**: working-tree diff against `4152ec2a`, 5 files, 365 changed lines
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, the gate counts memo requests rather than identity serializations

`crates/rdocx-layout/src/engine.rs:2508`

The layout metric copies `restart_identity_memo.computations`, which increments
only when the memo computes a slot. If a scan regresses to calling
`restart_body_identity` directly while publication still uses the memo, the
at-most-once regression can remain at or below 715 even though serialization
again exceeds the body length. Instrument the serialization boundary itself and
record that count for the completed layout.

## Smells

None.

## Nitpicks

None.

## Not found

Contract, panics, OOXML, and structure produced no findings. The fingerprint
check precedes serialization, each current body index owns one tri-state slot
across scans and publication, unserializable identities remain failed values,
and moved identity bytes retain the existing restart entry accounting. The diff
adds one concrete private helper with one current use and no public API, trait,
generic parameter, module, file, dependency, or feature flag.
