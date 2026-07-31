# F-036, MediaId

**Status**: approved
**Sprint**: S07
**Size**: S
**Depends on**: F-029

## Problem

The staged renderer model still keys images with relationship identifiers.
`crates/oxml-layout/src/output.rs:113` stores an optional `embed_id`, while
`crates/oxml-layout/src/line.rs:85` and `crates/oxml-layout/src/line.rs:135`
store the same relationship-local string in inline and laid-out images. That
assumes one relationship namespace and cannot identify the same bytes reused
by different presentation parts.

## Spec reference

- `docs/hld/08-rendering-spec.md`, "Extending `PositionedElement`" and "Four
  latent defects to fix".
- `docs/hld/11-migration-plan.md`, "Order of operations" and "Preserve
  behaviour, do not improve it".
- `docs/hld/12-testing-strategy.md`, "oxml-layout".
- `docs/hld/14-development-backlog.md`, "F-036, MediaId".

## Approach

Define the content-addressed handle in `output.rs` and export it from the crate
root:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MediaId(pub u64);

impl MediaId {
    pub fn from_bytes(bytes: &[u8]) -> Self;
}
```

Use a fixed 64-bit FNV-1a calculation over the raw bytes so the identifier is
stable across processes and platforms without adding a dependency. Document
that the handle is a renderer key rather than a collision-free content
guarantee.

Replace `embed_id` only in staged `oxml-layout` image representations:

```rust
PositionedElement::Image { media_id: MediaId, .. }
InlineItem::Image { media_id: MediaId, .. }
LineItem::Image { media_id: MediaId, .. }
```

Preserve the image bytes and content type in `PositionedElement::Image`.
Preserve `MediaId` through inline-to-line conversion. Do not add a media store
or change the released `rdocx-layout` copy.

## Rejected alternatives

- Keep relationship strings beside `MediaId`. That preserves the invalid
  renderer key and creates two identities for one image.
- Add a media store now. The staged input and renderer ownership for such a
  store belongs to later migration stories.
- Use the standard library's default hasher. Its algorithm is not a stable
  serialized contract.
- Add a hashing dependency. The small staged key does not justify another
  dependency.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit, gate | `the_same_image_bytes_inserted_twice_produce_one_media_id` | Two handles from identical bytes collapse to one `HashSet` entry. |
| unit | `media_id_depends_on_bytes_not_relationship_context` | Relationship names do not enter the identifier calculation. |
| unit | `different_image_bytes_have_different_fixture_ids` | Distinct fixed fixtures have distinct identifiers. |
| regression | `staged_image_types_use_media_id_instead_of_embed_id` | Staged image values and line conversion preserve the content handle. |

The backlog test gate is that the same image bytes inserted twice produce one
`MediaId`.

## HLD impact

None. The rendering specification already defines `MediaId` as the
content-addressed replacement for `embed_id`.

## Risk routing

- Layout model. Use deterministic font mode for the consolidated hash gate and
  require all 28 entries to remain unchanged.
- Crate dependency graph. Run `cargo tree -p oxml-layout --edges normal` and
  reject every `rdocx-*` or `rpptx-*` dependency.

The consolidated gate also runs both feature modes and a package dry-run with
the existing sub-10 MiB archive bound. The package must not be published.

## Hash harness

Expected to remain unchanged. Only the unpublished staged copy changes, and
released rdocx image relationship handling stays intact.

## Implementation checklist

- [ ] Add and export the stable content-addressed `MediaId`.
- [ ] Replace staged output and line image relationship keys.
- [ ] Preserve the handle through inline-to-line conversion.
- [ ] Add the content identity and staged-type tests.
- [ ] Confirm released rdocx source and manifests are unchanged.
- [ ] Run the scoped checks and consolidated sprint riders.

## Open questions

None. Approved as key-level deduplication without a byte-storage abstraction.
