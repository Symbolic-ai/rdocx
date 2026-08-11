# F-130, all, pass 1

**Reviewed**: working implementation diff from claim base `ccbeb7a`, 18 files,
868 changed lines, with 855 additions and 13 deletions
**Verdict**: 3 defects, 0 smells, 0 nitpicks

## Defects

### D1, The documented path constructor is missing
`crates/rdocx-py/src/document.rs:32`
`docs/hld/10-bindings-spec.md:115`

The Python `Document` constructor accepts no arguments, while the binding spec's
source-compatible example opens an existing file with `Document("in.docx")`.
Calling that example against the built wheel raises `TypeError:
Document.__new__() takes 0 positional arguments but 1 was given`. The separate
`Document.open` static method does not preserve the promised documented API.
The package tests cover the zero-argument and `from_bytes` paths but do not cover
the documented path constructor.

### D2, Read-only run access invalidates both layout caches
`crates/rdocx-py/src/run.rs:74`
`crates/rdocx-py/src/run.rs:129`
`crates/rdocx/src/document.rs:452`

The run text getter and run collection length resolver take a mutable document
borrow and call `Document::paragraph_mut`. That facade method unconditionally
clears both layout caches before returning the paragraph. As a result, reading
`run.text`, calling `len(paragraph.runs)`, indexing or slicing the run
collection, and advancing its iterator can discard a cached render even though
none of those operations changes the document. The next render recomputes the
layout instead of sharing the immutable cached result required by the binding
contract.

### D3, F-130 implements an incomplete exception surface owned by F-132
`crates/rdocx-py/python/rdocx/__init__.py:4`
`crates/rdocx-py/src/lib.rs:17`
`crates/rdocx-py/src/document.rs:25`
`docs/hld/14-development-backlog.md:1015`

F-132 owns the package exception hierarchy and mapping from shared Rust binding
errors. F-130 instead defines `StaleElementError` directly under `Exception`
and adds its native mapper, while every other `rdocx::Error` is still collapsed
to `RuntimeError`. The installed package therefore has no `RdocxError` base and
does not provide the approved hierarchy, despite the design checklist marking
the error surface complete. This also makes the nominally parallel F-130 and
F-132 work edit the same package and mapping boundary. The ownership or feature
sequencing must be resolved before integration.

## Smells

None.

## Nitpicks

None.

## Not found

- Correctness: apart from D1 and D2, revision bumps occur only after successful
  structural mutations. Failed removal preserves live handles, text mutation
  does not stale paths, and the exact shared stale message and recovery hint
  survive Python mapping.
- PyO3 safety: handles and collections retain `Py<PyDocument>` plus owned paths.
  No Rust facade borrow escapes a method call, and no unsafe ownership or borrow
  conversion was found.
- Lazy collections: paragraph and run collections do not retain facade `Vec`
  snapshots. Positive and negative indexes, forward and reverse slices, and
  iterators behaved correctly against the installed wheel.
- Packaging: the mixed-layout package imported from a built wheel, the wheel
  was tagged `cp39-abi3`, and the extension feature remains off by default.
  `cargo check -p rdocx-py --all-targets`, binding clippy, and all four focused
  Python tests passed.
- Public Rust API: the new facade accessors are additive and return `None` for
  absent indexes. The complete `rdocx` test suite and the focused totality gate
  passed.
- HLD and release metadata: the approved HLD impact includes HLD15, the new
  unpublished package carries release-family metadata, the workspace-version
  count is ten, and the focused release regression passed.
- F-133 scope: no rendering method was added. Releasing the GIL around
  `to_bytes` matches the existing threading specification for that serialization
  path and does not introduce F-133's concurrent rendering surface.
- Hash harness: all 28 entries matched, so the declared unchanged expectation
  holds.
- Panics: no reachable panic, unchecked collection index, or invalid slice-step
  loop was found in the binding paths.
- OOXML: this diff adds no parser, serializer child ordering, namespace, or raw
  subtree handling.
- Structure and artifacts: the approved crate and files contain no unjustified
  trait, generic, dynamic dispatch, forwarding wrapper, or extra feature flag.
  No generated wheel, extension binary, or cache artifact is part of the diff.
- Other gates: rustfmt, prose checking, generated-skill synchronization, the
  no-default layout suite, and the rdocx WASM target check passed.
