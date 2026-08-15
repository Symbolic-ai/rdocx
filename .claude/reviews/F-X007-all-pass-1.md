# F-X007, all, pass 1

**Reviewed**: `2f469e7` through `cd1e6e1` plus the working tree, 31 files and
1,134 changed or new lines. The scope is 25 tracked files with 862 insertions
and 150 deletions, plus six new crate READMEs with 122 lines.
**Verdict**: 8 defects, 0 smells, 0 nitpicks

## Defects

### D1, numbering edits delete unmodelled OOXML

`crates/rdocx-oxml/src/numbering.rs:101`
`crates/rdocx-oxml/src/numbering.rs:215`
`crates/rdocx-oxml/src/numbering.rs:281`
`crates/rdocx-oxml/src/numbering.rs:365`
`crates/rdocx/src/document.rs:292`

The numbering readers skip unknown level children, unknown abstract-number
children, `w:lvlOverride`, and unknown root children. `flush_to_package` then
serializes the incomplete typed model over the original numbering part. Open a
producer document whose numbering contains `w:nsid`, `w:tmpl`,
`w:lvlOverride`, or an extension, call either new list mutation, and save. The
unmodelled content is deleted. This violates the repository's verbatim
preservation rule and the parser or serializer risk rider. The added round-trip
tests start from rdocx-generated model data, so they cannot detect this loss.

### D2, list identifier allocation can panic or collide at the finite boundary

`crates/rdocx-oxml/src/numbering.rs:419`
`crates/rdocx-oxml/src/numbering.rs:429`
`crates/rdocx-oxml/src/numbering.rs:456`

Both next-ID helpers compute `max + 1` on parsed `u32` identifiers. A document
with `u32::MAX` as either maximum reaches the new `add_list` path and panics in
checked builds. An unchecked build wraps and can reuse an occupied identifier.
The public authoring API therefore is not non-panicking on a validly parsed
input boundary and does not preserve list identity.

### D3, the nine-level contract silently discards supplied definitions

`crates/rdocx/src/document.rs:1401`
`crates/rdocx-oxml/src/numbering.rs:463`

`Document::add_list_definition` copies the caller's complete slice, but
`CT_Numbering::add_list` reads only indices zero through eight. A tenth and
later `ListLevel` is silently ignored even though the public method does not
state a maximum or report rejection. The unnecessary full-slice copy also
makes work proportional to data that can never be represented. Enforce the
nine-level bound or document and test an explicit bounded policy before this
becomes a released API.

### D4, paragraph numbering accepts levels no list can define

`crates/rdocx/src/paragraph.rs:230`
`crates/rdocx/src/paragraph.rs:235`

The paragraph setters accept every `u32` level and store it directly. Passing
level 9 or greater succeeds and serializes an `ilvl` for which the custom-list
surface can never create a definition. `Document::set_list_level` correctly
rejects the same input. The paragraph API needs the same bound and a failure
signal, or a contract that prevents emitting an unresolved list level.

### D5, fixed column mutation accepts negative widths

`crates/rdocx/src/table.rs:173`
`crates/rdocx/src/table.rs:215`

The staged overflow check does not reject a negative requested width. When the
other columns keep the total in range, `set_column_width` returns true and
writes the negative value into `w:gridCol`, then derives table and cell widths
from that invalid geometry. The method must reject a negative column width
without mutation, with a regression covering the unchanged table.

### D6, the published CLI examples do not match the command parser

`crates/rdocx-cli/README.md:13`
`README.md:220`
`README.md:223`
`README.md:227`
`crates/rdocx-cli/src/main.rs:39`
`crates/rdocx-cli/src/main.rs:63`

Every shown `convert` command omits the required `--to` argument. The root
README's `replace` command uses `--find` and `--replace`, while the binary
accepts `--placeholder` and `--value`. The convert example was reproduced
against Clap and exits before opening the file with a missing `--to` error.
The README runner ignores shell fences, so its green result does not cover
these published examples.

### D7, the root installation snippet names a nonexistent feature

`README.md:55`
`crates/rdocx-layout/Cargo.toml:19`

The root package README tells users to enable `rdocx-layout/bundled-fonts`, but
the crate exposes only the default `system-fonts` feature. Copying the
installation block makes Cargo fail feature resolution. The documentation
needs to describe the actual always-available deterministic path and the real
feature switch.

### D8, both shim examples contradict their dependency blocks

`crates/rdocx-opc/README.md:12`
`crates/rdocx-opc/README.md:20`
`crates/rdocx-pdf/README.md:10`
`crates/rdocx-pdf/README.md:18`
`scripts/readme_doctests.py:175`

The deprecated-shim examples import `rdocx_opc` and `rdocx_pdf`, while their
adjacent dependency blocks add only `oxml-opc` and `oxml-pdf`. A user copying
each complete example cannot resolve the imported crate. The doctest runner
masks the mismatch by injecting the shim artifact directly with `--extern`.
Show a matched legacy dependency and import, or a matched migrated dependency
and import, and make the compiled example prove that pairing.

## Smells

None.

## Nitpicks

None.

## Not found

- Correctness: no further defect was found in hyperlink relationship ownership,
  hard-break construction, custom format selection, or staged table updates.
- OOXML: child order for the new hyperlink, break, numbering, and table output
  is correct apart from the preservation defect above.
- Panics and resources: no further reachable panic, unchecked width sum, span
  overflow, or partial table mutation was found.
- Tests: the focused numbering and table tests pass. All twelve Rust README
  examples compile with warnings denied. The missing coverage is stated in the
  findings above.
- Packaging: each of the seven stable package inventories contains exactly one
  intended README, and the manifest wiring matches those inventories.
- Contract: PR 25 is merged into `sprint/s38` at GitHub merge commit `6aade64`,
  and the public note credits Jon Stokes as `@jonstokes`.
- HLD: the implementation touches only the approved HLD04, HLD10, HLD12,
  HLD14, and HLD15 scope. No additional HLD owner was found.
- Structure: no unjustified trait, generic, wrapper, crate, module, or feature
  flag was introduced. The new README files were explicitly requested.
