# F-027, rdocx adopts oxml-media

**Status**: completed
**Sprint**: S32.2
**Size**: M
**Depends on**: F-023, F-025, F-X005

## Problem

`Document` still owns image numbering state and suffix scanning at
`crates/rdocx/src/document.rs:50` and `crates/rdocx/src/document.rs:704`.
`store_image_part` trusts a filename extension for both the part suffix and
content type at `crates/rdocx/src/document.rs:726`. HTML and layout extraction
also guess media type from part names. The duplicate helpers at
`crates/rdocx/src/document.rs:2771` bypass the completed shared media owner.

The current hash harness covers three Word XML parts and page-one renders. It
does not hash content types, relationship targets, or media part names. A
correct sniffing cutover therefore needs a focused package regression while
the existing 28-entry harness remains unchanged.

## Spec reference

- `docs/hld/03-architecture.md`, media ownership and dependency direction.
- `docs/hld/04-opc-and-packaging.md`, media format resolution and naming.
- `docs/hld/11-migration-plan.md`, consumer cutover and behavior isolation.
- `docs/hld/12-testing-strategy.md`, hash-harness scope.
- `docs/hld/14-development-backlog.md`, "F-027, rdocx adopts oxml-media".

## Approach

After F-X005 publishes `oxml-media` 0.1.2, add it as a direct rdocx dependency.
Replace `image_counter` with `MediaNamer`, initialized by scanning existing
`/word/media/imageN` parts. Delete the local numbering, extension, and MIME
helpers.

Make `store_image_part` resolve the bytes before mutation, allocate the next
part with the canonical sniffed extension, register the canonical content
type, store the bytes, and return the relative relationship target. Use the
same byte-first resolution when constructing HTML and layout image inputs.
Keep the existing relationship and package APIs unchanged.

Add a focused regression using JPEG bytes named `.png`. It must prove the
`.jpeg` part, `image/jpeg` registration, and matching relationship target.
Retain every existing collision and sparse-suffix regression.

## Rejected alternatives

- Keep extension-only behavior. It produces package metadata that disagrees
  with the stored bytes.
- Retain a local counter beside `MediaNamer`. Two allocation authorities would
  recreate collision risk.
- Expand the baseline harness inside this behavior story. That would combine a
  baseline-scope change with the consumer cutover and still require a new
  deliberately mislabelled sample.
- Move image decoding into rdocx. Shared media already owns only the required
  format and header metadata boundary.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression, gate | mislabelled JPEG package test | JPEG bytes named `.png` produce `/word/media/image1.jpeg`, `image/jpeg`, and a `media/image1.jpeg` relationship target |
| regression | existing naming cases | Sparse suffixes, unrelated names, malformed suffixes, and loaded packages remain collision-safe |
| integration | HTML and layout media extraction | MIME values come from bytes rather than misleading part names |
| dependency | `cargo tree -p rdocx --edges normal` | The edge is `rdocx -> oxml-media` and the shared crate remains dependency-free |
| packaging | affected released-package dry-runs | Registry 0.1.2 resolves and archives verify below 10 MiB |

The backlog gate is the focused package-structure regression, collision-safe
naming, and an unchanged hash harness.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/11-migration-plan.md`
- `docs/hld/14-development-backlog.md`

Record the completed consumer boundary and replace the unobservable harness
delta claim with the exact package regression plus unchanged harness evidence.

## Risk routing

- Crate dependency graph and cross-family use. Confirm the new edge points into
  dependency-free `oxml-media` and no reverse edge exists.
- Public behavior in a published crate. Isolate the sniffed metadata change in
  the F-027 commit and prove it with exact package assertions.
- Public package verification. Resolve registry version 0.1.2, run affected
  dry-runs, and enforce archive-size limits.
- File move and helper deletion. Account for every removed helper and retain
  all existing naming regressions.

## Hash harness

Expected unchanged across all 28 entries. The focused package test owns the
intentional media metadata evidence because the harness does not collect it.

## Implementation checklist

- [x] Add the published shared media dependency.
- [x] Replace local numbering with scanned `MediaNamer` state.
- [x] Rewire storage and downstream MIME resolution to byte-first formats.
- [x] Delete all duplicate numbering, extension, and MIME helpers.
- [x] Add the exact mislabelled-image package regression.
- [x] Run focused, dependency, package, workspace, and hash gates.
- [x] Update exactly the four listed HLD files.

## Open questions

None. The existing harness scope determines unchanged hashes, and the focused
package regression determines the intended metadata behavior.
