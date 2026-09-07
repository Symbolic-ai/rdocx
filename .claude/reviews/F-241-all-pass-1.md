# F-241, all aspects, pass 1

**Reviewed**: working diff from `0b082b3670c5d6cab67737df3e130bd1f0772275`, 7 files, 954 insertions and 18 deletions
**Verdict**: 3 defects, 0 smells, 0 nitpicks

## Defects

### D1, a non-object private manifest escapes the sanitized failure path

`scripts/docx_authoring_conformance.py:455`

`load_private_manifest` calls `set(payload)` before proving that the decoded
JSON value is an object. A JSON array containing an object raises `TypeError`
instead of `ConformanceError`, so required-private mode emits an unsanitized
traceback rather than failing closed with the declared manifest error.

### D2, normalized relationship records discard relationship IDs

`scripts/docx_authoring_conformance.py:244`

The parser validates each relationship ID but omits it from the normalized
record. A candidate that changes only an ID can therefore compare equal even
though document XML still refers to the original ID. The normalized package
graph must retain the ID so an unresolved relationship rewrite cannot pass.

### D3, the public dependency check permits a private package alias

`scripts/docx_authoring_conformance.py:340`

The check accepts any sole dependency line that starts with `rdocx = `. A line
such as `rdocx = { package = "rdocx-oxml", ... }` satisfies that test even
though the temporary consumer depends on a private crate. The boundary must
parse the manifest and require the exact public package and expected path.

## Smells

None.

## Nitpicks

None.

## Not found

No additional contract, panic, OOXML, test, or structure findings were found.
The new file is explicitly approved by the design plan, introduces no trait,
generic, crate, feature, or production dependency, and preserves the 49-entry
hash baseline.
