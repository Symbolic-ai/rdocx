# F-132, all, pass 1

**Reviewed**: working implementation diff from claim base `321ddce`, 10 files,
429 changed lines, with 399 additions and 30 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, The layout-error mapping has no sensitivity gate
`crates/rdocx-py/src/lib.rs:40`
`crates/rdocx-py/tests/test_shared.py:112`

The native classifier maps `rdocx::Error::Layout` to the public `LayoutError`
class, but the integration test exercises only package, XML, and stale errors.
Changing the layout arm to `RdocxError`, `PackageError`, or `XmlError` leaves all
11 binding tests green. This means the promised concrete layout mapping is not
protected by the F-132 test plan, even though F-132 owns that mapping before
F-133 adds a public rendering entry point. Add a focused classifier or binding
regression that fails when the layout arm names any other public class.

## Smells

None.

## Nitpicks

None.

## Not found

- Correctness: the unit constructors use the exact approved EMU factors and
  truncate positive and negative fractional inputs toward zero. All six unit
  properties match the canonical values, including negative twip truncation.
  `RGBColor` is an immutable three-channel tuple with checked channel bounds,
  hexadecimal parsing, and uppercase string output.
- Enums and namespaces: all four approved types are `IntEnum` classes with the
  exact bounded member names and integer values. Each carries documentation and
  is available from both its compatibility module and the package top level.
- Exception behavior: the hierarchy is rooted at `RdocxError`. OPC, I/O,
  missing-part, OXML, stale-domain, and layout variants have explicit native
  classifier arms. Installed-wheel tests proved the package, XML, and stale
  paths. No fallback to a generic runtime error occurs on those package imports.
- Contract and dependencies: the implementation consumes the integrated F-129
  revision domain and F-130 package facade without taking F-131 or F-133 scope.
  No runtime or test dependency on python-docx was introduced.
- Packaging and ABI: the built wheel is tagged `cp39-abi3`, declares Python 3.9
  or newer, and contains the extension plus every approved pure-Python package
  module. The installed-wheel suite passed all 11 F-130 and F-132 tests.
- HLD discipline: only HLD10 and HLD14 changed, exactly matching the approved
  impact list. HLD01 remains the unit authority and HLD15 remains consistent
  with the unchanged abi3 and release metadata.
- Canonical behavior and hashes: no canonical Rust `Length`, OOXML serializer,
  rendering source, or hash baseline changed. The baseline remains at SHA-256
  `a9fc6891c826fb1022cb0de846cc947e4a3b2017383cbd6ba6c9fac4e99c3f85`, and
  the worker evidence records all 28 entries matching after implementation.
- Tests: apart from D1, the named gate fails when the new package exports are
  absent, and the length, enum, hierarchy, package, XML, and stale regressions
  exercise their claimed behavior. The focused binding tests, canonical
  conversion regression, `rdocx-py` check and clippy, rustfmt, prose check,
  generated-skill check, and `rdocx-wasm` target check passed.
- Panics and PyO3 safety: no new reachable panic, unchecked index, unsafe code,
  escaped borrow, or GIL misuse was found. Error construction happens while
  Python is attached, after detached serialization returns.
- OOXML: the diff changes no parsing, namespace, schema ordering, whitespace,
  preservation, or serialization behavior.
- Structure and artifacts: the approved package modules and dedicated existing
  test entry add no trait, generic, dynamic dispatch, wrapper layer, or feature
  flag. No wheel, extension binary, cache directory, or compiled Python file is
  present in the worktree.
