# F-001, correctness, pass 1

**Reviewed**: F-001 working tree, 6 files, 121 changed lines
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, The golden gate never varies system-font availability
`crates/rdocx/src/document.rs:2766`

The test renders twice in the same process with the same installed fonts and
unchanged font configuration. It proves repeatability under one environment,
not independence from system fonts. If
`layout_document_deterministic()` accidentally regressed to `Engine::new()`,
both calls would still discover the same system fonts and this equality would
pass. The constructor unit test checks the direct `FontManager` path, but it
does not protect the end-to-end wiring that this named golden gate is meant to
cover. Run the public rendering path with controlled system-font-present and
system-font-absent configurations and compare those PNG bytes.

## Smells

None.

## Nitpicks

None.

## Not found

No implementation correctness issue was found in the deterministic font
manager, engine propagation, layout entry point, or public `Document` method.
The deterministic database starts from bundled font bytes without invoking
system discovery, while document-embedded fonts remain explicit inputs. The
feature-off constructor returns the planned error, the no-default-features
suite passes, and the default feature is enabled for the existing `rdocx`
consumer.

The additive public API and package contents match the approved plan. The
package evidence is within the recorded size limits. The known Caladea licence
gap is assigned to F-004 in the same integration wave and was not introduced by
this implementation.
