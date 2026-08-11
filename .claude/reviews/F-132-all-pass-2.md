# F-132, all, pass 2

**Reviewed**: working implementation diff from claim base `321ddce`, 12 files,
467 changed lines, with 437 additions and 30 deletions
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, The completed implementation checklist is wholly unticked
`.claude/plans/F-132-design.md:96`

All six implementation items remain unchecked at
`.claude/plans/F-132-design.md:98`, even though the progress record says the
implementation and remediation are complete. `/complete-feature` requires
every checklist item to be ticked before it can prepare the worker handoff.
The current plan therefore cannot pass its completion precondition.

### D2, The remediation's direct dev dependency is absent from risk routing
`crates/rdocx-py/Cargo.toml:34`
`.claude/plans/F-132-design.md:82`

The new exact classifier test names `oxml_layout::LayoutError` directly at
`crates/rdocx-py/src/lib.rs:78`, which adds a direct dependency edge from the
Word binding crate to the format-neutral layout crate. The edge is dev-only,
points inward, and does not violate the dependency rule. It is also the
smallest available way to construct the public `rdocx::Error::Layout` variant.
However, a new direct use across crate families triggers the crate dependency
graph risk row. The approved plan records unit conversion, PyO3, and new-file
routing only. It does not record the dependency check, HLD03 reading, or normal
and dev tree inspection earned by the remediation. Revise the plan and progress
record to include that routed obligation. No additional HLD impact is needed
because the dev-only edge follows the existing inward dependency rule.

## Smells

None.

## Nitpicks

None.

## Pass 1 finding re-evaluated

- D1 is resolved. The private Rust test constructs the concrete layout variant
  at `crates/rdocx-py/src/lib.rs:78` and checks exact identity with the public
  `LayoutError` class at `crates/rdocx-py/src/lib.rs:83`. The progress record at
  `.claude/scratch/F-132-progress.md:23` documents a source-hash-preserving
  mutation check in which mapping the variant to `PackageError` made this test
  fail. `cargo test -p rdocx-py` passes the restored classifier.

## Not found

- Length and integer semantics: exact EMU factors are defined at
  `crates/rdocx-py/python/rdocx/shared.py:14`. All float constructors use
  truncating `int(value * factor)` operations at
  `crates/rdocx-py/python/rdocx/shared.py:57`, and negative twip conversion
  truncates toward zero at `crates/rdocx-py/python/rdocx/shared.py:4`. The
  focused Python tests and canonical Rust conversion regression passed.
- RGB behavior: `RGBColor` is an immutable tuple with bounded integer channels
  at `crates/rdocx-py/python/rdocx/shared.py:92`, checked six-digit hexadecimal
  parsing at `crates/rdocx-py/python/rdocx/shared.py:101`, and uppercase output
  at `crates/rdocx-py/python/rdocx/shared.py:112`.
- Enum literals, documentation, and imports: the bounded text inventory is at
  `crates/rdocx-py/python/rdocx/enum/text.py:6`, and the table inventory is at
  `crates/rdocx-py/python/rdocx/enum/table.py:6`. Compatibility-module identity,
  exact members, integer values, and class documentation are asserted at
  `crates/rdocx-py/tests/test_shared.py:58`. Top-level exports are complete at
  `crates/rdocx-py/python/rdocx/__init__.py:30`.
- Exception hierarchy and mappings: all four public errors inherit from
  `RdocxError` at `crates/rdocx-py/python/rdocx/__init__.py:8`. OPC, I/O,
  missing-part, image-dimension, XML, layout, and fallback variants have
  explicit classifier arms at `crates/rdocx-py/src/lib.rs:33`. Stale-domain
  errors retain their dedicated mapping at `crates/rdocx-py/src/lib.rs:29`.
  Installed-wheel checks proved package, I/O, XML, and stale behavior, while
  the repaired private Rust test proves the otherwise inaccessible layout arm.
- PyO3 and ABI packaging: error conversion remains attached to Python after
  detached serialization at `crates/rdocx-py/src/document.rs:59`. A current
  wheel built as `cp39-abi3`, contained the extension and all five approved
  pure-Python files, and passed all 11 F-130 and F-132 Python tests.
- HLD discipline: only `docs/hld/10-bindings-spec.md:137` and
  `docs/hld/14-development-backlog.md:1015` changed. They record the implemented
  bounded surface, pure-Python ownership, concrete mapping, and real F-130
  dependency. No change to HLD01 or HLD15 is required.
- Scope and artifacts: no F-131 formatting, F-133 rendering, external oracle,
  new trait, generic, feature flag, runtime dependency, compiled Python file,
  extension binary, wheel, or cache directory is part of the working diff.
- Panics and OOXML: no production panic, unsafe code, escaped PyO3 borrow,
  parser change, serializer order change, namespace change, or raw XML
  preservation change was found.
- Gates: the exact layout classifier test, all 11 installed-wheel tests,
  `cargo check -p rdocx-py --all-targets`, binding clippy, the canonical unit
  truncation regression, the rdocx WASM check, normal and dev dependency-tree
  inspection, and the current-wheel ABI and inventory checks passed.
