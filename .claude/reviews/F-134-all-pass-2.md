# F-134, all, pass 2

**Reviewed**: complete current working diff from claim base `af239048`, 13
files and 754 added plus 17 removed lines, including the pass-1 review and six
approved implementation files
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Not found

Pass-1 D1 is resolved. `ShapeCollection.add_shape()` now requires the bounded
`MSO_SHAPE` at `crates/rpptx-py/python/rpptx/_rpptx.pyi:101`, and `Font.size`
returns the public `Length` at line 188. The smoke program asserts the returned
unit type and its `.pt` and `.emu` properties at
`crates/rpptx-py/tests/typing_smoke.py:38`, and its callable assignment at line
41 proves the enum input remains narrower than arbitrary integers. The live
wheel returned `rpptx.util.Length`, accepted `MSO_SHAPE.CHEVRON`, and rejected
raw value 50. Independently broadening both annotations made exact strict mypy
fail with the expected assignment and unused-ignore errors.

Pass-1 D2 is resolved. `RGBColor` now has the exact three-integer tuple base at
`crates/rdocx-py/python/rdocx/shared.py:92`, and its smoke checks indexed and
whole-tuple types at `crates/rdocx-py/tests/typing_smoke.py:33`. Exact
`mypy==2.3.0 --strict` passed all 11 inline Python and native-stub sources
extracted from the installed wheels. Reverting only the tuple parameterization
in a disposable copy reproduced the strict missing-type-arguments failure.

Pass-1 D3 is resolved. Every non-root native handle and collection has a
required private `NoReturn` constructor argument, beginning with rdocx
`Paragraph` at `crates/rdocx-py/python/rdocx/_rdocx.pyi:40` and rpptx
`SlideLayout` at `crates/rpptx-py/python/rpptx/_rpptx.pyi:30`. The two smokes
carry unused-ignore-sensitive calls for all 30 non-root types at
`crates/rdocx-py/tests/typing_smoke.py:56` and
`crates/rpptx-py/tests/typing_smoke.py:67`. Removing one private constructor in
a disposable stub made that exact gate fail. Runtime enumeration confirmed all
13 rdocx and 17 rpptx non-root classes reject direct construction, while
`Document()` and `Presentation()` remain constructible both statically and in
the live modules.

No fresh correctness, contract, optional-value, enum-literal, unit-signature,
path-like input, byte-output, lazy collection, integer and slice overload,
iterator, exception hierarchy, runtime-member, Python 3.9 syntax, package
version, release metadata, dependency, WASM isolation, HLD-impact,
speculative-surface, approval, formatting, prose, generated-skill,
diff-hygiene, hash-expectation, or artifact issue was found. The HLD changes
remain exactly HLD10, HLD12, and HLD14, and the six new implementation files
remain exactly those approved by the plan.

The source stubs and remediated inline source were byte-identical to the fresh
wheel copies. Both wheels contained their zero-byte `py.typed` markers and
carried exact `cp39-abi3-macosx_11_0_arm64` tags. Exact strict mypy passed both
installed-wheel smokes, combined stubtest passed all 11 modules, and all 41
binding tests passed. Independent enum, return, inline-source, constructor,
and member mutations made their intended gates fail. Both binding crate checks
and the existing `rdocx-wasm` target passed. Formatting, prose, skill sync,
diff hygiene, and artifact checks passed. The worker's unchanged hash evidence
records all 28 entries matching.
