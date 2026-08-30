# F-221, all, pass 2

**Reviewed**: uncommitted working tree implementation diff, 3 files, 468 changed lines, with 443 additions and 25 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, the mandatory PowerPoint password oracle remains unperformed
`.claude/scratch/F-221-progress.md:44`

The approved differential gate requires pinned PowerPoint 16.104 build
16.104.25121423 to open the generated Agile presentation with the correct
password and reject it with the wrong password. The ignored executable gate
now exists at `crates/rpptx/tests/integration.rs:133`, and the progress record
pins candidate SHA-256
`a0d33171c63ec084231daeef3b35718f5a2d709a5c92c9c0e2017ccaf9fa52d6`.
However, the record explicitly says that correct-password opening and
wrong-password rejection were both not performed at
`.claude/scratch/F-221-progress.md:44` and
`.claude/scratch/F-221-progress.md:45`. The modal PowerPoint UI explains the
missing evidence but does not satisfy the approved gate. Perform and record
both observations against that identified candidate and exact build before
completion.

## Smells

None.

## Nitpicks

None.

## Not found

Correctness produced no additional finding. The D1 selective-staging defect
from pass 1 is remediated. `staged_package` reparses and compares retained
presentation, slide, and notes roots before serializing them at
`crates/rpptx/src/lib.rs:770`, `crates/rpptx/src/lib.rs:834`, and
`crates/rpptx/src/lib.rs:853`. The producer-prefixed package regression at
`crates/rpptx/tests/integration.rs:200` passes and proves an untouched
producer-signed lexical form remains valid. The current-state mutation gate at
`crates/rpptx/tests/integration.rs:224` passes for slide, shape, text, core
property, and package-graph mutations, with retained signatures reported as
cryptographically invalid.

Encrypted-save atomicity produced no finding. The pass 1 password-path defect
is remediated at `crates/rpptx/tests/integration.rs:157`. An empty password
leaves sentinel destination bytes and the live presentation unchanged, while
the separate directory case exercises publication failure. The focused test
passes. The implementation stages encryption before opening its exclusive
sibling temporary file, syncs the staged file, atomically replaces the target,
and removes the temporary file on failure.

Panic safety produced zero findings. OOXML schema order, namespace handling,
and unmodelled subtree preservation produced zero findings. Security boundary,
signature coverage, certificate trust separation, and signing atomicity
produced zero findings. Public API shape, default-off feature forwarding, and
binding feature isolation produced zero findings. Structure produced zero
findings. No new crate, module, file, trait, generic, builder, forwarding
wrapper, or production dependency was introduced. Smells produced zero
findings. Nitpicks produced zero findings.

Focused integration tests for untouched producer signatures, invalidation
after mutation, and encrypted-save failure atomicity pass with both security
features. A library-only no-default check with both security features also
passes. Those results do not clear D1 because no test run or progress evidence
claims the two required PowerPoint password observations.
