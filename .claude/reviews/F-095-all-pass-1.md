# F-095, all, pass 1

**Reviewed**: working diff, 6 tracked files, 544 insertions and 19 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, the rendering risk rider has no endpoint raster evidence

`crates/rpptx-render/src/lib.rs:978`

The backlog gate proves the additional path structure, but it never sends an
arrowhead through the deterministic raster backend. The approved risk routing
requires generated in-memory paths and deterministic raster evidence. A
backend failure to fill the extra path would leave every F-095 test green.

## Smells

None.

## Nitpicks

None.

## Not found

No further correctness, contract, panic safety, OOXML, test, or structure
findings were found. Endpoint mapping stays source-neutral, missing and `none`
kinds are omitted, sizes default to medium, all five geometries are closed, and
non-finite or degenerate input is rejected without indexing or panicking.
