# F-X133, Stop rebinding a canonical prefix on every retained element

**Status**: completed
**Sprint**: S86
**Size**: S
**Depends on**: F-X131, F-X132

## Problem

F-X161 already added the canonical `w` skip to
`push_root_attribute_record` in `crates/rdocx-oxml/src/text.rs`. Retained
`r`, `mc`, and `wp` attributes still copy same-URI declarations onto descendants
when a document part root already declares those prefixes. A document model
round trip currently writes each twice. Header and footer roots guarantee
`r` and `wp`, while note roots guarantee `r`.

## Spec reference

- `docs/hld/04-opc-and-packaging.md`, "The package", retained root attributes,
  namespace scope and canonical serialization.
- `docs/hld/14-development-backlog.md`, "F-X133, Stop rebinding a canonical
  prefix on every retained element".

## Approach

Scope the part root's exact canonical binding guarantees around its existing
serializer. `CT_Document::to_xml` supplies `r` and `mc`, plus canonical `wp`
when its retained root binding permits it. Header and footer serializers
supply `r` and conditionally canonical `wp`. Footnote and endnote serializers
supply `r`. `push_root_attribute_record` omits only a declaration whose
prefix and URI match an active guarantee. A scoped thread-local guard restores
the prior context on nested serialization, error, and panic. This avoids
threading context through every nested paragraph, run, table, and section
serializer, while standalone and comment serialization retains local bindings.
The existing `w` and `w14` behavior stays unchanged. Refresh the
`rdocx-oxml` archive measurement in its README and
`scripts/readme_doctests.py` because the source and tests are packaged.

## Rejected alternatives

- Dropping declarations during capture would break expanded-name lookup in
  the retained record.
- Skipping `r`, `mc`, or `wp` globally would lose needed local declarations in
  standalone and comment contexts.
- Propagating a new context parameter through every nested serializer would
  enlarge the diff and public method surface for the same lexical scope.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `a_retained_element_does_not_rebind_a_prefix_its_part_root_declares` | **Test gate.** Document serialization emits canonical `w`, `r`, `mc`, and `wp` once at the root, retaining producer attributes, a new binding, and raw child bytes. |
| regression | `a_standalone_or_comment_paragraph_keeps_its_local_relationship_binding` | A context without a guaranteed `r` root keeps its local declaration. |
| regression | `header_footer_and_note_roots_own_their_canonical_bindings` | Header, footer, footnote, and endnote roots hold the guaranteed bindings once. |
| regression | `a_root_prefix_shadow_with_another_uri_remains_local` | A different-URI shadow remains on the retained element. |
| regression | `a_noncanonical_wp_root_does_not_claim_the_canonical_binding` | A noncanonical root `wp` leaves a local canonical `wp` declaration intact. |
| unit | `nested_root_binding_scopes_restore_on_unwind` | Nested scopes restore prior bindings, including after a panic. |

## HLD impact

- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- **Any parser or serialiser**. This changes part serialization. Read
  `docs/hld/04-opc-and-packaging.md` and
  `06-presentationml-model.md`. Check schema child order, prefix-tolerant read,
  fixed-prefix write and byte-for-byte unknown subtree retention.

## Hash harness

Expected unchanged. The generated samples do not use retained producer root
attributes. Confirm with the harness.

## Implementation checklist

- [x] Identify the exact canonical bindings each affected part root guarantees.
- [x] Add scoped, same-URI skips to retained-attribute writing.
- [x] Prove standalone and comment contexts retain local declarations.
- [x] Prove the new gate fails with the new `r`, `mc`, and `wp` skip removed.
- [x] Run scoped verification and obtain a zero-finding microscope review.

## Open questions

None. The skip must be tied to what the serialized part root actually declares.
