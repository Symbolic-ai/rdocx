# Noto Sans SC F-X199 oracle Thin fixture

This oracle-only fixture is a static Thin instance derived from the exact
published `NotoSansSC-FX058-subset.ttf` bytes checked in at
`crates/oxml-layout/fonts/`. It lets the pinned Writer oracle render the same
Thin outlines that the product's variable-font default uses. It is not loaded
by product code and does not replace or alter the shared bundled font.

The source font is the F-X058 deterministic subset of
`ofl/notosanssc/NotoSansSC[wght].ttf` from the Google Fonts `main` branch,
retrieved 2026-08-26. Its upstream provenance and subset repertoire are in
`crates/oxml-layout/fonts/NOTICE-Noto` and
`crates/oxml-layout/fonts/SUBSET-NotoSansSC.md`.

Source SHA-256:
`b06144fa7b2d5212fe21344261449c9350f603e3e2ae625e76306022d024fbe5`

Output SHA-256:
`390ba9f55d4dd69915736d2b225d602b40012cd2c50db4c1e6d2bbdfd61e63a6`

Reproduce the fixture with `hb-subset (HarfBuzz) 13.2.1`:

```text
hb-subset crates/oxml-layout/fonts/NotoSansSC-FX058-subset.ttf --output-file=scripts/oracle-fonts/NotoSansSC-FX058-oracle-thin.ttf --unicodes='*' --variations='wght=100' --name-IDs='*' --name-languages='*'
```

The complete name table is retained so Writer can resolve the static Thin
instance through the document's `Noto Sans SC` family request. The fixture is
distributed under the SIL Open Font License 1.1 in `LICENSE-Noto`.
