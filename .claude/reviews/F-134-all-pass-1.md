# F-134, all, pass 1

**Reviewed**: complete working diff from claim base `af239048`, 11 files and
568 added plus 16 removed lines, including the six approved untracked files
**Verdict**: 3 defects, 0 smells, 0 nitpicks

## Defects

### D1, the rpptx stub erases the public enum and returned unit types
`crates/rpptx-py/python/rpptx/_rpptx.pyi:92`
`crates/rpptx-py/python/rpptx/_rpptx.pyi:172`

`ShapeCollection.add_shape()` declares its bounded `MSO_SHAPE` argument as an
arbitrary integer, and `Font.size` declares its result as `int | None` even
though the live getter deliberately constructs `rpptx.Length` at
`crates/rpptx-py/src/text.rs:472`. The former accepts unsupported integers such
as 50 at type-check time before the live method rejects them at
`crates/rpptx-py/src/shape.rs:341`. The latter rejects valid typed code which
uses `.pt` on a returned size or assigns that result to `Length | None`. An
independent strict check reproduced the latter failure while the live getter
returned `rpptx.util.Length`. The smoke program writes a `Pt` at
`crates/rpptx-py/tests/typing_smoke.py:26` and passes an `MSO_SHAPE` at line 28,
but never asserts either semantic type, so both broadenings remain green.

### D2, the inline-typed rdocx package does not pass strict mypy
`crates/rdocx-py/python/rdocx/shared.py:92`

`RGBColor` subclasses unparameterized `tuple`, so exact `mypy==2.3.0 --strict`
over the installed package sources reports `Missing type arguments for generic
type "tuple"`. The current positive smoke merely imports selected public names
at `crates/rdocx-py/tests/typing_smoke.py:3`, which makes mypy trust rather than
strictly check the installed inline-typed implementation. `stubtest` also does
not type-check implementation bodies. The advertised contract says the
pure-Python units remain inline typed at `docs/hld/10-bindings-spec.md:210`, but
the gate can therefore report green while that part of the marked package is
not strict and tuple element access degrades to `Any`.

### D3, the stubs advertise constructors that the native handles do not have
`crates/rdocx-py/python/rdocx/_rdocx.pyi:39`
`crates/rpptx-py/python/rpptx/_rpptx.pyi:26`

Every handle and collection class other than `Document` and `Presentation`
omits a constructor declaration. Mypy consequently supplies the permissive
object constructor and accepts calls such as `Paragraph()` and `Slide()` with
no errors, while both live extension types raise `TypeError: cannot create
... instances`. These objects are created only by package-internal factories,
for example `PyParagraph::new` at `crates/rdocx-py/src/paragraph.rs:63` and the
slide collection factory at `crates/rpptx-py/src/slide.rs:61`. The positive
smoke programs obtain handles from their roots, and successful `stubtest` does
not detect the false constructors, so the installed-wheel gate misses this
surface drift across both packages.

## Smells

None.

## Nitpicks

None.

## Not found

No additional wheel-inventory, marker placement, path-like input, byte output,
optional-value, collection overload, slice, iterator, runtime-member,
exception hierarchy, enum literal, package version, abi3 tag, dependency,
WASM isolation, HLD-impact, release metadata, speculative surface, formatting,
prose, generated-skill, diff-hygiene, hash expectation, or artifact findings
were found. The six new files exactly match the approvals in the plan, and the
only HLD changes are the listed HLD10, HLD12, and HLD14 files.

Both source stubs were byte-identical to the freshly installed wheel copies.
The wheels contained their zero-byte `py.typed` markers and carried exact
`cp39-abi3-macosx_11_0_arm64` tags. Exact mypy 2.3 strict checks passed the two
current smoke programs, stubtest passed all 11 installed modules, and the
installed binding suite passed all 41 tests. Independent member deletion made
stubtest fail and a `Presentation.to_bytes` return mutation made strict mypy
fail, confirming the documented representative mutations. Both binding crate
checks and the existing `rdocx-wasm` target passed using an external target
directory. Formatting, prose, skill sync, and diff checks passed. The worker's
unchanged hash evidence records all 28 entries matching.
