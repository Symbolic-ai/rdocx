# F-178, HTML import

**Status**: approved
**Sprint**: S55
**Size**: L
**Depends on**: none

## Problem

The facade exports DOCX and RTF input plus HTML output, but it cannot turn a
browser fragment or CMS document into an editable Word document.
`Document::to_html` is outbound only (`crates/rdocx/src/document.rs:3717`),
and `rdocx-html` owns a semantic emitter rather than an HTML parser
(`crates/rdocx-html/src/lib.rs:29`). Browser markup is not XML, so the existing
`quick-xml` readers cannot safely recover common omitted end tags or malformed
nesting.

M18 requires source-ordered paragraphs, runs, tables, and lists. It also
requires a stable diagnostic for every unsupported CSS property or visible
construct that can be skipped safely (`docs/hld/14-development-backlog.md:1439`
and `docs/hld/14-development-backlog.md:1484`).

## Spec reference

- `docs/hld/03-architecture.md`, "Why these seams" and "Facade conventions".
- `docs/hld/10-bindings-spec.md`, "Native Word facade stability".
- `docs/hld/12-testing-strategy.md`, "Test taxonomy" and "Binding tests".
- `docs/hld/14-development-backlog.md`, "Milestone 18, Format breadth" and
  "F-178, HTML import".
- `docs/hld/15-build-and-toolchain.md`, "Dependency policy" and the WASM and
  package gates.
- WHATWG HTML Living Standard, "Parsing HTML documents", for tree repair,
  text tokenization, and whitespace handling.
- CSS Cascading and Inheritance Level 5, "Cascading origins" and "Specificity",
  for the supported inline and embedded style boundary.

## Approach

Add one private `html` module to `rdocx`. Add `scraper` 0.27 with default
features disabled and only its `errors` feature enabled. Its `html5ever` tree
builder supplies browser-compatible document and fragment parsing plus parse
error capture. Keep the dependency in the facade. Moving import into
`rdocx-html` would require either a dependency cycle or a second public
intermediate document model.

Expose an additive native pre-1.0 API:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HtmlDiagnostic {
    pub location: String,
    pub property: Option<String>,
    pub message: String,
}

pub struct HtmlReadResult {
    pub document: Document,
    pub diagnostics: Vec<HtmlDiagnostic>,
}

impl Document {
    pub fn from_html(html: &str) -> Result<HtmlReadResult>;
    pub fn open_html<P: AsRef<Path>>(path: P) -> Result<HtmlReadResult>;
}
```

Add a concrete `Error::Html { location, message }` variant. Path input is
bounded before allocation and must be valid UTF-8. Parser recovery produces a
diagnostic. A limit violation or unrecoverable projection error fails without
returning a partial document.

Project complete documents and fragments directly into a fresh `Document`.
Support block order from `p`, `div`, headings, block quotes, `pre`, and direct
body text. Support nested inline text, `span`, bold, italic, underline, strike,
superscript, subscript, code, and hard breaks. Collapse HTML whitespace outside
`pre` and preserve it inside `pre`.

Project nested ordered and unordered lists through one Word list definition per
top-level list, with levels capped at Word's nine-level boundary. Project
tables, row groups, rows, headers, and cells in source order. Resolve bounded
`colspan` and `rowspan` through the existing grid-span and vertical-merge
model, and preserve multiple cell paragraphs.

Support inline declarations and embedded `<style>` rules with bounded type,
class, id, descendant, and child selectors. Apply normal specificity, source
order, and inline priority for font family, point or pixel size, bold, italic,
underline, strike, foreground and background color, text alignment, paragraph
spacing, and indentation. Diagnose every unsupported property, value, at-rule,
or selector once at a deterministic DOM path while retaining supported sibling
content.

Do not fetch external resources. Keep anchor text and image alternate text,
but diagnose dropped links, images, external stylesheets, scripts, forms,
frames, embeds, and objects. This follows the story's four-item fidelity gate
without inventing network authority.

Apply explicit 64 MiB input and retained-text bounds, depth 256, 100,000 DOM
nodes, 100,000 projected blocks, 100,000 runs, 10,000 rows, 256 columns,
50,000 cells, and 10,000 diagnostics. Count the DOM before projection and use
checked traversal throughout. Serialize the candidate to DOCX and reopen it
before publishing the result, proving the generated Word tree is valid and
schema ordered.

## Rejected alternatives

- `quick-xml` cannot implement HTML5 tree repair for normal browser fragments.
- Regex tokenization cannot preserve nested formatting, lists, and tables
  under malformed but recoverable markup.
- Import inside `rdocx-html` creates a facade dependency cycle or a second
  public document model.
- Fetching images or stylesheets would add network, security, and lifetime
  semantics that the story does not authorize.
- Silently dropping unsupported CSS violates the M18 diagnostic contract.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `html_import_collapses_whitespace_without_losing_inline_order` | Document and fragment whitespace, entities, direct body text, `pre`, and hard breaks project in exact source order. |
| unit | `html_import_restores_nested_inline_and_css_formatting` | Semantic tags, inline declarations, embedded selectors, specificity, and inherited supported properties produce exact run and paragraph formatting. |
| unit | `html_import_rejects_each_declared_resource_limit` | Input, DOM, depth, block, run, row, column, cell, diagnostic, and retained-text bounds fail closed. |
| regression | `unsupported_html_css_is_diagnosed_without_dropping_supported_siblings` | Parser repairs and every unsupported property, value, selector, and visible element produce stable path-aware diagnostics while adjacent supported content survives. |
| regression | `html_import_projects_nested_lists_and_spanned_tables` | Nine-level list bounds, list identity, table source order, multiple cell paragraphs, grid spans, and vertical merges match the expected typed tree. |
| integration | `html_import_projects_a_reopenable_word_document` | A source-built browser and CMS fixture matrix saves, reopens, and retains equal normalized paragraphs, runs, lists, tables, and diagnostics. |

The **test gate** is regression. A fixture set of HTML documents produces the
expected paragraph, run, table and list structure, with unsupported CSS
recorded as a diagnostic. All fixtures are source strings inside existing unit,
regression, and integration binaries.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- **Any parser or serialiser**. Read `docs/hld/04-opc-and-packaging.md` and
  `docs/hld/06-presentationml-model.md`. The HTML reader is prefix-independent
  and one-way. Its extra gate saves and reopens the generated DOCX, proves
  source order and typed schema order, and confirms unsupported source markup
  is diagnosed rather than smuggled into OOXML.
- **Crate dependency graph**. Read `docs/hld/03-architecture.md`. Add `scraper`
  only to `rdocx`, retain the existing `rdocx` to `rdocx-html` direction, and
  verify no `oxml-*` crate gains a format dependency.
- **Public API of a published crate**. The result, diagnostic, error, and two
  constructors are additive native pre-1.0 API. Run the verified package dry
  run and archive-size assertion for `rdocx`.
- **WASM or PyO3 bindings**. Python, WASM, and CLI gain no new methods, but the
  facade dependency graph changes. Run both wasm32 checks, a scoped MSRV check,
  and workspace tests with both Python binding exclusions.
- **A new trait, generic parameter, crate, module or file**. The private
  `crates/rdocx/src/html.rs` module needs explicit approval. No trait, generic,
  crate, or second model is introduced.

## Hash harness

Expected unchanged, 49 of 49. Existing samples do not import HTML. Any delta
is unrelated and blocks the sprint. Do not edit `scripts/hash_baseline.json`.

## Implementation checklist

- [ ] Add the approved private module, browser-grade parser dependency, facade
      error, result, diagnostics, and native constructors.
- [ ] Bound DOM construction, traversal, projected content, diagnostics, and
      retained text.
- [ ] Project source-ordered blocks, nested inline formatting, whitespace, and
      hard breaks.
- [ ] Project nested lists and spanned tables through the existing Word model.
- [ ] Apply the supported inline and embedded CSS cascade.
- [ ] Diagnose parser repairs, unsupported CSS, dropped visible constructs,
      and external resources without losing supported siblings.
- [ ] Add source-built unit, regression, save, and reopen coverage to existing
      test binaries.
- [ ] Run scoped facade, MSRV, dependency, packaging, WASM, full verification,
      and unchanged-harness checks.

## Open questions

- Resolved. Create `crates/rdocx/src/html.rs` and add the direct workspace
  `scraper` 0.27 dependency with default features disabled and only its
  `errors` feature enabled. The module keeps the parser and projection in one
  readable owner. The dependency provides HTML5 tree repair and parse-error
  reporting without a second facade model.
