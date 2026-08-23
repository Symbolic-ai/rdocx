//! PDF/A profile selection, preflight, and deterministic metadata.

use std::error::Error;
use std::fmt;

use oxml_layout::{Color, LayoutResult, Paint, PositionedElement};

use crate::font;
use crate::structure::valid_structure;

pub(crate) const SRGB2014: &[u8] = include_bytes!("../assets/sRGB2014.icc");
pub(crate) const FILE_IDENTIFIER: &[u8; 16] = b"rdocx-pdfa-00001";

/// An archival conformance level supported by the PDF backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PdfConformance {
    /// ISO 19005-2, level B.
    PdfA2b,
    /// ISO 19005-3, level B.
    PdfA3b,
}

impl PdfConformance {
    pub(crate) const fn part(self) -> u8 {
        match self {
            Self::PdfA2b => 2,
            Self::PdfA3b => 3,
        }
    }
}

/// Options for the explicit archival PDF renderer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PdfOptions {
    /// The required archival profile.
    pub profile: PdfConformance,
}

impl PdfOptions {
    /// Select one supported archival profile.
    pub const fn new(profile: PdfConformance) -> Self {
        Self { profile }
    }
}

/// A reason an input cannot be represented by the requested PDF/A profile.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum PdfError {
    /// A shaped run refers to a font that cannot be embedded and subset.
    MissingEmbeddedFont { font_id: u32 },
    /// The logical structure tree or one of its marked-content references is invalid.
    InvalidStructure,
    /// A link requests an action that the archival path refuses.
    ForbiddenAction { action: String },
    /// A color component is non-finite or outside its declared range.
    UnsupportedColor,
    /// A paint kind has no archival PDF representation.
    UnsupportedPaint { paint: &'static str },
}

impl fmt::Display for PdfError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingEmbeddedFont { font_id } => {
                write!(formatter, "font {font_id} cannot be embedded for PDF/A")
            }
            Self::InvalidStructure => formatter.write_str("invalid tagged PDF structure"),
            Self::ForbiddenAction { action } => {
                write!(formatter, "action is forbidden in PDF/A output: {action}")
            }
            Self::UnsupportedColor => formatter.write_str("unsupported PDF/A color state"),
            Self::UnsupportedPaint { paint } => {
                write!(formatter, "unsupported PDF/A paint: {paint}")
            }
        }
    }
}

impl Error for PdfError {}

pub(crate) fn preflight(layout: &LayoutResult) -> Result<(), PdfError> {
    if let Some(structure) = &layout.structure
        && !valid_structure(structure, &layout.pages)
    {
        return Err(PdfError::InvalidStructure);
    }

    let mut usage = font::collect_glyph_usage(layout);
    let fonts = layout
        .fonts
        .iter()
        .map(|font| (font.id, font))
        .collect::<std::collections::HashMap<_, _>>();
    for (font_id, font_usage) in &mut usage {
        let Some(font_data) = fonts.get(font_id) else {
            return Err(PdfError::MissingEmbeddedFont { font_id: font_id.0 });
        };
        if font::prepare_font(font_data, font_usage).is_none() {
            return Err(PdfError::MissingEmbeddedFont { font_id: font_id.0 });
        }
    }

    for page in &layout.pages {
        if let Some(background) = &page.background {
            validate_paint(background)?;
        }
        validate_elements(&page.elements)?;
    }
    Ok(())
}

fn validate_elements(elements: &[PositionedElement]) -> Result<(), PdfError> {
    for element in elements {
        match element {
            PositionedElement::Text(run) => validate_color(run.color)?,
            PositionedElement::Line { color, .. } | PositionedElement::FilledRect { color, .. } => {
                validate_color(*color)?
            }
            PositionedElement::LinkAnnotation { url, .. } => {
                let scheme = url
                    .split_once(':')
                    .map(|(scheme, _)| scheme.to_ascii_lowercase());
                if scheme
                    .as_deref()
                    .is_some_and(|scheme| !matches!(scheme, "http" | "https" | "mailto" | "tel"))
                {
                    return Err(PdfError::ForbiddenAction {
                        action: url.clone(),
                    });
                }
            }
            PositionedElement::Path(path) => {
                if let Some(fill) = &path.fill {
                    validate_paint(fill)?;
                }
                if let Some(stroke) = &path.stroke {
                    validate_paint(&stroke.paint)?;
                }
            }
            PositionedElement::Group(group) => {
                if !unit_component(group.opacity) {
                    return Err(PdfError::UnsupportedColor);
                }
                for effect in &group.effects {
                    match effect {
                        oxml_layout::Effect::OuterShadow { color, .. } => validate_color(*color)?,
                        _ => return Err(PdfError::UnsupportedColor),
                    }
                }
                validate_elements(&group.children)?;
            }
            PositionedElement::MarkedContent { children, .. } => validate_elements(children)?,
            PositionedElement::Image { .. } => {}
            _ => return Err(PdfError::UnsupportedColor),
        }
    }
    Ok(())
}

fn validate_paint(paint: &Paint) -> Result<(), PdfError> {
    match paint {
        Paint::Solid(color) => validate_color(*color),
        Paint::Linear { stops, .. } | Paint::Radial { stops, .. } => {
            for stop in stops {
                if !unit_component(stop.offset) {
                    return Err(PdfError::UnsupportedColor);
                }
                validate_color(stop.color)?;
            }
            Ok(())
        }
        Paint::Tile { .. } => Err(PdfError::UnsupportedPaint { paint: "tile" }),
    }
}

fn validate_color(color: Color) -> Result<(), PdfError> {
    if [color.r, color.g, color.b, color.a]
        .into_iter()
        .all(unit_component)
    {
        Ok(())
    } else {
        Err(PdfError::UnsupportedColor)
    }
}

fn unit_component(component: f64) -> bool {
    component.is_finite() && (0.0..=1.0).contains(&component)
}

pub(crate) fn archival_xmp(
    layout: &LayoutResult,
    profile: PdfConformance,
    declares_pdfua: bool,
) -> Vec<u8> {
    let metadata = layout.metadata.as_ref();
    let title = xml_escape(
        metadata
            .and_then(|value| value.title.as_deref())
            .unwrap_or("Untitled document"),
    );
    let author = metadata
        .and_then(|value| value.author.as_deref())
        .map(xml_escape);
    let subject = metadata
        .and_then(|value| value.subject.as_deref())
        .map(xml_escape);
    let keywords = metadata
        .and_then(|value| value.keywords.as_deref())
        .map(xml_escape);
    let creator = xml_escape(
        metadata
            .and_then(|value| value.creator.as_deref())
            .unwrap_or("rdocx-pdf"),
    );
    let mut properties = format!(
        "<dc:title><rdf:Alt><rdf:li xml:lang=\"x-default\">{title}</rdf:li></rdf:Alt></dc:title>\n"
    );
    if let Some(author) = author {
        properties.push_str(&format!(
            "<dc:creator><rdf:Seq><rdf:li>{author}</rdf:li></rdf:Seq></dc:creator>\n"
        ));
    }
    if let Some(subject) = subject {
        properties.push_str(&format!(
            "<dc:description><rdf:Alt><rdf:li xml:lang=\"x-default\">{subject}</rdf:li></rdf:Alt></dc:description>\n"
        ));
    }
    if let Some(keywords) = keywords {
        properties.push_str(&format!("<pdf:Keywords>{keywords}</pdf:Keywords>\n"));
    }
    properties.push_str(&format!(
        "<xmp:CreatorTool>{creator}</xmp:CreatorTool>\n<pdf:Producer>rdocx-pdf</pdf:Producer>\n"
    ));
    properties.push_str(&format!(
        "<pdfaid:part>{}</pdfaid:part>\n<pdfaid:conformance>B</pdfaid:conformance>\n",
        profile.part()
    ));
    if declares_pdfua {
        properties.push_str("<pdfuaid:part>1</pdfuaid:part>\n");
    }
    properties.push_str(
        "<xmpMM:DocumentID>uuid:72646f63-782d-7064-6661-2d3030303031</xmpMM:DocumentID>\n<xmpMM:InstanceID>uuid:72646f63-782d-7064-6661-2d3030303031</xmpMM:InstanceID>",
    );
    let extension = if declares_pdfua {
        r#"
<rdf:Description rdf:about="" xmlns:pdfaExtension="http://www.aiim.org/pdfa/ns/extension/" xmlns:pdfaSchema="http://www.aiim.org/pdfa/ns/schema#" xmlns:pdfaProperty="http://www.aiim.org/pdfa/ns/property#">
<pdfaExtension:schemas><rdf:Bag><rdf:li rdf:parseType="Resource">
<pdfaSchema:schema>PDF/UA identification schema</pdfaSchema:schema>
<pdfaSchema:namespaceURI>http://www.aiim.org/pdfua/ns/id/</pdfaSchema:namespaceURI>
<pdfaSchema:prefix>pdfuaid</pdfaSchema:prefix>
<pdfaSchema:property><rdf:Seq><rdf:li rdf:parseType="Resource">
<pdfaProperty:name>part</pdfaProperty:name>
<pdfaProperty:valueType>Integer</pdfaProperty:valueType>
<pdfaProperty:category>internal</pdfaProperty:category>
<pdfaProperty:description>PDF/UA version identifier</pdfaProperty:description>
</rdf:li></rdf:Seq></pdfaSchema:property>
</rdf:li></rdf:Bag></pdfaExtension:schemas>
</rdf:Description>"#
    } else {
        ""
    };
    format!(
        r#"<?xpacket begin="﻿" id="W5M0MpCehiHzreSzNTczkc9d"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
<rdf:Description rdf:about="" xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:xmp="http://ns.adobe.com/xap/1.0/" xmlns:pdf="http://ns.adobe.com/pdf/1.3/" xmlns:xmpMM="http://ns.adobe.com/xap/1.0/mm/" xmlns:pdfaid="http://www.aiim.org/pdfa/ns/id/" xmlns:pdfuaid="http://www.aiim.org/pdfua/ns/id/">
{properties}
</rdf:Description>
{extension}
</rdf:RDF>
</x:xmpmeta>
<?xpacket end="w"?>"#
    )
    .into_bytes()
}

fn xml_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
