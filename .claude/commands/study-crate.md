---
description: Build a working understanding of one crate before changing it. Read-only.
---

# /study-crate <crate>

Produce a map of a crate: what it owns, what it depends on, where its seams are,
and where its traps are. Read-only.

Use before the first story that touches an unfamiliar crate, or before an
`XL` story that will be split.

## Steps

1. **The manifest.** Dependencies, features and their defaults, and anything
   unusual: a narrowed feature set, an `include`, a `publish = false`. The root
   `Cargo.toml` carries comments explaining why several feature sets are
   minimal. Read them rather than regenerating the reasoning.

2. **The module tree.** Every file with a one-line purpose and a line count. The
   line counts matter, because they say where the weight is.

3. **The public surface.** Everything re-exported from `lib.rs`. Then, separately,
   **types that leak**: public function signatures naming a type from another
   crate that this crate does not re-export. `rdocx` has several, and they turn
   an apparently internal change into a breaking one.

4. **The dependency position.** What depends on this crate, and what it depends
   on. Confirm it respects the layering rule in `docs/hld/03-architecture.md`.

5. **The conventions in force.** How this crate parses and serialises, how it
   names things, how it handles errors, whether it captures raw XML. New code
   matches the file it sits in.

6. **The traps.** Anything a newcomer would get wrong:
   - Behaviour that is deliberately wrong, listed in `CLAUDE.md`.
   - Invariants held by convention rather than by the type system.
   - Ordering requirements, such as OOXML `xsd:sequence`.
   - Anything whose change moves the hash harness.

7. **The tests.** Where they live, what they cover, and **what they do not**.
   The gap is usually more useful than the coverage.

8. **Report** as a map. Do not paste large code blocks. Cite `path:line` and let
   the reader open it.

## Report shape

```markdown
# <crate>

**Purpose**: one sentence.
**Position**: depends on X, Y. Depended on by A, B.
**Size**: N files, N lines.

## Modules
| File | Lines | Purpose |

## Public surface
What is exported. What leaks and from where.

## Conventions
How this crate does things.

## Traps
Each with a path:line and why it bites.

## Test coverage
What is covered. **What is not.**
```

## Refused situations

- **Changing anything.** This command reads.
- **Reporting a structure you inferred from names rather than read.**
