# F-X133, Stop rebinding a canonical prefix on every retained element

**Status**: approved
**Sprint**: S86
**Size**: S
**Depends on**: F-X131, F-X132

## Problem

`capture_root_attribute_record` in `crates/rdocx-oxml/src/text.rs` must retain
the binding of each prefixed producer attribute in its private record. The
write side should copy that binding only when its output element needs it.
It already skips canonical `w` and `w14`, but the remaining fixed root bindings
can be repeated on every retained element, growing a saved part without
changing name resolution.

## Spec reference

- `docs/hld/04-opc-and-packaging.md`, "The package", retained root attributes,
  namespace scope and canonical serialization.
- `docs/hld/14-development-backlog.md`, "F-X133, Stop rebinding a canonical
  prefix on every retained element".

## Approach

At `push_root_attribute_record`, compare a record declaration with the
canonical binding guaranteed by the output part root. Keep the declaration in
the private record for namespace-resolved attribute precedence, but omit a
same-URI binding on the output element. Copy aliases, genuinely new bindings
and shadows, including a canonical prefix rebound to a different URI. Keep
part roots namespace complete. Do not loosen the retained-owner ambiguity
matcher from F-X132.

## Rejected alternatives

- Dropping declarations during capture would break expanded-name lookup in
  the retained record.
- Treating every prefix as root-bound would lose a needed local declaration.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `a_retained_element_does_not_rebind_a_prefix_its_part_root_declares` | **Test gate.** Plain save keeps producer attributes and a new binding but emits each guaranteed canonical binding only at the part root. |
| regression | `a_genuinely_rebound_prefix_remains_local` | A canonical spelling mapped to another URI keeps its local shadow. |
| round-trip | `an_unmodelled_child_survives_prefix_deduplication` | Raw child bytes survive and reopen under the correct namespace. |

## HLD impact

- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- **Any parser or serialiser**. Read `docs/hld/04-opc-and-packaging.md` and
  `06-presentationml-model.md`. Check schema child order, prefix-tolerant read,
  fixed-prefix write and byte-for-byte unknown subtree retention.

## Hash harness

Expected unchanged. The generated samples do not use retained producer root
attributes; confirm with the harness.

## Implementation checklist

- [ ] Identify the canonical bindings each written part root guarantees.
- [ ] Skip only same-URI declarations already guaranteed on that part root.
- [ ] Add the regression and round-trip cases to an existing test module.
- [ ] Run scoped verification and obtain a zero-finding microscope review.

## Open questions

None. The skip must be tied to what the serialized part root actually declares.
