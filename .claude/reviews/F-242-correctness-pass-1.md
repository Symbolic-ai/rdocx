# F-242, correctness, pass 1

**Reviewed**: working-tree diff, 7 files, 794 changed lines
**Verdict**: 3 defects, 0 smells, 0 nitpicks

## Defects

### D1, comparison claims are not bound to their evidence

`scripts/readme_doctests.py:475`

The validator requires the approved URLs to occur on each product row, but it
does not validate the functionality, licence, runtime, or host-boundary cells.
Changing the python-docx row to claim macro execution and a proprietary licence
still passes. The approved-evidence matrix therefore does not reject the
unsupported comparison claims that the design plan names as its regression
contract.

### D2, capability prose can contradict its matrix row

`scripts/readme_doctests.py:347`

The validator compares only each capability ID and classification with the
canonical matrix. Changing DOCX-001 from open, save, and byte serialization to
embedded-application execution still passes with the same ID and
classification. The gate must bind the claimed public boundary as well as the
classification so an ID cannot become an unrelated citation.

### D3, nested badge destinations bypass local-link validation

`scripts/readme_doctests.py:281`

The Markdown-link expression consumes the image URL in a linked badge and
never observes the outer destination. Changing the local `LICENSE` destination
at `README.md:6` to a missing path still passes because the separate footer
link remains valid. The contract requires every local path to resolve, including
linked badge destinations.

## Smells

None.

## Nitpicks

None.

## Not found

No additional correctness, contract, panic, OOXML, test, or structure findings.
The change adds no crate, module, dependency, trait, generic, wrapper, feature
flag, parser, serializer, or rendered-output behavior.
