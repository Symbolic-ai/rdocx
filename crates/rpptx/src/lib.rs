//! Public read facade for PresentationML packages.

use std::io::Cursor;
use std::path::Path;

use oxml_opc::relationship::{Relationship, rel_types};
use oxml_opc::{OpcError, OpcPackage};
use rpptx_oxml::graphic_frame::GraphicDataPayload;
use rpptx_oxml::notes_parts::CT_NotesSlide;
use rpptx_oxml::presentation::CT_Presentation;
use rpptx_oxml::shape_tree::ShapeTreeChild;
use rpptx_oxml::slide_parts::CT_Slide;
use thiserror::Error;

/// A result returned by the `rpptx` read facade.
pub type Result<T> = std::result::Result<T, Error>;

/// An error opening, resolving, or serialising a presentation package.
#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Package(#[from] OpcError),

    #[error("presentation package has no officeDocument relationship")]
    MissingMainDocument,

    #[error("{source_part}: relationship {relationship_id} is missing")]
    MissingRelationship {
        source_part: String,
        relationship_id: String,
    },

    #[error("{source_part}: relationship {relationship_id} has type {actual}, expected {expected}")]
    WrongRelationshipType {
        source_part: String,
        relationship_id: String,
        expected: &'static str,
        actual: String,
    },

    #[error("{source_part}: relationship {relationship_id} has an external target")]
    ExternalRelationship {
        source_part: String,
        relationship_id: String,
    },

    #[error("package part is missing: {part_name}")]
    MissingPart { part_name: String },

    #[error("{source_part}: found {count} notes-slide relationships, expected at most one")]
    DuplicateNotesSlides { source_part: String, count: usize },

    #[error("malformed PresentationML part {part_name}: {message}")]
    MalformedPart { part_name: String, message: String },
}

/// An opened PresentationML package and its ordered slide read model.
#[derive(Clone, Debug)]
pub struct Presentation {
    package: OpcPackage,
    presentation_part: String,
    presentation: CT_Presentation,
    slides: Vec<SlideRecord>,
}

#[derive(Clone, Debug)]
struct SlideRecord {
    id: u32,
    part_name: String,
    slide: CT_Slide,
    notes: Option<NotesRecord>,
}

#[derive(Clone, Debug)]
struct NotesRecord {
    part_name: String,
    notes: CT_NotesSlide,
}

impl Presentation {
    /// Opens a presentation from a `.pptx` path.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        Self::from_package(OpcPackage::open(path)?)
    }

    /// Opens a presentation from in-memory `.pptx` bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        Self::from_package(OpcPackage::from_reader(Cursor::new(bytes))?)
    }

    fn from_package(package: OpcPackage) -> Result<Self> {
        let main_relationship = package
            .package_rels
            .get_by_type(rel_types::DOCUMENT)
            .ok_or(Error::MissingMainDocument)?;
        reject_external("/", main_relationship)?;
        let presentation_part = OpcPackage::resolve_rel_target("/", &main_relationship.target);
        let presentation_xml = required_part(&package, &presentation_part)?;
        let presentation =
            CT_Presentation::from_xml(presentation_xml).map_err(|error| Error::MalformedPart {
                part_name: presentation_part.clone(),
                message: error.to_string(),
            })?;

        let presentation_relationships = package.get_part_rels(&presentation_part);
        let mut slides = Vec::with_capacity(presentation.slide_ids.len());
        for slide_id in &presentation.slide_ids {
            let relationship = presentation_relationships
                .and_then(|relationships| relationships.get_by_id(&slide_id.relationship_id))
                .ok_or_else(|| Error::MissingRelationship {
                    source_part: presentation_part.clone(),
                    relationship_id: slide_id.relationship_id.clone(),
                })?;
            require_relationship_type(&presentation_part, relationship, rel_types::SLIDE)?;
            reject_external(&presentation_part, relationship)?;
            let slide_part =
                OpcPackage::resolve_rel_target(&presentation_part, &relationship.target);
            let slide_xml = required_part(&package, &slide_part)?;
            let slide = CT_Slide::from_xml(slide_xml).map_err(|error| Error::MalformedPart {
                part_name: slide_part.clone(),
                message: error.to_string(),
            })?;
            let notes = resolve_notes(&package, &slide_part)?;
            slides.push(SlideRecord {
                id: slide_id.id,
                part_name: slide_part,
                slide,
                notes,
            });
        }

        Ok(Self {
            package,
            presentation_part,
            presentation,
            slides,
        })
    }

    /// Serialises the package through the deterministic OPC writer.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut package = self.package.clone();
        package.set_part(
            &self.presentation_part,
            self.presentation
                .to_xml()
                .map_err(|error| Error::MalformedPart {
                    part_name: self.presentation_part.clone(),
                    message: error.to_string(),
                })?,
        );
        for record in &self.slides {
            package.set_part(
                &record.part_name,
                record
                    .slide
                    .to_xml()
                    .map_err(|error| Error::MalformedPart {
                        part_name: record.part_name.clone(),
                        message: error.to_string(),
                    })?,
            );
            if let Some(notes) = &record.notes {
                package.set_part(
                    &notes.part_name,
                    notes.notes.to_xml().map_err(|error| Error::MalformedPart {
                        part_name: notes.part_name.clone(),
                        message: error.to_string(),
                    })?,
                );
            }
        }
        let mut output = Cursor::new(Vec::new());
        package.write_to(&mut output)?;
        Ok(output.into_inner())
    }

    /// Returns the number of slides in presentation order.
    pub fn len(&self) -> usize {
        self.slides.len()
    }

    /// Returns whether the presentation has no slides.
    pub fn is_empty(&self) -> bool {
        self.slides.is_empty()
    }

    /// Returns one slide by zero-based presentation index.
    pub fn slide(&self, index: usize) -> Option<SlideRef<'_>> {
        self.slides.get(index).map(slide_ref)
    }

    /// Iterates slides in `p:sldIdLst` order.
    pub fn slides(&self) -> impl ExactSizeIterator<Item = SlideRef<'_>> {
        self.slides.iter().map(slide_ref)
    }
}

fn required_part<'a>(package: &'a OpcPackage, part_name: &str) -> Result<&'a [u8]> {
    package
        .get_part(part_name)
        .ok_or_else(|| Error::MissingPart {
            part_name: part_name.to_owned(),
        })
}

fn require_relationship_type(
    source_part: &str,
    relationship: &Relationship,
    expected: &'static str,
) -> Result<()> {
    if relationship.rel_type == expected {
        return Ok(());
    }
    Err(Error::WrongRelationshipType {
        source_part: source_part.to_owned(),
        relationship_id: relationship.id.clone(),
        expected,
        actual: relationship.rel_type.clone(),
    })
}

fn reject_external(source_part: &str, relationship: &Relationship) -> Result<()> {
    if relationship
        .target_mode
        .as_deref()
        .is_some_and(|mode| mode.eq_ignore_ascii_case("external"))
    {
        return Err(Error::ExternalRelationship {
            source_part: source_part.to_owned(),
            relationship_id: relationship.id.clone(),
        });
    }
    Ok(())
}

fn resolve_notes(package: &OpcPackage, slide_part: &str) -> Result<Option<NotesRecord>> {
    let Some(relationships) = package.get_part_rels(slide_part) else {
        return Ok(None);
    };
    let notes_relationships = relationships.get_all_by_type(rel_types::NOTES_SLIDE);
    if notes_relationships.len() > 1 {
        return Err(Error::DuplicateNotesSlides {
            source_part: slide_part.to_owned(),
            count: notes_relationships.len(),
        });
    }
    let Some(relationship) = notes_relationships.first() else {
        return Ok(None);
    };
    reject_external(slide_part, relationship)?;
    let part_name = OpcPackage::resolve_rel_target(slide_part, &relationship.target);
    let xml = required_part(package, &part_name)?;
    let notes = CT_NotesSlide::from_xml(xml).map_err(|error| Error::MalformedPart {
        part_name: part_name.clone(),
        message: error.to_string(),
    })?;
    Ok(Some(NotesRecord { part_name, notes }))
}

/// A borrowed slide in presentation order.
#[derive(Clone, Copy)]
pub struct SlideRef<'a> {
    record: &'a SlideRecord,
}

fn slide_ref(record: &SlideRecord) -> SlideRef<'_> {
    SlideRef { record }
}

impl SlideRef<'_> {
    /// Returns the producer-assigned slide id.
    pub fn id(&self) -> u32 {
        self.record.id
    }

    /// Returns the optional `p:cSld` name.
    pub fn name(&self) -> Option<&str> {
        self.record.slide.common_slide_data.name.as_deref()
    }

    /// Returns one immediate z-order child by zero-based index.
    pub fn shape(&self, index: usize) -> Option<ShapeRef<'_>> {
        self.record
            .slide
            .common_slide_data
            .shape_tree
            .children
            .get(index)
            .map(shape_ref)
    }

    /// Iterates the immediate shape tree in z-order.
    pub fn shapes(&self) -> impl ExactSizeIterator<Item = ShapeRef<'_>> {
        self.record
            .slide
            .common_slide_data
            .shape_tree
            .children
            .iter()
            .map(shape_ref)
    }

    /// Returns visible shape and table text recursively in z-order.
    pub fn text(&self) -> String {
        let mut text = Vec::new();
        collect_text(
            &self.record.slide.common_slide_data.shape_tree.children,
            &mut text,
        );
        text.join("\n")
    }

    /// Returns speaker-note text, distinguishing no notes part from empty notes.
    pub fn notes_text(&self) -> Option<String> {
        self.record
            .notes
            .as_ref()
            .map(|notes| notes.notes.notes_text())
    }
}

/// The structural kind of one PresentationML shape-tree child.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShapeKind {
    Shape,
    Picture,
    GraphicFrame,
    Group,
    Connector,
    AlternateContent,
}

/// A borrowed shape-tree child.
#[derive(Clone, Copy)]
pub struct ShapeRef<'a> {
    child: &'a ShapeTreeChild,
}

fn shape_ref(child: &ShapeTreeChild) -> ShapeRef<'_> {
    ShapeRef { child }
}

impl<'a> ShapeRef<'a> {
    /// Returns the child's normalized structural kind.
    pub fn kind(&self) -> ShapeKind {
        match self.child {
            ShapeTreeChild::Shape(_) => ShapeKind::Shape,
            ShapeTreeChild::Picture(_) => ShapeKind::Picture,
            ShapeTreeChild::GraphicFrame(_) => ShapeKind::GraphicFrame,
            ShapeTreeChild::GroupShape(_) => ShapeKind::Group,
            ShapeTreeChild::Connector(_) => ShapeKind::Connector,
            ShapeTreeChild::AlternateContent(_) => ShapeKind::AlternateContent,
        }
    }

    /// Returns ordinary shape text or row-major table text when modelled.
    pub fn text(&self) -> Option<String> {
        match self.child {
            ShapeTreeChild::Shape(shape) => shape.text_body.as_ref().map(|text| text.plain_text()),
            ShapeTreeChild::GraphicFrame(frame) => match &frame.graphic_data.payload {
                GraphicDataPayload::Table(table) => Some(
                    table
                        .rows
                        .iter()
                        .map(|row| {
                            row.cells
                                .iter()
                                .map(|cell| {
                                    cell.text_body
                                        .as_ref()
                                        .map_or_else(String::new, |text| text.plain_text())
                                })
                                .collect::<Vec<_>>()
                                .join("\t")
                        })
                        .collect::<Vec<_>>()
                        .join("\n"),
                ),
                _ => None,
            },
            _ => None,
        }
    }

    /// Returns the number of immediate group or selected fallback children.
    pub fn child_count(&self) -> usize {
        self.child_slice().len()
    }

    /// Returns one immediate group or selected fallback child.
    pub fn child(&self, index: usize) -> Option<ShapeRef<'a>> {
        self.child_slice().get(index).map(shape_ref)
    }

    /// Iterates immediate group or selected fallback children.
    pub fn children(&self) -> impl ExactSizeIterator<Item = ShapeRef<'a>> + 'a {
        self.child_slice().iter().map(shape_ref)
    }

    fn child_slice(&self) -> &'a [ShapeTreeChild] {
        match self.child {
            ShapeTreeChild::GroupShape(group) => &group.children,
            ShapeTreeChild::AlternateContent(alternate) => {
                alternate.selected_fallback().unwrap_or_default()
            }
            _ => &[],
        }
    }
}

fn collect_text(children: &[ShapeTreeChild], output: &mut Vec<String>) {
    for child in children {
        let shape = shape_ref(child);
        if let Some(text) = shape.text()
            && !text.is_empty()
        {
            output.push(text);
        }
        collect_text(shape.child_slice(), output);
    }
}
