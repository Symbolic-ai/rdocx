# 04, OPC and packaging

Owner: `oxml-opc`, with media naming in `oxml-media`.

## The package

```rust
pub struct OpcPackage {
    pub content_types: ContentTypes,
    pub package_rels:  Relationships,
    pub part_rels:     HashMap<String, Relationships>,  // key: "/ppt/slides/slide1.xml"
    pub parts:         HashMap<String, Vec<u8>>,        // same key shape
}
```

Fully in memory. Every part is decompressed at open. Part names are normalised
to a leading slash. This design is already format-neutral and is carried over
essentially unchanged from `rdocx-opc`.

**Saves are deterministic.** Both `part_rels` and `parts` are emitted in sorted
key order, so writing the same package twice produces byte-identical output.
That property is load-bearing for the round-trip corpus and must not regress.

## Generalising the constructors

The only docx-specific code in the existing crate is two constructors. They are
replaced by:

```rust
impl OpcPackage {
    /// Empty package: minimal content types, no parts, no relationships.
    pub fn new() -> Self;

    /// Package whose officeDocument relationship points at `part_name`.
    /// Package-relative, no leading slash: "word/document.xml", "ppt/presentation.xml".
    pub fn with_main_part(part_name: &str, content_type: &str) -> Self;
}

impl ContentTypes {
    /// Only the two universal defaults, "rels" and "xml".
    pub fn minimal() -> Self;
}
```

The docx presets become a short private helper in `crates/rdocx/src/document.rs`,
and the pptx presets one in `crates/rpptx/src/package.rs`.

Rejected alternatives, recorded so they are not revisited: a `PackageKind` enum
forces the leaf crate to carry every format's content-type table and grows a
variant per format. Feature-gated `new_docx` / `new_pptx` helpers make a leaf
crate feature-conditional for two functions' worth of string constants, and
features are additive so a workspace containing both compiles both anyway.

## What transfers unmodified

**`main_document_part()`** keys off the `officeDocument` relationship type,
which PowerPoint uses for `/ppt/presentation.xml`. It reads a `.pptx` today.

**`resolve_rel_target(source_part, target)`** joins a relative target against
the source part's directory and collapses `.` and `..`. It already handles
`../slideLayouts/slideLayout1.xml` from `/ppt/slides/slide1.xml` correctly, and
it clamps a traversal that escapes the root rather than allowing zip-slip.

**`rels_path_to_part_name`** and its inverse are generic path algebra.

## Relationship types

`rel_types` stays one flat module, grouped by comment. The existing thirteen
constants are kept. Added:

```rust
// Package-level. Note the package namespace, not officeDocument.
CORE_PROPERTIES, THUMBNAIL

// Shared officeDocument
EXTENDED_PROPERTIES   // docProps/app.xml
CUSTOM_PROPERTIES     // docProps/custom.xml
COMMENTS              // Word comments part

// PresentationML
SLIDE, SLIDE_LAYOUT, SLIDE_MASTER, NOTES_SLIDE, NOTES_MASTER,
PRES_PROPS, VIEW_PROPS, TABLE_STYLES, HANDOUT_MASTER
```

A `content_types` constants module is added alongside, so neither format crate
hand-types the long MIME strings.

Both facades resolve core properties through the package-level
`CORE_PROPERTIES` relationship and retain its normalized target. Immutable
property access leaves the source part bytes untouched. Mutable access marks
the typed `CoreProperties` model for serialization to that target with its
content-type override. A package that creates metadata without an existing
relationship uses `/docProps/core.xml` and adds the missing package
relationship. If that conventional part name is already occupied without the
core-properties relationship, serialization returns an error before changing
the package.

The Word facade owns external hyperlink relationships at the document part
boundary. `Document::add_hyperlink_relationship` allocates the relationship,
and `Paragraph::add_hyperlink` writes a schema-ordered `w:hyperlink` that
references it. The same paragraph writer emits explicit hard breaks as run
content. Both operations use the existing package-preserving save path, so
unmodelled parts and relationships remain intact.

Numbering state is also fail-closed at this boundary. Updating a known list
level marks the existing numbering model for serialization. Rejecting an
unknown list identifier or an invalid level does not create an empty numbering
part, relationship, or content-type entry. Numbering parsers retain namespace
declarations and compatibility attributes from modelled containers. Unknown
level children use their `CT_Lvl` schema slots, while abstract-definition,
instance, and root children keep insertion-aware boundaries. Mutating or adding
a definition therefore preserves producer extensions, identifiers, templates,
and level overrides verbatim. Identifier allocation uses the next value after
the maximum when available and the first unoccupied value when the maximum is
`u32::MAX`.

The Word facade resolves an existing comments part through the main document's
`COMMENTS` relationship and retains the normalized target. Saving serializes
the typed comments model back to that target with its content-type override.
The model preserves unmodelled attributes and children at their insertion
boundaries, while comment range and reference anchors remain ordered among
neighbouring paragraph and run XML. A document without a comments relationship
does not gain a comments part, relationship, or override during an ordinary
save.

The Word facade resolves an existing settings part through the main document's
`SETTINGS` relationship and retains the normalized target instead of assuming
`/word/settings.xml`. `rdocx-oxml` projects valid document protection metadata
while retaining the complete settings bytes as the serialization source.
Saving writes those bytes only to the resolved existing target. Unsupported
protection modes, unsupported algorithm enum values, and malformed numeric
metadata remain opaque and byte-identical. A document without a settings
relationship does not gain a settings part, relationship, or content-type
override during an ordinary save.

Threaded comments add a document relationship using the Microsoft
`commentsExtended` relationship type. The facade retains its resolved target
and writes the comments-extended content type at that exact part. New comment
state creates both relationships and both overrides together. Existing custom
targets remain authoritative, and removal of the final API-owned thread removes
only the parts, relationships, and overrides created by the typed model.

Content-control data binding follows the existing package graph rather than a
conventional filename. The main document's custom XML relationship resolves an
item part. That item's custom XML properties relationship resolves the
properties part whose root `ds:itemID` identifies the datastore. Matching is
case-insensitive and accepts the optional braces used by producers, while a
same-local-name attribute or element in another namespace is not metadata.

The binding evaluator accepts namespace-aware absolute child paths with
optional one-based numeric child indices. Functions, wildcards, descendant
axes, and general predicates are outside the contract. Prefix mappings and
in-scope namespace shadowing resolve every path step by expanded name. The
selected final element must be unique and contain only simple text or be empty.
The facade replaces only that element's text span, expands an empty selected
element when needed, and retains every unrelated custom XML byte exactly.
Display and custom-part changes are staged together and committed only after
all selected bindings serialize and reparse successfully.

## Part naming

**Numeric suffixes are allocated after the greatest positive parsed suffix,
never as `count + 1`.** Deleting slide 2 and then adding a slide must not
collide with slide 3. `MediaNamer` scans positive decimal suffixes in the
requested directory and stem, including `usize::MAX`, and ignores missing,
signed, zero, nonnumeric and unrelated suffixes. Ordinary packages allocate
`1 + max(existing suffix)`. At the finite boundary, checked increment wraps
from `usize::MAX` to 1 and skips every occupied parsed suffix until a free
positive number is found. Both facades use this allocator, so allocation never
creates `image0` or overwrites an existing numbered image part.

Canonical part layouts:

```
docx                      pptx
/word/document.xml        /ppt/presentation.xml
/word/styles.xml          /ppt/slides/slideN.xml
/word/media/imageN.ext    /ppt/slideLayouts/slideLayoutN.xml
/word/charts/chartN.xml   /ppt/slideMasters/slideMasterN.xml
/word/embeddings/         /ppt/notesSlides/notesSlideN.xml
  WorkbookN.xlsx          /ppt/theme/themeN.xml
/word/comments.xml        /ppt/media/imageN.ext
/word/commentsExtended.xml /ppt/charts/chartN.xml
                          /ppt/embeddings/WorkbookN.xlsx
```

Comment part creation uses the conventional names when free and scans numbered
alternatives when either path is occupied. It never overwrites an unrelated
part merely because the conventional comment path exists.

Word chart assembly follows the same independent suffix rule as PowerPoint.
The document relationship targets `/word/charts/chartN.xml`, and that chart's
package relationship targets `/word/embeddings/WorkbookN.xlsx`. Both parts and
their content-type overrides are staged with the drawing on cloned package and
document state. The mutation becomes visible only after the typed ChartML,
SpreadsheetML workbook, relationships, content types, and structured drawing
all serialize successfully.

## Media

`oxml-media` owns everything about an image byte string.

```rust
pub enum ImageFormat { Png, Jpeg, Gif, Bmp, Tiff, Webp, Svg, Emf, Wmf }

impl ImageFormat {
    pub fn sniff(data: &[u8]) -> Option<Self>;
    pub fn from_extension(ext: &str) -> Option<Self>;
    pub fn extension(self) -> &'static str;
    pub fn content_type(self) -> &'static str;
}

/// Sniff first, fall back to the extension, default to PNG.
pub fn resolve(data: &[u8], filename: &str) -> ImageFormat;

pub struct ImageInfo {
    pub format: ImageFormat,
    pub width_px: u32, pub height_px: u32,
    pub dpi_x: Option<f64>, pub dpi_y: Option<f64>,  // None means the file declares none
    pub bit_depth: u8, pub channels: u8, pub has_alpha: bool,
}
pub fn probe(data: &[u8]) -> Option<ImageInfo>;

pub struct NativeSize {
    pub width_emu: i64, pub height_emu: i64,
}

impl ImageInfo {
    pub fn native_size(&self, default_dpi: f64) -> Option<NativeSize>;
}

pub struct MediaNamer { /* dir, stem, next */ }
impl MediaNamer {
    pub fn scan<'a>(dir: &str, stem: &str, existing: impl Iterator<Item = &'a str>) -> Self;
    pub fn next_part_name(&mut self, ext: &str) -> String;
}
```

**Sniffing beats the extension.** `resolve` uses detected image bytes before a
filename extension, so a `.png` that is really a JPEG receives the JPEG
extension and content type. Unknown bytes fall back through a recognised
filename extension and finally to PNG for compatibility.

`rdocx::Document` scans existing `/word/media/imageN.ext` parts into a
`MediaNamer` when it opens. Every body, header, footer, and raw-XML image path
uses the allocator and registers the sniffed canonical extension and content
type before adding its relationship. HTML and layout extraction resolve MIME
from the stored bytes first, so a misleading package part name cannot override
the actual image format.

**`native_size` takes the DPI rather than baking one in**, because the right
default differs by consumer: python-docx assumes 72 when a file declares none,
while Word assumes 96. Each declared finite positive axis DPI takes precedence
over the caller default. Missing or invalid declared DPI falls back per axis.
The conversion multiplies pixels by 914400 EMU per inch and truncates toward
zero. The method returns `None` if either effective DPI is not finite and
positive, or if a converted dimension is outside the `i64` range.

`NativeSize` keeps the result dependency-free and exposes explicit EMU fields.
The PresentationML picture insertion path supplies 72 for python-pptx parity
without adding an `oxml-core` edge to `oxml-media`.

`rdocx::Document::add_picture_auto` is an additive convenience API that probes
the image and calculates `native_size(72.0)` before changing document state.
It converts the shared EMU result with `Length::emu` and delegates successful
insertion to the existing explicit-size `add_picture` path. Unavailable
dimensions return `rdocx::Error::UnavailableImageDimensions` with the supplied
filename before a media part, relationship, drawing, or paragraph is added.

`rpptx::Presentation` scans `/ppt/media/` into a content-hash `MediaStore` when
it opens. Insertion compares the complete byte string inside each hash bucket,
reuses an equal package-wide media part, and otherwise allocates the next
numbered part after the greatest occupied suffix. The sniffed canonical
extension and content type are registered with the package. Each source slide
creates or reuses its own internal image relationship to that shared part, with
a relative target resolved from the slide part name.

Slide removal considers only `/ppt/media/` targets reached from the removed
slide and its removed notes relationship scopes. A candidate part is deleted
only when no remaining internal package relationship reaches it. Pre-existing
orphan media outside that candidate set is left untouched. The facade rebuilds
its content-hash media index after the graph change, so a later insertion sees
the surviving package state.

Header parsing is lifted from the PDF crate, where `jpeg_dimensions` and the
PNG IHDR reader are currently private. The JPEG walk classifies SOI, TEM and
RST0 through RST7 as standalone markers with no length field, and EOI terminates
the codestream. Length-bearing segments are validated for a present length of
at least two bytes and bounds within the input before the walk indexes or
advances. Fill bytes and truncated input return safely. Preserve these
invariants when the reader moves to `oxml-media`.

## Package integrity

Both facades expose a `validate()` that is cheap, non-panicking, and run
automatically under `debug_assertions` before `save`. It checks dangling
relationship ids, missing content-type overrides, relationship targets that
resolve to no part, and orphan media. `rpptx` adds its own presentation-specific
checks, listed in `06-presentationml-model.md`.

Comment mutations validate coordinates and allocate every required id before
changing package or document state. Saving keeps the comments and
comments-extended relationship graph reachable from the main document, with
matching overrides and namespace declarations. A failed validation or
allocation leaves anchors, typed parts, relationships, and overrides unchanged.

Tracked-revision resolution is also staged above the package boundary. The
facade serializes a candidate main-document tree, resolves selected revision
placements, and reparses the complete candidate before replacing live typed
state. Namespace declarations carried only by a removed revision or property
owner are promoted to retained raw descendants. Any selector, revision-shape,
namespace, parse, or serialization failure leaves the package part bytes and
live document unchanged. The ordinary deterministic save path writes the
validated result later and preserves every unrelated part and relationship.

Template rendering follows the same staged package boundary. A stack parser
pairs nested controls within one body or table-row container before evaluation.
The evaluator clones typed body entries and rows into candidate sequences, so
section properties, row properties, and ordered raw-child sidecars travel with
their owner. A row loop may clone several adjacent template rows per iteration.
The original table and its properties, grid, raw boundaries, content controls,
and relationships remain in place. Cloned row and cell property sequences keep
grid spans, vertical merge state, and unmodelled children byte for byte.
Repeated numbered paragraphs keep their existing numbering part reference and
level. No numbering relationship, instance, or abstract definition is added.
Markers are removed only from the candidate. Scalar syntax and JSON values are
resolved against lexical loop scopes before replacement reaches typed body
content, relationship-resolved headers and footers, raw text boxes, or chart
parts. Replacement values pass through collision-free sentinels, so a value
that contains template syntax is not evaluated recursively. The live typed
document and package are replaced only after every discovered tag is accounted
for, every repeated numbering reference resolves, and the candidate document
serializes successfully. Any control, lookup, numbering, scalar-type, parse, or
serialization failure leaves package parts, typed content, and layout caches
unchanged.

Mail merge uses the same fail-closed package boundary. Separate mode clones the
typed document and complete package for each record, applies the merge-local
field policy, serializes, and reopens every candidate before returning the
record-ordered outputs. Section mode serializes the main document and scans it
by expanded name for every header and footer reference, including references
inside content controls and preserved wrappers. Relationship-namespace ids are
resolved through the document relationship graph, and only the resulting
internal header and footer parts join relationship-resolved footnotes and
endnotes in the merge-dependency scan. A referenced non-body `MERGEFIELD` that
varies across records rejects the operation before candidate assembly.

Combined output reuses the first validated package and replaces only its main
body with the record bodies and their schema-ordered section boundaries.
Bookmark, content-control, and drawing identities are allocated without
collision across those bodies. Simple and complex bookmark field targets plus
hyperlink anchors follow renamed bookmarks in typed and preserved raw XML.
Clean parsed footnotes remain source-backed. An actual footnote field update
patches only the field-source spans in the relationship-resolved part, so
unmodelled siblings remain byte-preserved. Any rejected record, XML parse, or
identity-allocation failure leaves the source and all prospective outputs
uncommitted.
