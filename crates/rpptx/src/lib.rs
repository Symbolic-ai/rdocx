//! Public facade for PresentationML packages.
//!
//! The `default-template` feature embeds a 16:9 zero-slide template derived
//! from the MIT-licensed python-pptx 1.0.2 default template. The slide size was
//! changed to 12,192,000 by 6,858,000 EMU, the size kind was changed to
//! `screen16x9`, and python-pptx generated the notes-master infrastructure.

use std::collections::{HashMap, HashSet};
use std::io::Cursor;
use std::path::Path;

use oxml_layout::MediaId;
use oxml_media::{MediaNamer, resolve};
use oxml_opc::content_types;
use oxml_opc::relationship::{Relationship, rel_types};
use oxml_opc::{OpcError, OpcPackage, Relationships};
use rpptx_oxml::graphic_frame::GraphicDataPayload;
use rpptx_oxml::notes_parts::CT_NotesSlide;
use rpptx_oxml::placeholder::{CT_Placeholder, PhType};
use rpptx_oxml::presentation::{CT_Presentation, CT_SlideId};
use rpptx_oxml::shape_tree::{CT_Shape, CT_ShapeTree, ShapeIdAllocator, ShapeTreeChild};
use rpptx_oxml::slide_parts::{CT_Slide, CT_SlideLayout};
use thiserror::Error;

#[cfg(feature = "default-template")]
const DEFAULT_TEMPLATE: &[u8] = include_bytes!("../assets/default.pptx");

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

    #[error("layout index {index} is out of range for {layout_count} layouts")]
    UnknownLayoutIndex { index: usize, layout_count: usize },

    #[error("PowerPoint slide ids are exhausted")]
    SlideIdExhausted,
}

/// An opened PresentationML package and its ordered slide read model.
#[derive(Clone, Debug)]
pub struct Presentation {
    package: OpcPackage,
    media_store: MediaStore,
    presentation_part: String,
    presentation: CT_Presentation,
    layouts: Vec<LayoutRecord>,
    slides: Vec<SlideRecord>,
}

#[derive(Clone, Debug)]
struct LayoutRecord {
    part_name: String,
    layout: CT_SlideLayout,
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

#[derive(Clone, Debug)]
struct MediaEntry {
    part_name: String,
    bytes: Vec<u8>,
    extension: String,
    content_type: String,
    newly_inserted: bool,
}

#[derive(Clone, Debug)]
struct MediaStore {
    by_id: HashMap<MediaId, Vec<MediaEntry>>,
    namer: MediaNamer,
}

impl MediaStore {
    fn scan(package: &OpcPackage) -> Self {
        let namer = MediaNamer::scan(
            "/ppt/media",
            "image",
            package.parts.keys().map(String::as_str),
        );
        let mut by_id: HashMap<MediaId, Vec<MediaEntry>> = HashMap::new();
        let mut parts = package
            .parts
            .iter()
            .filter(|(part_name, _)| part_name.starts_with("/ppt/media/"))
            .collect::<Vec<_>>();
        parts.sort_unstable_by_key(|(part_name, _)| part_name.as_str());
        for (part_name, bytes) in parts {
            let format = resolve(bytes, part_name);
            let extension = part_name
                .rsplit_once('.')
                .map_or_else(|| format.extension(), |(_, extension)| extension)
                .to_owned();
            let content_type = package
                .content_types
                .content_type_for(part_name)
                .unwrap_or_else(|| format.content_type())
                .to_owned();
            by_id
                .entry(MediaId::from_bytes(bytes))
                .or_default()
                .push(MediaEntry {
                    part_name: part_name.clone(),
                    bytes: bytes.clone(),
                    extension,
                    content_type,
                    newly_inserted: false,
                });
        }
        Self { by_id, namer }
    }

    #[allow(dead_code)]
    fn insert(&mut self, package: &mut OpcPackage, bytes: &[u8], filename: &str) -> String {
        self.insert_with_id(package, MediaId::from_bytes(bytes), bytes, filename)
    }

    #[allow(dead_code)]
    fn insert_with_id(
        &mut self,
        package: &mut OpcPackage,
        media_id: MediaId,
        bytes: &[u8],
        filename: &str,
    ) -> String {
        if let Some(existing) = self
            .by_id
            .get(&media_id)
            .and_then(|bucket| bucket.iter().find(|entry| entry.bytes == bytes))
        {
            return existing.part_name.clone();
        }

        let format = resolve(bytes, filename);
        let extension = format.extension();
        let content_type = format.content_type();
        let part_name = self.namer.next_part_name(extension);
        package.set_part(&part_name, bytes.to_vec());
        register_content_type(package, &part_name, extension, content_type);
        self.by_id.entry(media_id).or_default().push(MediaEntry {
            part_name: part_name.clone(),
            bytes: bytes.to_vec(),
            extension: extension.to_owned(),
            content_type: content_type.to_owned(),
            newly_inserted: true,
        });
        part_name
    }

    fn write_new_parts(&self, package: &mut OpcPackage) {
        for entry in self.by_id.values().flatten() {
            if entry.newly_inserted {
                package.set_part(&entry.part_name, entry.bytes.clone());
                register_content_type(
                    package,
                    &entry.part_name,
                    &entry.extension,
                    &entry.content_type,
                );
            }
        }
    }
}

fn register_content_type(
    package: &mut OpcPackage,
    part_name: &str,
    extension: &str,
    content_type: &str,
) {
    match package.content_types.content_type_for(part_name) {
        Some(existing) if existing == content_type => {}
        Some(_) => package.content_types.add_override(part_name, content_type),
        None => package.content_types.add_default(extension, content_type),
    }
}

impl Presentation {
    /// Creates an empty 16:9 presentation from the bundled standard template.
    #[cfg(feature = "default-template")]
    pub fn new() -> Result<Self> {
        Self::from_bytes(DEFAULT_TEMPLATE)
    }

    /// Opens a presentation from a `.pptx` path.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        Self::from_package(OpcPackage::open(path)?)
    }

    /// Opens a presentation from in-memory `.pptx` bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        Self::from_package(OpcPackage::from_reader(Cursor::new(bytes))?)
    }

    fn from_package(package: OpcPackage) -> Result<Self> {
        let media_store = MediaStore::scan(&package);
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
        let layouts = resolve_layouts(&package, &presentation_part, &presentation)?;

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
            media_store,
            presentation_part,
            presentation,
            layouts,
            slides,
        })
    }

    /// Serialises the package through the deterministic OPC writer.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut package = self.package.clone();
        self.media_store.write_new_parts(&mut package);
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

    /// Returns the number of layouts reachable through presentation masters.
    pub fn layout_count(&self) -> usize {
        self.layouts.len()
    }

    /// Returns a layout's optional `p:cSld` name by zero-based index.
    pub fn layout_name(&self, index: usize) -> Option<&str> {
        self.layouts
            .get(index)
            .and_then(|record| record.layout.common_slide_data.name.as_deref())
    }

    /// Synthesizes a new slide from one layout selected by zero-based index.
    pub fn add_slide(&mut self, layout_index: usize) -> Result<SlideRef<'_>> {
        let layout = self
            .layouts
            .get(layout_index)
            .ok_or(Error::UnknownLayoutIndex {
                index: layout_index,
                layout_count: self.layouts.len(),
            })?;
        let layout_part = layout.part_name.clone();
        let placeholders = layout
            .layout
            .common_slide_data
            .shape_tree
            .children
            .iter()
            .filter_map(|child| match child {
                ShapeTreeChild::Shape(shape) => shape.placeholder.as_ref(),
                _ => None,
            })
            .filter(|placeholder| !is_latent_placeholder(placeholder))
            .map(|placeholder| (placeholder.ph_type.clone(), placeholder.idx))
            .collect::<Vec<_>>();

        let mut shape_tree = CT_ShapeTree::new();
        let mut shape_ids = ShapeIdAllocator::scan(&shape_tree);
        for (ph_type, idx) in placeholders {
            let shape =
                CT_Shape::new_placeholder(shape_ids.allocate(), CT_Placeholder::new(ph_type, idx))
                    .map_err(|error| Error::MalformedPart {
                        part_name: "new slide".to_owned(),
                        message: error.to_string(),
                    })?;
            shape_tree.children.push(ShapeTreeChild::Shape(shape));
        }

        let slide_part = MediaNamer::scan(
            "/ppt/slides",
            "slide",
            self.package.parts.keys().map(String::as_str),
        )
        .next_part_name("xml");
        let slide = CT_Slide::new(shape_tree);
        let slide_xml = slide.to_xml().map_err(|error| Error::MalformedPart {
            part_name: slide_part.clone(),
            message: error.to_string(),
        })?;

        let id = self
            .presentation
            .slide_ids
            .iter()
            .map(|slide| slide.id)
            .max()
            .unwrap_or(255)
            .max(255)
            .checked_add(1)
            .ok_or(Error::SlideIdExhausted)?;

        let mut slide_relationships = Relationships::new();
        slide_relationships.add(
            rel_types::SLIDE_LAYOUT,
            &relative_part_target(&slide_part, &layout_part),
        );
        let mut presentation_relationships = self
            .package
            .get_part_rels(&self.presentation_part)
            .cloned()
            .unwrap_or_default();
        let presentation_relationship_id = presentation_relationships.add(
            rel_types::SLIDE,
            &relative_part_target(&self.presentation_part, &slide_part),
        );
        let slide_id = CT_SlideId::new(id, presentation_relationship_id).map_err(|error| {
            Error::MalformedPart {
                part_name: self.presentation_part.clone(),
                message: error.to_string(),
            }
        })?;

        self.package.set_part(&slide_part, slide_xml);
        self.package
            .part_rels
            .insert(slide_part.clone(), slide_relationships);
        self.package
            .part_rels
            .insert(self.presentation_part.clone(), presentation_relationships);
        self.package
            .content_types
            .add_override(&slide_part, content_types::SLIDE);
        self.presentation.slide_ids.push(slide_id);
        self.slides.push(SlideRecord {
            id,
            part_name: slide_part,
            slide,
            notes: None,
        });
        self.slides
            .last()
            .map(slide_ref)
            .ok_or(Error::MalformedPart {
                part_name: self.presentation_part.clone(),
                message: "new slide record was not retained".to_owned(),
            })
    }
}

fn resolve_layouts(
    package: &OpcPackage,
    presentation_part: &str,
    presentation: &CT_Presentation,
) -> Result<Vec<LayoutRecord>> {
    let presentation_relationships = package.get_part_rels(presentation_part);
    let mut layouts = Vec::new();
    let mut seen = HashSet::new();
    for master in &presentation.slide_master_ids {
        let relationship = presentation_relationships
            .and_then(|relationships| relationships.get_by_id(&master.relationship_id))
            .ok_or_else(|| Error::MissingRelationship {
                source_part: presentation_part.to_owned(),
                relationship_id: master.relationship_id.clone(),
            })?;
        require_relationship_type(presentation_part, relationship, rel_types::SLIDE_MASTER)?;
        reject_external(presentation_part, relationship)?;
        let master_part = OpcPackage::resolve_rel_target(presentation_part, &relationship.target);
        required_part(package, &master_part)?;
        let Some(master_relationships) = package.get_part_rels(&master_part) else {
            continue;
        };
        for relationship in master_relationships
            .items
            .iter()
            .filter(|relationship| relationship.rel_type == rel_types::SLIDE_LAYOUT)
        {
            reject_external(&master_part, relationship)?;
            let part_name = OpcPackage::resolve_rel_target(&master_part, &relationship.target);
            if !seen.insert(part_name.clone()) {
                continue;
            }
            let xml = required_part(package, &part_name)?;
            let layout = CT_SlideLayout::from_xml(xml).map_err(|error| Error::MalformedPart {
                part_name: part_name.clone(),
                message: error.to_string(),
            })?;
            layouts.push(LayoutRecord { part_name, layout });
        }
    }
    Ok(layouts)
}

fn is_latent_placeholder(placeholder: &CT_Placeholder) -> bool {
    matches!(
        placeholder.ph_type.as_ref(),
        Some(PhType::DateTime | PhType::Footer | PhType::SlideNumber)
    )
}

fn relative_part_target(source_part: &str, target_part: &str) -> String {
    let mut source = source_part
        .trim_start_matches('/')
        .split('/')
        .collect::<Vec<_>>();
    source.pop();
    let target = target_part
        .trim_start_matches('/')
        .split('/')
        .collect::<Vec<_>>();
    let common = source
        .iter()
        .zip(&target)
        .take_while(|(left, right)| left == right)
        .count();
    std::iter::repeat_n("..", source.len() - common)
        .chain(target[common..].iter().copied())
        .collect::<Vec<_>>()
        .join("/")
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

#[cfg(test)]
mod write_tests {
    use std::collections::HashSet;

    use rpptx_oxml::placeholder::PhType;

    use super::*;

    fn layout_index(presentation: &Presentation, name: &str) -> usize {
        (0..presentation.layout_count())
            .find(|index| presentation.layout_name(*index) == Some(name))
            .unwrap_or_else(|| panic!("missing layout {name}"))
    }

    #[test]
    fn three_added_slides_have_unique_ids_and_reopen() {
        let mut presentation = Presentation::new().unwrap();
        for _ in 0..3 {
            presentation.add_slide(0).unwrap();
        }

        let ids = presentation
            .slides()
            .map(|slide| slide.id())
            .collect::<HashSet<_>>();
        assert_eq!(ids.len(), 3);
        assert!(ids.iter().all(|id| *id >= 256));
        assert_eq!(
            Presentation::from_bytes(&presentation.to_bytes().unwrap())
                .unwrap()
                .len(),
            3
        );
    }

    #[test]
    fn add_slide_allocates_after_the_highest_existing_part_suffix() {
        let mut presentation = Presentation::new().unwrap();
        let sentinel = b"existing sparse slide".to_vec();
        presentation
            .package
            .set_part("/ppt/slides/slide7.xml", sentinel.clone());

        presentation.add_slide(0).unwrap();

        assert_eq!(
            presentation.slides.last().unwrap().part_name,
            "/ppt/slides/slide8.xml"
        );
        assert_eq!(
            presentation.package.get_part("/ppt/slides/slide7.xml"),
            Some(sentinel.as_slice())
        );
    }

    #[test]
    fn add_slide_synthesizes_only_non_latent_layout_placeholders() {
        let mut presentation = Presentation::new().unwrap();
        let layout_index = layout_index(&presentation, "Title and Content");
        let expected = presentation.layouts[layout_index]
            .layout
            .common_slide_data
            .shape_tree
            .children
            .iter()
            .filter_map(|child| match child {
                ShapeTreeChild::Shape(shape) => shape.placeholder.as_ref(),
                _ => None,
            })
            .filter(|placeholder| {
                !matches!(
                    placeholder.ph_type,
                    Some(PhType::DateTime | PhType::Footer | PhType::SlideNumber)
                )
            })
            .map(|placeholder| (placeholder.ph_type.clone(), placeholder.idx))
            .collect::<Vec<_>>();

        presentation.add_slide(layout_index).unwrap();

        let actual = presentation.slides[0]
            .slide
            .common_slide_data
            .shape_tree
            .children
            .iter()
            .filter_map(|child| match child {
                ShapeTreeChild::Shape(shape) => shape.placeholder.as_ref(),
                _ => None,
            })
            .map(|placeholder| (placeholder.ph_type.clone(), placeholder.idx))
            .collect::<Vec<_>>();
        assert_eq!(actual, expected);
    }

    #[test]
    fn synthesized_slide_uses_schema_order_and_one_relative_layout_relationship() {
        let mut presentation = Presentation::new().unwrap();
        let relocated_layout = "/custom/layouts/relocated.xml";
        let original_layout = presentation.layouts[0].part_name.clone();
        let layout_xml = presentation
            .package
            .get_part(&original_layout)
            .unwrap()
            .to_vec();
        presentation.package.set_part(relocated_layout, layout_xml);
        presentation.layouts[0].part_name = relocated_layout.to_owned();
        presentation.add_slide(0).unwrap();
        let record = &presentation.slides[0];
        let xml = String::from_utf8(record.slide.to_xml().unwrap()).unwrap();
        let relationships = presentation
            .package
            .get_part_rels(&record.part_name)
            .unwrap();

        assert!(xml.find("<p:cSld").unwrap() < xml.find("<p:clrMapOvr").unwrap());
        assert_eq!(relationships.items.len(), 1);
        assert_eq!(relationships.items[0].rel_type, rel_types::SLIDE_LAYOUT);
        assert!(!relationships.items[0].target.starts_with('/'));
        assert_eq!(
            OpcPackage::resolve_rel_target(&record.part_name, &relationships.items[0].target),
            relocated_layout
        );
        assert_eq!(CT_Slide::from_xml(xml.as_bytes()).unwrap(), record.slide);
    }

    #[test]
    fn synthesized_text_bodies_always_contain_a_paragraph() {
        let mut presentation = Presentation::new().unwrap();
        let layout_index = layout_index(&presentation, "Title and Content");
        presentation.add_slide(layout_index).unwrap();

        for child in &presentation.slides[0]
            .slide
            .common_slide_data
            .shape_tree
            .children
        {
            if let ShapeTreeChild::Shape(shape) = child
                && shape.text_body.is_some()
            {
                assert!(
                    String::from_utf8(shape.to_xml().unwrap())
                        .unwrap()
                        .contains("<a:p")
                );
            }
        }
    }

    #[test]
    fn add_slide_rejects_an_unknown_layout_index_without_mutation() {
        let mut presentation = Presentation::new().unwrap();
        let before = presentation.to_bytes().unwrap();

        let error = presentation
            .add_slide(presentation.layout_count())
            .err()
            .expect("unknown layout must fail");

        assert!(matches!(error, Error::UnknownLayoutIndex { .. }));
        assert_eq!(presentation.to_bytes().unwrap(), before);
    }

    #[test]
    fn equal_media_bytes_inserted_twice_reuse_one_part() {
        let mut package = OpcPackage::new();
        let mut media = MediaStore::scan(&package);
        let png = b"\x89PNG\r\n\x1a\nfixture";

        let first = media.insert(&mut package, png, "first.png");
        let second = media.insert(&mut package, png, "second.png");

        assert_eq!(first, second);
        assert_eq!(
            package
                .parts
                .keys()
                .filter(|part| part.starts_with("/ppt/media/"))
                .count(),
            1
        );
    }

    #[test]
    fn media_store_compares_bytes_inside_a_hash_bucket() {
        let mut package = OpcPackage::new();
        let mut media = MediaStore::scan(&package);
        let first = b"\x89PNG\r\n\x1a\nfirst";
        let second = b"\x89PNG\r\n\x1a\nsecond";

        let first_part = media.insert_with_id(&mut package, MediaId(7), first, "first.png");
        let second_part = media.insert_with_id(&mut package, MediaId(7), second, "second.png");

        assert_ne!(first_part, second_part);
        assert_eq!(package.get_part(&first_part), Some(first.as_slice()));
        assert_eq!(package.get_part(&second_part), Some(second.as_slice()));
    }

    #[test]
    fn media_store_allocates_after_the_highest_existing_suffix() {
        let mut package = OpcPackage::new();
        package.set_part("/ppt/media/image1.png", b"\x89PNG\r\n\x1a\none".to_vec());
        package.set_part("/ppt/media/image3.png", b"\x89PNG\r\n\x1a\nthree".to_vec());
        package.content_types.add_default("png", "image/png");
        let mut media = MediaStore::scan(&package);

        let part = media.insert(&mut package, b"\x89PNG\r\n\x1a\nfour", "misleading.jpeg");

        assert_eq!(part, "/ppt/media/image4.png");
        assert_eq!(
            package.content_types.content_type_for(&part),
            Some("image/png")
        );
    }

    #[test]
    fn media_store_overrides_a_conflicting_existing_default() {
        let mut package = OpcPackage::new();
        package
            .content_types
            .add_default("png", "application/octet-stream");
        let mut media = MediaStore::scan(&package);

        let part = media.insert(&mut package, b"\x89PNG\r\n\x1a\nnew", "new.png");

        assert_eq!(
            package.content_types.content_type_for(&part),
            Some("image/png")
        );
        assert_eq!(
            package
                .content_types
                .defaults
                .get("png")
                .map(String::as_str),
            Some("application/octet-stream")
        );
    }

    #[test]
    fn media_store_reuses_the_first_sorted_duplicate_part() {
        let mut package = OpcPackage::new();
        let png = b"\x89PNG\r\n\x1a\nduplicate";
        package.set_part("/ppt/media/image2.png", png.to_vec());
        package.set_part("/ppt/media/image1.png", png.to_vec());
        let mut media = MediaStore::scan(&package);

        let part = media.insert(&mut package, png, "duplicate.png");

        assert_eq!(part, "/ppt/media/image1.png");
    }
}
