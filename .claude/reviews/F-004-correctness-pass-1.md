# F-004, correctness, pass 1

**Reviewed**: F-004 working tree, 3 files, 262 changed lines
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Not found

Correctness issues were not found. The Apache 2.0 text matches the Apache
Software Foundation source exactly. The Caladea notice matches the copyright,
trademark, designer, and licence metadata embedded in the four bundled TTFs.
Those TTFs match the archived ChromeOS source byte for byte.

The test gate derives every distinct family from `bundled_font_data`, compares
that complete set with the family-to-licence mapping, and verifies each mapped
licence file exists. The focused gate passes. The package listing includes both
`fonts/LICENSE-Caladea` and `fonts/NOTICE-Caladea`.
