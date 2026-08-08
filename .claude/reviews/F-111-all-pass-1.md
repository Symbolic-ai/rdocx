# F-111, all, pass 1

**Reviewed**: uncommitted working diff against `HEAD`, 3 files, 553 additions and 2 deletions
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Not found

Correctness produced no findings. Slide lookup, dimension calculation, shape-id
allocation, media insertion, relationship creation, picture construction, and
append occur in an order that keeps every fallible step ahead of mutation to
the live presentation. Package and media changes are staged in clones. The
live package, `MediaStore`, relationships, and shape tree are committed only
after picture construction succeeds, and nothing fallible remains after the
first live assignment.

Dimension inference produced no findings. Probed raster dimensions are
nonzero, so the aspect-ratio divisor cannot be zero. Multiplication widens an
`i64` EMU value and a `u32` pixel value to `i128`, which covers their complete
product range. Integer division truncates toward zero and the final conversion
to `i64` is checked. Both omitted dimensions use the existing 72-DPI native
size contract. Both supplied dimensions bypass probing as approved. Negative
extents are rejected by staged transform reparsing before commit, while zero
extents remain accepted by the nonnegative DrawingML size model.

Media and relationship handling produced no findings. Format resolution sniffs
bytes before considering the filename, and newly inserted parts use the
sniffed canonical extension and MIME type. Content-hash comparison includes a
full byte equality check. Equal bytes share one package media part across
slides. Relationship reuse is restricted to an internal image relationship in
the target slide's own scope whose resolved target is exactly that media part.
New targets are written relative to the source slide part.

Contract produced no findings. `Presentation::add_picture` has the approved
owning-facade signature and returns a borrow of the exact appended picture.
Rust keeps the mutable presentation borrow active for the returned
`ShapeRef`, so later structural mutation cannot invalidate the reference.
Deterministic names derive from the tree-wide allocated id.

Panics and overflow produced no findings. The validated slide index remains
valid because the method does not resize the slide vector. The final index and
append are therefore guarded by a local invariant. Probe output guarantees
positive pixel axes, widened arithmetic prevents intermediate overflow, and
the narrowing conversion reports an error.

OOXML produced no findings. `CT_Picture::new` emits fixed `p:`, `a:`, and `r:`
prefixes with `p:nvPicPr`, `p:blipFill`, and `p:spPr` in schema order. The
non-visual shell contains `p:cNvPr`, picture locks, and `p:nvPr`. The blip fill
uses the slide relationship id, stretch fill, and a typed transform. The
constructor reparses its own output. Tree append reuses the raw-boundary-aware
helper, and id allocation reuses the complete namespace-resolved tree scan.

Tests produced no findings. The gate proves exact native dimensions after save
and reopen. Aspect-ratio tests cover both inferred axes and truncation. The
suite also covers cross-slide media deduplication, same-slide relationship
reuse, misleading extensions, explicit-size probe bypass, raw trailing
content, contextual failures with byte-identical state, validation, and native
acceptance. `cargo test -p rpptx --test integration` passed with 39 tests and 4
ignored tests. The focused `rpptx-oxml` constructor test passed.

Both ignored oracle helpers were invoked explicitly. The python-pptx 1.0.2
comparison passed with matching `PICTURE` kind and 12,700 by 12,700 EMU bounds.
The native gate passed against Microsoft PowerPoint 16.104 build
16.104.25121423 without repair. `cargo fmt --all --check` and
`git diff --check` passed.

Structure produced no findings. No new trait, generic parameter, module, file,
feature, dependency, forwarding wrapper, or erased concrete type was added.
