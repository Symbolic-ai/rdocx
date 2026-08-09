//! Public facade for PresentationML packages.
//!
//! The `default-template` feature embeds a 16:9 zero-slide template derived
//! from the MIT-licensed python-pptx 1.0.2 default template. The slide size was
//! changed to 12,192,000 by 6,858,000 EMU, the size kind was changed to
//! `screen16x9`, and python-pptx generated the notes-master infrastructure.

use std::collections::{HashMap, HashSet};
use std::io::Cursor;
use std::path::Path;

use oxml_core::OxmlError;
pub use oxml_core::units::{Angle, Emu};
pub use oxml_drawing::fill::Fill;
pub use oxml_drawing::line::CT_LineProperties;
use oxml_drawing::shape_props::CT_ShapeProperties;
use oxml_drawing::table::{CT_Table, CT_TableCell, CT_TableCellProperties, CT_TableProperties};
use oxml_drawing::text::{CT_RegularTextRun, CT_TextBody, CT_TextParagraph};
pub use oxml_drawing::text::{
    CT_TextCharacterProperties, CT_TextParagraphProperties, TextBullet, TextBulletCharacter,
    TextBulletChoice, TextFont,
};
use oxml_drawing::xfrm::{CT_Point2D, CT_PositiveSize2D, CT_Transform2D};
use oxml_layout::MediaId;
use oxml_media::{ImageFormat, MediaNamer, probe, resolve};
use oxml_opc::content_types;
use oxml_opc::relationship::{Relationship, rel_types};
use oxml_opc::{OpcError, OpcPackage, Relationships};
use rpptx_oxml::connector::CT_ConnectionShape;
use rpptx_oxml::graphic_frame::{CT_GraphicFrame, GraphicDataPayload};
use rpptx_oxml::notes_parts::CT_NotesSlide;
use rpptx_oxml::picture::CT_Picture;
use rpptx_oxml::placeholder::{CT_Placeholder, PhType};
use rpptx_oxml::presentation::{CT_Presentation, CT_SlideId, custom_show_relationship_ids};
use rpptx_oxml::relmap::relationship_ids;
use rpptx_oxml::shape_tree::{
    CT_GroupShape, CT_Shape, CT_ShapeTree, ShapeIdAllocator, ShapeTreeChild,
};
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

    #[error("slide index {index} is out of range for {slide_count} slides")]
    UnknownSlideIndex { index: usize, slide_count: usize },

    #[error("picture {filename} has unsupported image bytes")]
    UnsupportedPicture { filename: String },

    #[error("picture {filename} has no usable native dimensions")]
    UnavailablePictureDimensions { filename: String },

    #[error("picture {filename} has an inferred {axis} outside the EMU range")]
    PictureDimensionOutOfRange {
        filename: String,
        axis: &'static str,
    },

    #[error("PowerPoint slide ids are exhausted")]
    SlideIdExhausted,

    #[error("{operation} is not supported for {shape_kind:?}")]
    UnsupportedShapeMutation {
        operation: &'static str,
        shape_kind: ShapeKind,
    },

    #[error("{operation} failed: {message}")]
    InvalidShapeMutation {
        operation: &'static str,
        message: String,
    },

    #[error("{operation} failed: {message}")]
    InvalidTableMutation {
        operation: &'static str,
        message: String,
    },

    #[error("adjustment {name} requires a finite value")]
    NonFiniteAdjustmentValue { name: String },
}

/// A package or presentation invariant that would make a saved deck unsafe.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ValidationIssue {
    DuplicateShapeId {
        slide: usize,
        id: u32,
    },
    SlideIdOutOfRange {
        slide: usize,
        id: u32,
    },
    DuplicateSlideId {
        id: u32,
    },
    MissingContentTypeOverride {
        part: String,
    },
    DanglingRelationship {
        part: String,
        r_id: String,
        target: String,
    },
    UnreachableRelationshipTarget {
        part: String,
        target: String,
    },
    EmptyTextBody {
        slide: usize,
        shape: usize,
    },
    DuplicatePlaceholderIdx {
        slide: usize,
        idx: u32,
    },
    OrphanMedia {
        part: String,
    },
    CustomShowReference {
        slide_id: u32,
    },
    MissingLayoutRel {
        slide: usize,
    },
    MissingThemeRel {
        master: usize,
    },
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
        debug_assert!(
            self.validate().is_empty(),
            "invalid presentation at byte save boundary: {:?}",
            self.validate()
        );
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

    /// Saves the deterministic package bytes to a `.pptx` path.
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        debug_assert!(
            self.validate().is_empty(),
            "invalid presentation at path save boundary: {:?}",
            self.validate()
        );
        std::fs::write(path, self.to_bytes()?).map_err(OpcError::from)?;
        Ok(())
    }

    /// Returns every detected package and PresentationML invariant violation.
    pub fn validate(&self) -> Vec<ValidationIssue> {
        let mut duplicate_shape_ids = Vec::new();
        let mut out_of_range_slide_ids = Vec::new();
        let mut duplicate_slide_ids = Vec::new();
        let mut missing_content_types = Vec::new();
        let mut dangling_relationships = Vec::new();
        let mut unreachable_targets = Vec::new();
        let mut empty_text_bodies = Vec::new();
        let mut duplicate_placeholder_indices = Vec::new();
        let mut orphan_media = Vec::new();
        let mut custom_show_references = Vec::new();
        let mut missing_layout_relationships = Vec::new();
        let mut missing_theme_relationships = Vec::new();

        for (slide_index, record) in self.slides.iter().enumerate() {
            let mut shape_ids = record
                .slide
                .common_slide_data
                .shape_tree
                .non_visual_id()
                .into_iter()
                .collect();
            let mut placeholder_indices = HashSet::new();
            let mut shape_index = 0usize;
            validate_shape_children(
                slide_index,
                &record.slide.common_slide_data.shape_tree.children,
                &mut shape_index,
                &mut shape_ids,
                &mut placeholder_indices,
                &mut duplicate_shape_ids,
                &mut empty_text_bodies,
                &mut duplicate_placeholder_indices,
            );
        }

        let mut slide_ids = HashSet::new();
        for (slide_index, slide) in self.presentation.slide_ids.iter().enumerate() {
            if !(256..=2_147_483_647).contains(&slide.id) {
                out_of_range_slide_ids.push(ValidationIssue::SlideIdOutOfRange {
                    slide: slide_index,
                    id: slide.id,
                });
            }
            if !slide_ids.insert(slide.id) {
                duplicate_slide_ids.push(ValidationIssue::DuplicateSlideId { id: slide.id });
            }
        }

        let mut part_names = self.package.parts.keys().collect::<Vec<_>>();
        part_names.sort_unstable();
        for part_name in part_names {
            if !part_has_content_type(&self.package, part_name) {
                missing_content_types.push(ValidationIssue::MissingContentTypeOverride {
                    part: part_name.clone(),
                });
            }
        }

        let mut incoming_targets = HashSet::new();
        validate_relationship_scope(
            &self.package,
            "/",
            None,
            &self.package.package_rels,
            &mut incoming_targets,
            &mut dangling_relationships,
            &mut unreachable_targets,
        );
        let mut relationship_sources = self
            .package
            .parts
            .keys()
            .chain(self.package.part_rels.keys())
            .map(String::as_str)
            .collect::<Vec<_>>();
        relationship_sources.sort_unstable();
        relationship_sources.dedup();
        let empty_relationships = Relationships::new();
        for source_part in relationship_sources {
            let relationships = self
                .package
                .get_part_rels(source_part)
                .unwrap_or(&empty_relationships);
            let source_xml = self.current_part_xml(source_part);
            validate_relationship_scope(
                &self.package,
                source_part,
                source_xml.as_deref(),
                relationships,
                &mut incoming_targets,
                &mut dangling_relationships,
                &mut unreachable_targets,
            );
        }

        let mut media_parts = self
            .package
            .parts
            .keys()
            .filter(|part| part.starts_with("/ppt/media/"))
            .collect::<Vec<_>>();
        media_parts.sort_unstable();
        for part in media_parts {
            if !incoming_targets.contains(part.as_str()) {
                orphan_media.push(ValidationIssue::OrphanMedia { part: part.clone() });
            }
        }

        let presentation_relationships = self.package.get_part_rels(&self.presentation_part);
        let slide_relationship_ids = self
            .presentation
            .slide_ids
            .iter()
            .map(|slide| slide.relationship_id.as_str())
            .collect::<HashSet<_>>();
        if let Some(xml) = self.current_part_xml(&self.presentation_part)
            && let Ok(references) = custom_show_relationship_ids(&xml)
        {
            for relationship_id in references {
                if !slide_relationship_ids.contains(relationship_id.as_str()) {
                    custom_show_references.push(ValidationIssue::CustomShowReference {
                        slide_id: numeric_relationship_id(&relationship_id),
                    });
                }
            }
        }

        for (slide_index, slide) in self.slides.iter().enumerate() {
            let count = self
                .package
                .get_part_rels(&slide.part_name)
                .map(|relationships| {
                    relationships
                        .items
                        .iter()
                        .filter(|relationship| {
                            relationship.rel_type == rel_types::SLIDE_LAYOUT
                                && !relationship_is_external(relationship)
                        })
                        .count()
                })
                .unwrap_or_default();
            if count != 1 {
                missing_layout_relationships
                    .push(ValidationIssue::MissingLayoutRel { slide: slide_index });
            }
        }

        for (master_index, master) in self.presentation.slide_master_ids.iter().enumerate() {
            let Some(relationship) = presentation_relationships
                .and_then(|relationships| relationships.get_by_id(&master.relationship_id))
            else {
                missing_theme_relationships.push(ValidationIssue::MissingThemeRel {
                    master: master_index,
                });
                continue;
            };
            let master_part =
                OpcPackage::resolve_rel_target(&self.presentation_part, &relationship.target);
            let has_theme = self
                .package
                .get_part_rels(&master_part)
                .is_some_and(|relationships| {
                    relationships.items.iter().any(|relationship| {
                        relationship.rel_type == rel_types::THEME
                            && !relationship_is_external(relationship)
                    })
                });
            if !has_theme {
                missing_theme_relationships.push(ValidationIssue::MissingThemeRel {
                    master: master_index,
                });
            }
        }

        duplicate_shape_ids
            .into_iter()
            .chain(out_of_range_slide_ids)
            .chain(duplicate_slide_ids)
            .chain(missing_content_types)
            .chain(dangling_relationships)
            .chain(unreachable_targets)
            .chain(empty_text_bodies)
            .chain(duplicate_placeholder_indices)
            .chain(orphan_media)
            .chain(custom_show_references)
            .chain(missing_layout_relationships)
            .chain(missing_theme_relationships)
            .collect()
    }

    fn current_part_xml(&self, part_name: &str) -> Option<Vec<u8>> {
        if part_name == self.presentation_part {
            return self.presentation.to_xml().ok();
        }
        if let Some(slide) = self
            .slides
            .iter()
            .find(|slide| slide.part_name == part_name)
        {
            return slide.slide.to_xml().ok();
        }
        if let Some(notes) = self
            .slides
            .iter()
            .filter_map(|slide| slide.notes.as_ref())
            .find(|notes| notes.part_name == part_name)
        {
            return notes.notes.to_xml().ok();
        }
        self.package.get_part(part_name).map(<[u8]>::to_vec)
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

    /// Returns one slide for in-place mutation by zero-based index.
    pub fn slide_mut(&mut self, index: usize) -> Option<SlideMut<'_>> {
        self.slides.get_mut(index).map(slide_mut)
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

    /// Appends a relationship-backed picture at the top of one slide's z-order.
    #[allow(clippy::too_many_arguments)]
    pub fn add_picture(
        &mut self,
        slide_index: usize,
        image_data: &[u8],
        image_filename: &str,
        left: Emu,
        top: Emu,
        width: Option<Emu>,
        height: Option<Emu>,
    ) -> Result<ShapeRef<'_>> {
        let slide = self
            .slides
            .get(slide_index)
            .ok_or(Error::UnknownSlideIndex {
                index: slide_index,
                slide_count: self.slides.len(),
            })?;
        let slide_part = slide.part_name.clone();
        let (width, height) = picture_dimensions(image_data, image_filename, width, height)?;
        let id = ShapeIdAllocator::scan(&slide.slide.common_slide_data.shape_tree).allocate();

        let mut package = self.package.clone();
        let mut media_store = self.media_store.clone();
        let media_part = media_store.insert(&mut package, image_data, image_filename);
        let mut relationships = package
            .get_part_rels(&slide_part)
            .cloned()
            .unwrap_or_default();
        let relationship_id = relationships
            .items
            .iter()
            .find(|relationship| {
                relationship.rel_type == rel_types::IMAGE
                    && !relationship_is_external(relationship)
                    && OpcPackage::resolve_rel_target(&slide_part, &relationship.target)
                        == media_part
            })
            .map(|relationship| relationship.id.clone())
            .unwrap_or_else(|| {
                relationships.add(
                    rel_types::IMAGE,
                    &relative_part_target(&slide_part, &media_part),
                )
            });
        let picture = CT_Picture::new(
            id,
            &format!("Picture {id}"),
            &relationship_id,
            positioned_transform(left, top, width, height),
        )
        .map_err(|error| invalid_shape_construction("add picture", error))?;

        package.part_rels.insert(slide_part, relationships);
        self.package = package;
        self.media_store = media_store;
        let tree = &mut self.slides[slide_index].slide.common_slide_data.shape_tree;
        Ok(shape_ref(
            tree.append_child(ShapeTreeChild::Picture(picture)),
        ))
    }
}

fn picture_dimensions(
    image_data: &[u8],
    image_filename: &str,
    width: Option<Emu>,
    height: Option<Emu>,
) -> Result<(Emu, Emu)> {
    if let (Some(width), Some(height)) = (width, height) {
        return Ok((width, height));
    }
    if ImageFormat::sniff(image_data).is_none() {
        return Err(Error::UnsupportedPicture {
            filename: image_filename.to_owned(),
        });
    }
    let info = probe(image_data).ok_or_else(|| Error::UnavailablePictureDimensions {
        filename: image_filename.to_owned(),
    })?;
    match (width, height) {
        (None, None) => info
            .native_size(72.0)
            .map(|size| (Emu(size.width_emu), Emu(size.height_emu)))
            .ok_or_else(|| Error::UnavailablePictureDimensions {
                filename: image_filename.to_owned(),
            }),
        (Some(width), None) => infer_picture_dimension(
            width,
            info.height_px,
            info.width_px,
            image_filename,
            "height",
        )
        .map(|height| (width, height)),
        (None, Some(height)) => infer_picture_dimension(
            height,
            info.width_px,
            info.height_px,
            image_filename,
            "width",
        )
        .map(|width| (width, height)),
        (Some(_), Some(_)) => unreachable!(),
    }
}

fn infer_picture_dimension(
    provided: Emu,
    inferred_pixels: u32,
    provided_pixels: u32,
    image_filename: &str,
    axis: &'static str,
) -> Result<Emu> {
    let inferred =
        i128::from(provided.0) * i128::from(inferred_pixels) / i128::from(provided_pixels);
    i64::try_from(inferred)
        .map(Emu)
        .map_err(|_| Error::PictureDimensionOutOfRange {
            filename: image_filename.to_owned(),
            axis,
        })
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

#[allow(clippy::too_many_arguments)]
fn validate_shape_children(
    slide_index: usize,
    children: &[ShapeTreeChild],
    shape_index: &mut usize,
    shape_ids: &mut HashSet<u32>,
    placeholder_indices: &mut HashSet<u32>,
    duplicate_shape_ids: &mut Vec<ValidationIssue>,
    empty_text_bodies: &mut Vec<ValidationIssue>,
    duplicate_placeholder_indices: &mut Vec<ValidationIssue>,
) {
    let mut pending = children.iter().rev().collect::<Vec<_>>();
    while let Some(child) = pending.pop() {
        let current_shape_index = *shape_index;
        *shape_index += 1;
        if let Some(id) = child.non_visual_id()
            && !shape_ids.insert(id)
        {
            duplicate_shape_ids.push(ValidationIssue::DuplicateShapeId {
                slide: slide_index,
                id,
            });
        }
        let placeholder = match child {
            ShapeTreeChild::Shape(shape) => {
                if shape
                    .text_body
                    .as_ref()
                    .is_some_and(|body| body.paragraph_count() == 0)
                {
                    empty_text_bodies.push(ValidationIssue::EmptyTextBody {
                        slide: slide_index,
                        shape: current_shape_index,
                    });
                }
                shape.placeholder.as_ref()
            }
            ShapeTreeChild::Picture(picture) => picture.placeholder.as_ref(),
            _ => None,
        };
        if let Some(idx) = placeholder
            .and_then(|placeholder| placeholder.idx)
            .filter(|idx| *idx != u32::MAX)
            && !placeholder_indices.insert(idx)
        {
            duplicate_placeholder_indices.push(ValidationIssue::DuplicatePlaceholderIdx {
                slide: slide_index,
                idx,
            });
        }
        match child {
            ShapeTreeChild::GroupShape(group) => {
                pending.extend(group.children.iter().rev());
            }
            ShapeTreeChild::AlternateContent(alternate) => {
                if let Some(fallback) = alternate.selected_fallback() {
                    pending.extend(fallback.iter().rev());
                }
            }
            _ => {}
        }
    }
}

fn validate_relationship_scope(
    package: &OpcPackage,
    source_part: &str,
    source_xml: Option<&[u8]>,
    relationships: &Relationships,
    incoming_targets: &mut HashSet<String>,
    dangling_relationships: &mut Vec<ValidationIssue>,
    unreachable_targets: &mut Vec<ValidationIssue>,
) {
    let referenced = source_xml.and_then(|xml| relationship_ids(xml).ok());
    let has_relationship_namespace = source_xml.is_some_and(|xml| {
        xml.windows(rpptx_oxml::namespace::R_NS.len())
            .any(|window| window == rpptx_oxml::namespace::R_NS.as_bytes())
    });
    if let Some(referenced) = &referenced {
        let mut reported = HashSet::new();
        for relationship_id in referenced {
            if relationships.get_by_id(relationship_id).is_none()
                && reported.insert(relationship_id)
            {
                dangling_relationships.push(ValidationIssue::DanglingRelationship {
                    part: source_part.to_owned(),
                    r_id: relationship_id.clone(),
                    target: relationship_id.clone(),
                });
            }
        }
    }
    for relationship in &relationships.items {
        if let Some(referenced) = &referenced
            && has_relationship_namespace
            && relationship_requires_xml_reference(&relationship.rel_type)
            && !referenced.iter().any(|id| id == &relationship.id)
        {
            dangling_relationships.push(ValidationIssue::DanglingRelationship {
                part: source_part.to_owned(),
                r_id: relationship.id.clone(),
                target: relationship.target.clone(),
            });
        }
        if relationship_is_external(relationship) {
            continue;
        }
        let target = OpcPackage::resolve_rel_target(source_part, &relationship.target);
        incoming_targets.insert(target.clone());
        if !package.parts.contains_key(&target) {
            unreachable_targets.push(ValidationIssue::UnreachableRelationshipTarget {
                part: source_part.to_owned(),
                target,
            });
        }
    }
}

fn relationship_requires_xml_reference(relationship_type: &str) -> bool {
    matches!(
        relationship_type,
        rel_types::IMAGE | rel_types::HYPERLINK | rel_types::CHART
    ) || relationship_type.ends_with("/audio")
        || relationship_type.ends_with("/video")
        || relationship_type.ends_with("/oleObject")
        || relationship_type.ends_with("/diagramData")
}

fn part_has_content_type(package: &OpcPackage, part_name: &str) -> bool {
    package.content_types.content_type_for(part_name).is_some()
        || part_name.rsplit_once('.').is_some_and(|(_, extension)| {
            package
                .content_types
                .defaults
                .contains_key(&extension.to_ascii_lowercase())
        })
}

fn relationship_is_external(relationship: &Relationship) -> bool {
    relationship
        .target_mode
        .as_deref()
        .is_some_and(|mode| mode.eq_ignore_ascii_case("external"))
}

fn numeric_relationship_id(relationship_id: &str) -> u32 {
    relationship_id
        .strip_prefix("rId")
        .and_then(|value| value.parse().ok())
        .unwrap_or_default()
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

/// A mutably borrowed slide in presentation order.
pub struct SlideMut<'a> {
    record: &'a mut SlideRecord,
}

fn slide_mut(record: &mut SlideRecord) -> SlideMut<'_> {
    SlideMut { record }
}

impl SlideMut<'_> {
    /// Appends a rectangular table at the top of the slide's z-order.
    pub fn add_table(
        &mut self,
        rows: usize,
        columns: usize,
        left: Emu,
        top: Emu,
        width: Emu,
        height: Emu,
    ) -> Result<ShapeMut<'_>> {
        let table = CT_Table::new(rows, columns, width, height)
            .map_err(|error| invalid_table_mutation("add table", error.to_string()))?;
        let tree = &mut self.record.slide.common_slide_data.shape_tree;
        let id = ShapeIdAllocator::scan(tree).allocate();
        let frame = CT_GraphicFrame::new_table(
            id,
            &format!("Table {id}"),
            positioned_transform(left, top, width, height),
            table,
        )
        .map_err(|error| invalid_table_mutation("add table", error.to_string()))?;
        Ok(shape_mut(tree.append_child(ShapeTreeChild::GraphicFrame(
            Box::new(frame),
        ))))
    }

    /// Appends a no-fill textbox at the top of the slide's z-order.
    pub fn add_textbox(
        &mut self,
        left: Emu,
        top: Emu,
        width: Emu,
        height: Emu,
    ) -> Result<ShapeMut<'_>> {
        let tree = &mut self.record.slide.common_slide_data.shape_tree;
        let id = ShapeIdAllocator::scan(tree).allocate();
        let shape = CT_Shape::new_textbox(
            id,
            &format!("TextBox {id}"),
            positioned_transform(left, top, width, height),
        )
        .map_err(|error| invalid_shape_construction("add textbox", error))?;
        Ok(shape_mut(tree.append_child(ShapeTreeChild::Shape(shape))))
    }

    /// Appends an ordinary preset shape at the top of the slide's z-order.
    pub fn add_shape(
        &mut self,
        preset: &str,
        left: Emu,
        top: Emu,
        width: Emu,
        height: Emu,
    ) -> Result<ShapeMut<'_>> {
        let tree = &mut self.record.slide.common_slide_data.shape_tree;
        let id = ShapeIdAllocator::scan(tree).allocate();
        let shape = CT_Shape::new_preset(
            id,
            &format!("Shape {id}"),
            preset,
            positioned_transform(left, top, width, height),
        )
        .map_err(|error| invalid_shape_construction("add shape", error))?;
        Ok(shape_mut(tree.append_child(ShapeTreeChild::Shape(shape))))
    }

    /// Appends a free-standing connector at the top of the slide's z-order.
    pub fn add_connector(
        &mut self,
        connector: ConnectorType,
        begin_x: Emu,
        begin_y: Emu,
        end_x: Emu,
        end_y: Emu,
    ) -> Result<ShapeMut<'_>> {
        let transform = connector_transform(begin_x, begin_y, end_x, end_y)?;
        let tree = &mut self.record.slide.common_slide_data.shape_tree;
        let id = ShapeIdAllocator::scan(tree).allocate();
        let connector = CT_ConnectionShape::new_free_standing(
            id,
            &format!("Connector {id}"),
            connector.preset_name(),
            transform,
        )
        .map_err(|error| invalid_shape_construction("add connector", error))?;
        Ok(shape_mut(
            tree.append_child(ShapeTreeChild::Connector(connector)),
        ))
    }

    /// Appends an empty group at the top of the slide's z-order.
    pub fn add_group_shape(&mut self) -> Result<ShapeMut<'_>> {
        let tree = &mut self.record.slide.common_slide_data.shape_tree;
        let id = ShapeIdAllocator::scan(tree).allocate();
        let group = CT_GroupShape::new_empty(id, &format!("Group {id}"));
        Ok(shape_mut(
            tree.append_child(ShapeTreeChild::GroupShape(Box::new(group))),
        ))
    }

    /// Returns one immediate z-order child as a read-only handle.
    pub fn shape(&self, index: usize) -> Option<ShapeRef<'_>> {
        self.record
            .slide
            .common_slide_data
            .shape_tree
            .children
            .get(index)
            .map(shape_ref)
    }

    /// Returns one immediate z-order child for in-place mutation.
    pub fn shape_mut(&mut self, index: usize) -> Option<ShapeMut<'_>> {
        self.record
            .slide
            .common_slide_data
            .shape_tree
            .children
            .get_mut(index)
            .map(shape_mut)
    }
}

/// The geometry family of a free-standing connector.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConnectorType {
    Straight,
    Elbow,
    Curve,
}

impl ConnectorType {
    const fn preset_name(self) -> &'static str {
        match self {
            Self::Straight => "line",
            Self::Elbow => "bentConnector3",
            Self::Curve => "curvedConnector3",
        }
    }
}

fn positioned_transform(left: Emu, top: Emu, width: Emu, height: Emu) -> CT_Transform2D {
    let mut transform = CT_Transform2D::default();
    transform.offset = Some(CT_Point2D { x: left, y: top });
    transform.extent = Some(CT_PositiveSize2D {
        cx: width,
        cy: height,
    });
    transform
}

fn connector_transform(
    begin_x: Emu,
    begin_y: Emu,
    end_x: Emu,
    end_y: Emu,
) -> Result<CT_Transform2D> {
    let width =
        i64::try_from(begin_x.0.abs_diff(end_x.0)).map_err(|_| Error::InvalidShapeMutation {
            operation: "add connector",
            message: "horizontal endpoint span exceeds the EMU range".to_owned(),
        })?;
    let height =
        i64::try_from(begin_y.0.abs_diff(end_y.0)).map_err(|_| Error::InvalidShapeMutation {
            operation: "add connector",
            message: "vertical endpoint span exceeds the EMU range".to_owned(),
        })?;
    let mut transform = CT_Transform2D::default();
    transform.offset = Some(CT_Point2D {
        x: Emu(begin_x.0.min(end_x.0)),
        y: Emu(begin_y.0.min(end_y.0)),
    });
    transform.extent = Some(CT_PositiveSize2D {
        cx: Emu(width),
        cy: Emu(height),
    });
    transform.flip_horizontal = begin_x.0 > end_x.0;
    transform.flip_vertical = begin_y.0 > end_y.0;
    Ok(transform)
}

fn invalid_shape_construction(operation: &'static str, error: OxmlError) -> Error {
    Error::InvalidShapeMutation {
        operation,
        message: error.to_string(),
    }
}

fn invalid_table_mutation(operation: &'static str, message: String) -> Error {
    Error::InvalidTableMutation { operation, message }
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

/// A mutably borrowed shape-tree child.
pub struct ShapeMut<'a> {
    child: &'a mut ShapeTreeChild,
}

fn shape_mut(child: &mut ShapeTreeChild) -> ShapeMut<'_> {
    ShapeMut { child }
}

impl ShapeMut<'_> {
    /// Returns the child's normalized structural kind.
    pub fn kind(&self) -> ShapeKind {
        shape_kind(self.child)
    }

    /// Returns one immediate group child for in-place mutation.
    ///
    /// `mc:AlternateContent` fallback children remain a read-only projection.
    pub fn child_mut(&mut self, index: usize) -> Option<ShapeMut<'_>> {
        match self.child {
            ShapeTreeChild::GroupShape(group) => group.children.get_mut(index).map(shape_mut),
            _ => None,
        }
    }

    /// Changes the shape offset in EMU.
    pub fn set_position(&mut self, left: Emu, top: Emu) -> Result<()> {
        self.transform_mut("set position")?.offset = Some(CT_Point2D { x: left, y: top });
        Ok(())
    }

    /// Changes the shape extent in EMU.
    pub fn set_size(&mut self, width: Emu, height: Emu) -> Result<()> {
        self.transform_mut("set size")?.extent = Some(CT_PositiveSize2D {
            cx: width,
            cy: height,
        });
        Ok(())
    }

    /// Changes the clockwise DrawingML rotation.
    pub fn set_rotation(&mut self, rotation: Angle) -> Result<()> {
        self.transform_mut("set rotation")?.rotation = rotation;
        Ok(())
    }

    /// Changes the producer-facing non-visual name.
    pub fn set_name(&mut self, name: &str) -> Result<()> {
        let result = match self.child {
            ShapeTreeChild::Shape(shape) => shape.set_name(name),
            ShapeTreeChild::Picture(picture) => picture.set_name(name),
            ShapeTreeChild::GraphicFrame(frame) => frame.set_name(name),
            ShapeTreeChild::GroupShape(group) => group.set_name(name),
            ShapeTreeChild::Connector(connector) => connector.set_name(name),
            ShapeTreeChild::AlternateContent(_) => {
                return Err(self.unsupported("set name"));
            }
        };
        result.map_err(|error| Error::InvalidShapeMutation {
            operation: "set name",
            message: error.to_string(),
        })
    }

    /// Replaces the direct fill on a shape with typed shape properties.
    pub fn set_fill(&mut self, fill: Fill) -> Result<()> {
        self.shape_properties_mut("set fill")?.fill = Some(fill);
        Ok(())
    }

    /// Replaces the direct line on a shape with typed shape properties.
    pub fn set_line(&mut self, line: CT_LineProperties) -> Result<()> {
        self.shape_properties_mut("set line")?.line = Some(line);
        Ok(())
    }

    /// Inserts or replaces one finite preset-geometry adjustment.
    pub fn set_adjust_value(&mut self, name: &str, value: f64) -> Result<()> {
        if !value.is_finite() {
            return Err(Error::NonFiniteAdjustmentValue {
                name: name.to_owned(),
            });
        }
        let shape_kind = shape_kind(self.child);
        let properties = self.shape_properties_mut("set adjustment")?;
        let geometry =
            properties
                .preset_geometry
                .as_mut()
                .ok_or(Error::UnsupportedShapeMutation {
                    operation: "set adjustment",
                    shape_kind,
                })?;
        geometry
            .set_adjust_value(name, value)
            .map_err(|error| Error::InvalidShapeMutation {
                operation: "set adjustment",
                message: error.to_string(),
            })
    }

    /// Replaces ordinary shape text without changing placeholder identity.
    pub fn set_text(&mut self, text: &str) -> Result<()> {
        let shape_kind = shape_kind(self.child);
        let ShapeTreeChild::Shape(shape) = self.child else {
            return Err(Error::UnsupportedShapeMutation {
                operation: "set text",
                shape_kind,
            });
        };
        shape
            .text_body
            .get_or_insert_with(CT_TextBody::new)
            .set_text(text);
        Ok(())
    }

    /// Returns the ordinary shape text body for in-place mutation.
    pub fn text_frame(&mut self) -> Option<TextFrame<'_>> {
        match self.child {
            ShapeTreeChild::Shape(shape) => shape.text_body.as_mut().map(|body| TextFrame { body }),
            _ => None,
        }
    }

    /// Returns a behavior-bearing mutable handle for a table graphic frame.
    pub fn table_mut(&mut self) -> Option<TableMut<'_>> {
        let ShapeTreeChild::GraphicFrame(frame) = self.child else {
            return None;
        };
        let GraphicDataPayload::Table(table) = &mut frame.graphic_data.payload else {
            return None;
        };
        Some(TableMut {
            table,
            transform: &mut frame.transform,
        })
    }

    fn transform_mut(&mut self, operation: &'static str) -> Result<&mut CT_Transform2D> {
        match self.child {
            ShapeTreeChild::Shape(shape) => Ok(shape
                .shape_properties
                .transform
                .get_or_insert_with(CT_Transform2D::default)),
            ShapeTreeChild::Picture(picture) => Ok(picture
                .shape_properties
                .transform
                .get_or_insert_with(CT_Transform2D::default)),
            ShapeTreeChild::GraphicFrame(frame) => Ok(&mut frame.transform),
            ShapeTreeChild::GroupShape(group) => Ok(group.group_transform_mut()),
            ShapeTreeChild::Connector(connector) => Ok(connector
                .shape_properties
                .transform
                .get_or_insert_with(CT_Transform2D::default)),
            ShapeTreeChild::AlternateContent(_) => Err(self.unsupported(operation)),
        }
    }

    fn shape_properties_mut(&mut self, operation: &'static str) -> Result<&mut CT_ShapeProperties> {
        match self.child {
            ShapeTreeChild::Shape(shape) => Ok(&mut shape.shape_properties),
            ShapeTreeChild::Picture(picture) => Ok(&mut picture.shape_properties),
            ShapeTreeChild::Connector(connector) => Ok(&mut connector.shape_properties),
            _ => Err(self.unsupported(operation)),
        }
    }

    fn unsupported(&self, operation: &'static str) -> Error {
        Error::UnsupportedShapeMutation {
            operation,
            shape_kind: shape_kind(self.child),
        }
    }
}

/// A mutably borrowed ordinary-shape text body.
pub struct TextFrame<'a> {
    body: &'a mut CT_TextBody,
}

/// A borrowed DrawingML table.
#[derive(Clone, Copy)]
pub struct TableRef<'a> {
    table: &'a CT_Table,
}

impl TableRef<'_> {
    /// Returns the number of explicit table rows.
    pub fn row_count(&self) -> usize {
        self.table.rows.len()
    }

    /// Returns the number of grid columns.
    pub fn column_count(&self) -> usize {
        self.table.grid.columns.len()
    }

    /// Returns one explicit grid cell by zero-based row and column.
    pub fn cell(&self, row: usize, column: usize) -> Option<TableCellRef<'_>> {
        table_cell(self.table, row, column).map(|cell| TableCellRef { cell })
    }

    /// Returns one grid-column width in EMU.
    pub fn column_width(&self, column: usize) -> Option<Emu> {
        self.table.grid.columns.get(column).copied()
    }

    /// Returns whether first-row table styling is enabled.
    pub fn first_row(&self) -> bool {
        self.table
            .properties
            .as_ref()
            .is_some_and(|properties| properties.first_row)
    }

    /// Returns whether last-row table styling is enabled.
    pub fn last_row(&self) -> bool {
        self.table
            .properties
            .as_ref()
            .is_some_and(|properties| properties.last_row)
    }

    /// Returns whether first-column table styling is enabled.
    pub fn first_column(&self) -> bool {
        self.table
            .properties
            .as_ref()
            .is_some_and(|properties| properties.first_column)
    }

    /// Returns whether last-column table styling is enabled.
    pub fn last_column(&self) -> bool {
        self.table
            .properties
            .as_ref()
            .is_some_and(|properties| properties.last_column)
    }

    /// Returns whether horizontal row banding is enabled.
    pub fn horizontal_banding(&self) -> bool {
        self.table
            .properties
            .as_ref()
            .is_some_and(|properties| properties.band_rows)
    }

    /// Returns whether vertical column banding is enabled.
    pub fn vertical_banding(&self) -> bool {
        self.table
            .properties
            .as_ref()
            .is_some_and(|properties| properties.band_columns)
    }
}

/// A behavior-bearing mutable DrawingML table and its containing frame extent.
pub struct TableMut<'a> {
    table: &'a mut CT_Table,
    transform: &'a mut CT_Transform2D,
}

impl TableMut<'_> {
    /// Returns the number of explicit table rows.
    pub fn row_count(&self) -> usize {
        self.table.rows.len()
    }

    /// Returns the number of grid columns.
    pub fn column_count(&self) -> usize {
        self.table.grid.columns.len()
    }

    /// Returns one explicit grid cell by zero-based row and column.
    pub fn cell(&self, row: usize, column: usize) -> Option<TableCellRef<'_>> {
        table_cell(self.table, row, column).map(|cell| TableCellRef { cell })
    }

    /// Returns one explicit grid cell for in-place mutation.
    pub fn cell_mut(&mut self, row: usize, column: usize) -> Option<TableCellMut<'_>> {
        table_cell(self.table, row, column)?;
        Some(TableCellMut {
            table: self.table,
            row,
            column,
        })
    }

    /// Returns one grid-column width in EMU.
    pub fn column_width(&self, column: usize) -> Option<Emu> {
        self.table.grid.columns.get(column).copied()
    }

    /// Changes one grid width and synchronizes the containing frame width.
    pub fn set_column_width(&mut self, column: usize, width: Emu) -> Result<()> {
        if width.0 <= 0 {
            return Err(invalid_table_mutation(
                "set column width",
                "column width must be positive".to_owned(),
            ));
        }
        if column >= self.table.grid.columns.len() {
            return Err(invalid_table_mutation(
                "set column width",
                format!("column index {column} is out of range"),
            ));
        }
        let total_width = self
            .table
            .grid
            .columns
            .iter()
            .enumerate()
            .try_fold(0i64, |total, (index, current)| {
                total.checked_add(if index == column { width.0 } else { current.0 })
            })
            .ok_or_else(|| {
                invalid_table_mutation(
                    "set column width",
                    "table width exceeds the EMU range".to_owned(),
                )
            })?;
        let total_height = self
            .table
            .rows
            .iter()
            .try_fold(0i64, |total, row| total.checked_add(row.height.0))
            .ok_or_else(|| {
                invalid_table_mutation(
                    "set column width",
                    "table height exceeds the EMU range".to_owned(),
                )
            })?;
        if total_width <= 0 || total_height <= 0 {
            return Err(invalid_table_mutation(
                "set column width",
                "table width and height must remain positive".to_owned(),
            ));
        }

        let mut staged = self.table.clone();
        staged.grid.columns[column] = width;
        staged
            .to_xml()
            .map_err(|error| invalid_table_mutation("set column width", error.to_string()))?;
        self.table.grid.columns[column] = width;
        let height = self
            .transform
            .extent
            .map_or(Emu(total_height), |extent| extent.cy);
        self.transform.extent = Some(CT_PositiveSize2D {
            cx: Emu(total_width),
            cy: height,
        });
        Ok(())
    }

    /// Returns whether first-row table styling is enabled.
    pub fn first_row(&self) -> bool {
        self.table
            .properties
            .as_ref()
            .is_some_and(|properties| properties.first_row)
    }

    /// Enables or disables first-row table styling.
    pub fn set_first_row(&mut self, value: bool) {
        table_properties_mut(self.table).first_row = value;
    }

    /// Returns whether last-row table styling is enabled.
    pub fn last_row(&self) -> bool {
        self.table
            .properties
            .as_ref()
            .is_some_and(|properties| properties.last_row)
    }

    /// Enables or disables last-row table styling.
    pub fn set_last_row(&mut self, value: bool) {
        table_properties_mut(self.table).last_row = value;
    }

    /// Returns whether first-column table styling is enabled.
    pub fn first_column(&self) -> bool {
        self.table
            .properties
            .as_ref()
            .is_some_and(|properties| properties.first_column)
    }

    /// Enables or disables first-column table styling.
    pub fn set_first_column(&mut self, value: bool) {
        table_properties_mut(self.table).first_column = value;
    }

    /// Returns whether last-column table styling is enabled.
    pub fn last_column(&self) -> bool {
        self.table
            .properties
            .as_ref()
            .is_some_and(|properties| properties.last_column)
    }

    /// Enables or disables last-column table styling.
    pub fn set_last_column(&mut self, value: bool) {
        table_properties_mut(self.table).last_column = value;
    }

    /// Returns whether horizontal row banding is enabled.
    pub fn horizontal_banding(&self) -> bool {
        self.table
            .properties
            .as_ref()
            .is_some_and(|properties| properties.band_rows)
    }

    /// Enables or disables horizontal row banding.
    pub fn set_horizontal_banding(&mut self, value: bool) {
        table_properties_mut(self.table).band_rows = value;
    }

    /// Returns whether vertical column banding is enabled.
    pub fn vertical_banding(&self) -> bool {
        self.table
            .properties
            .as_ref()
            .is_some_and(|properties| properties.band_columns)
    }

    /// Enables or disables vertical column banding.
    pub fn set_vertical_banding(&mut self, value: bool) {
        table_properties_mut(self.table).band_columns = value;
    }
}

/// A borrowed explicit table-grid cell.
#[derive(Clone, Copy)]
pub struct TableCellRef<'a> {
    cell: &'a CT_TableCell,
}

impl TableCellRef<'_> {
    /// Returns the cell's visible plain text.
    pub fn text(&self) -> String {
        self.cell
            .text_body
            .as_ref()
            .map_or_else(String::new, CT_TextBody::plain_text)
    }

    /// Returns whether this cell is the top-left origin of a merge.
    pub fn is_merge_origin(&self) -> bool {
        !self.is_spanned() && (self.cell.row_span > 1 || self.cell.grid_span > 1)
    }

    /// Returns whether this cell continues a merge from another grid cell.
    pub fn is_spanned(&self) -> bool {
        self.cell.horizontal_merge || self.cell.vertical_merge
    }

    /// Returns the origin's vertical span, or one for an unmerged cell.
    pub fn span_height(&self) -> u32 {
        self.cell.row_span
    }

    /// Returns the origin's horizontal span, or one for an unmerged cell.
    pub fn span_width(&self) -> u32 {
        self.cell.grid_span
    }

    /// Returns the direct cell fill, when present.
    pub fn fill(&self) -> Option<&Fill> {
        self.cell
            .properties
            .as_ref()
            .and_then(|properties| properties.fill.as_ref())
    }

    /// Returns the optional left, right, top, and bottom cell margins.
    pub fn margins(&self) -> (Option<Emu>, Option<Emu>, Option<Emu>, Option<Emu>) {
        self.cell
            .properties
            .as_ref()
            .map_or((None, None, None, None), |properties| {
                (
                    properties.margin_left,
                    properties.margin_right,
                    properties.margin_top,
                    properties.margin_bottom,
                )
            })
    }
}

/// A behavior-bearing mutable table-grid cell addressed within its table.
pub struct TableCellMut<'a> {
    table: &'a mut CT_Table,
    row: usize,
    column: usize,
}

impl TableCellMut<'_> {
    /// Returns the cell's visible plain text.
    pub fn text(&self) -> String {
        self.cell_ref().text()
    }

    /// Replaces the cell text with one paragraph and one regular run.
    pub fn set_text(&mut self, text: &str) {
        self.cell_mut()
            .text_body
            .get_or_insert_with(CT_TextBody::new)
            .set_text(text);
    }

    /// Returns the cell text body for in-place mutation.
    pub fn text_frame(&mut self) -> TextFrame<'_> {
        let body = self
            .cell_mut()
            .text_body
            .get_or_insert_with(CT_TextBody::new);
        TextFrame { body }
    }

    /// Replaces or clears the direct cell fill.
    pub fn set_fill(&mut self, fill: Option<Fill>) {
        self.properties_mut().fill = fill;
    }

    /// Returns the direct cell fill, when present.
    pub fn fill(&self) -> Option<&Fill> {
        self.cell_ref().cell.properties.as_ref()?.fill.as_ref()
    }

    /// Replaces all four optional cell margins.
    pub fn set_margins(
        &mut self,
        left: Option<Emu>,
        right: Option<Emu>,
        top: Option<Emu>,
        bottom: Option<Emu>,
    ) {
        let properties = self.properties_mut();
        properties.margin_left = left;
        properties.margin_right = right;
        properties.margin_top = top;
        properties.margin_bottom = bottom;
    }

    /// Returns the optional left, right, top, and bottom cell margins.
    pub fn margins(&self) -> (Option<Emu>, Option<Emu>, Option<Emu>, Option<Emu>) {
        let Some(properties) = self.cell_ref().cell.properties.as_ref() else {
            return (None, None, None, None);
        };
        (
            properties.margin_left,
            properties.margin_right,
            properties.margin_top,
            properties.margin_bottom,
        )
    }

    /// Returns whether this cell is the top-left origin of a merge.
    pub fn is_merge_origin(&self) -> bool {
        self.cell_ref().is_merge_origin()
    }

    /// Returns whether this cell continues a merge from another grid cell.
    pub fn is_spanned(&self) -> bool {
        self.cell_ref().is_spanned()
    }

    /// Returns the origin's vertical span, or one for an unmerged cell.
    pub fn span_height(&self) -> u32 {
        self.cell_ref().span_height()
    }

    /// Returns the origin's horizontal span, or one for an unmerged cell.
    pub fn span_width(&self) -> u32 {
        self.cell_ref().span_width()
    }

    /// Merges the rectangle between this cell and the other grid coordinate.
    pub fn merge_to(&mut self, row: usize, column: usize) -> Result<()> {
        let mut staged = self.table.clone();
        merge_cells(&mut staged, self.row, self.column, row, column)
            .map_err(|message| invalid_table_mutation("merge cells", message))?;
        staged
            .to_xml()
            .map_err(|error| invalid_table_mutation("merge cells", error.to_string()))?;
        *self.table = staged;
        Ok(())
    }

    /// Splits this merge origin back into its explicit grid cells.
    pub fn split(&mut self) -> Result<()> {
        let mut staged = self.table.clone();
        split_cell(&mut staged, self.row, self.column)
            .map_err(|message| invalid_table_mutation("split cell", message))?;
        staged
            .to_xml()
            .map_err(|error| invalid_table_mutation("split cell", error.to_string()))?;
        *self.table = staged;
        Ok(())
    }

    fn cell_ref(&self) -> TableCellRef<'_> {
        TableCellRef {
            cell: table_cell(self.table, self.row, self.column)
                .expect("TableCellMut coordinates were validated at construction"),
        }
    }

    fn cell_mut(&mut self) -> &mut CT_TableCell {
        &mut self.table.rows[self.row].cells[self.column]
    }

    fn properties_mut(&mut self) -> &mut CT_TableCellProperties {
        self.cell_mut()
            .properties
            .get_or_insert_with(CT_TableCellProperties::default)
    }
}

fn table_cell(table: &CT_Table, row: usize, column: usize) -> Option<&CT_TableCell> {
    table.rows.get(row)?.cells.get(column)
}

fn table_properties_mut(table: &mut CT_Table) -> &mut CT_TableProperties {
    table
        .properties
        .get_or_insert_with(CT_TableProperties::default)
}

fn rectangular_dimensions(table: &CT_Table) -> std::result::Result<(usize, usize), String> {
    let rows = table.rows.len();
    let columns = table.grid.columns.len();
    if rows == 0 || columns == 0 || table.rows.iter().any(|row| row.cells.len() != columns) {
        return Err("table does not have a rectangular explicit cell grid".to_owned());
    }
    Ok((rows, columns))
}

fn merge_cells(
    table: &mut CT_Table,
    first_row: usize,
    first_column: usize,
    second_row: usize,
    second_column: usize,
) -> std::result::Result<(), String> {
    let (rows, columns) = rectangular_dimensions(table)?;
    if first_row >= rows
        || second_row >= rows
        || first_column >= columns
        || second_column >= columns
    {
        return Err("merge coordinate is out of range".to_owned());
    }
    let top = first_row.min(second_row);
    let bottom = first_row.max(second_row);
    let left = first_column.min(second_column);
    let right = first_column.max(second_column);
    if top == bottom && left == right {
        return Err("merge requires at least two cells".to_owned());
    }
    if table.rows[top..=bottom].iter().any(|row| {
        row.cells[left..=right].iter().any(|cell| {
            cell.row_span != 1
                || cell.grid_span != 1
                || cell.horizontal_merge
                || cell.vertical_merge
        })
    }) {
        return Err("merge range contains one or more merged cells".to_owned());
    }
    let row_span = u32::try_from(bottom - top + 1)
        .map_err(|_| "merge row span exceeds the DrawingML range".to_owned())?;
    let grid_span = u32::try_from(right - left + 1)
        .map_err(|_| "merge column span exceeds the DrawingML range".to_owned())?;

    let mut moved_bodies = Vec::new();
    for row in top..=bottom {
        for column in left..=right {
            if row == top && column == left {
                continue;
            }
            let body = table.rows[row].cells[column]
                .text_body
                .take()
                .unwrap_or_default();
            moved_bodies.push((row, column, body));
        }
    }
    for (row, column, mut body) in moved_bodies {
        let origin_body = table.rows[top].cells[left]
            .text_body
            .get_or_insert_with(CT_TextBody::new);
        body.move_content_to(origin_body);
        table.rows[row].cells[column].text_body = Some(body);
    }

    for row in top..=bottom {
        for column in left..=right {
            let cell = &mut table.rows[row].cells[column];
            cell.row_span = if row == top { row_span } else { 1 };
            cell.grid_span = if column == left { grid_span } else { 1 };
            cell.horizontal_merge = column != left;
            cell.vertical_merge = row != top;
        }
    }
    Ok(())
}

fn split_cell(table: &mut CT_Table, row: usize, column: usize) -> std::result::Result<(), String> {
    let (rows, columns) = rectangular_dimensions(table)?;
    let origin = table_cell(table, row, column)
        .ok_or_else(|| "split coordinate is out of range".to_owned())?;
    if origin.horizontal_merge
        || origin.vertical_merge
        || (origin.row_span == 1 && origin.grid_span == 1)
    {
        return Err("only a merge-origin cell can be split".to_owned());
    }
    let bottom = row
        .checked_add(origin.row_span as usize - 1)
        .ok_or_else(|| "merge row span exceeds the table grid".to_owned())?;
    let right = column
        .checked_add(origin.grid_span as usize - 1)
        .ok_or_else(|| "merge column span exceeds the table grid".to_owned())?;
    if bottom >= rows || right >= columns {
        return Err("merge origin span exceeds the table grid".to_owned());
    }
    let row_span = origin.row_span;
    let grid_span = origin.grid_span;
    for current_row in row..=bottom {
        for current_column in column..=right {
            let cell = &table.rows[current_row].cells[current_column];
            let expected = (
                if current_row == row { row_span } else { 1 },
                if current_column == column {
                    grid_span
                } else {
                    1
                },
                current_column != column,
                current_row != row,
            );
            if (
                cell.row_span,
                cell.grid_span,
                cell.horizontal_merge,
                cell.vertical_merge,
            ) != expected
            {
                return Err("merge origin does not have a valid continuation rectangle".to_owned());
            }
        }
    }
    for current_row in row..=bottom {
        for current_column in column..=right {
            let cell = &mut table.rows[current_row].cells[current_column];
            cell.row_span = 1;
            cell.grid_span = 1;
            cell.horizontal_merge = false;
            cell.vertical_merge = false;
        }
    }
    Ok(())
}

impl TextFrame<'_> {
    /// Returns plain text in paragraph and ordered text-choice order.
    pub fn text(&self) -> String {
        self.body.plain_text()
    }

    /// Replaces the frame content with one paragraph and one regular run.
    pub fn set_text(&mut self, text: &str) {
        self.body.set_text(text);
    }

    /// Returns the number of paragraphs in the text body.
    pub fn paragraph_count(&self) -> usize {
        self.body.paragraph_count()
    }

    /// Returns one paragraph for in-place mutation.
    pub fn paragraph_mut(&mut self, index: usize) -> Option<TextParagraphMut<'_>> {
        self.body
            .paragraph_mut(index)
            .map(|paragraph| TextParagraphMut { paragraph })
    }

    /// Appends one empty paragraph.
    pub fn add_paragraph(&mut self) -> TextParagraphMut<'_> {
        TextParagraphMut {
            paragraph: self.body.add_paragraph(),
        }
    }
}

/// A mutably borrowed DrawingML text paragraph.
pub struct TextParagraphMut<'a> {
    paragraph: &'a mut CT_TextParagraph,
}

impl TextParagraphMut<'_> {
    /// Replaces fields, breaks, and runs with one regular run.
    pub fn set_text(&mut self, text: &str) {
        self.paragraph.set_text(text);
    }

    /// Appends a regular run after the existing ordered text choices.
    pub fn add_run(&mut self, text: &str) -> TextRunMut<'_> {
        TextRunMut {
            run: self.paragraph.add_run(text),
        }
    }

    /// Replaces the paragraph's direct typed properties.
    pub fn set_properties(&mut self, properties: CT_TextParagraphProperties) {
        *self.paragraph.properties_mut() = properties;
    }

    /// Sets or clears the direct paragraph bullet.
    pub fn set_bullet(&mut self, bullet: Option<TextBullet>) {
        if let Some(properties) = self.paragraph.properties.as_mut() {
            properties.bullet = bullet;
        } else if bullet.is_some() {
            self.paragraph.properties_mut().bullet = bullet;
        }
    }
}

/// A mutably borrowed regular DrawingML text run.
pub struct TextRunMut<'a> {
    run: &'a mut CT_RegularTextRun,
}

impl TextRunMut<'_> {
    /// Replaces text while retaining the run's typed and unmodelled state.
    pub fn set_text(&mut self, text: &str) {
        self.run.set_text(text);
    }

    /// Replaces the run's direct typed character properties.
    pub fn set_properties(&mut self, properties: CT_TextCharacterProperties) {
        self.run.properties = Some(properties);
    }

    /// Sets or clears the direct Latin typeface.
    pub fn set_font(&mut self, font: Option<TextFont>) {
        if let Some(properties) = self.run.properties.as_mut() {
            properties.latin = font;
        } else if font.is_some() {
            let mut properties = CT_TextCharacterProperties::default();
            properties.latin = font;
            self.run.properties = Some(properties);
        }
    }
}

fn shape_kind(child: &ShapeTreeChild) -> ShapeKind {
    match child {
        ShapeTreeChild::Shape(_) => ShapeKind::Shape,
        ShapeTreeChild::Picture(_) => ShapeKind::Picture,
        ShapeTreeChild::GraphicFrame(_) => ShapeKind::GraphicFrame,
        ShapeTreeChild::GroupShape(_) => ShapeKind::Group,
        ShapeTreeChild::Connector(_) => ShapeKind::Connector,
        ShapeTreeChild::AlternateContent(_) => ShapeKind::AlternateContent,
    }
}

impl<'a> ShapeRef<'a> {
    /// Returns the child's normalized structural kind.
    pub fn kind(&self) -> ShapeKind {
        shape_kind(self.child)
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

    /// Returns the typed table carried by this graphic frame, when present.
    pub fn table(&self) -> Option<TableRef<'a>> {
        let ShapeTreeChild::GraphicFrame(frame) = self.child else {
            return None;
        };
        let GraphicDataPayload::Table(table) = &frame.graphic_data.payload else {
            return None;
        };
        Some(TableRef { table })
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

    #[test]
    fn two_dimensional_merge_encodes_origins_and_continuations() {
        let mut presentation = Presentation::new().expect("open bundled template");
        presentation.add_slide(0).expect("add slide");
        let mut slide = presentation.slide_mut(0).expect("borrow slide");
        let mut shape = slide
            .add_table(3, 3, Emu(0), Emu(0), Emu(300), Emu(300))
            .expect("add table");
        let mut table = shape.table_mut().expect("table handle");
        table
            .cell_mut(0, 0)
            .expect("merge origin")
            .merge_to(2, 1)
            .expect("merge cells");

        let expected = [
            [(3, 2, false, false), (3, 1, true, false)],
            [(1, 2, false, true), (1, 1, true, true)],
            [(1, 2, false, true), (1, 1, true, true)],
        ];
        for (row_index, row) in expected.into_iter().enumerate() {
            for (column_index, expected_cell) in row.into_iter().enumerate() {
                let cell = &table.table.rows[row_index].cells[column_index];
                assert_eq!(
                    (
                        cell.row_span,
                        cell.grid_span,
                        cell.horizontal_merge,
                        cell.vertical_merge,
                    ),
                    expected_cell,
                    "cell {row_index},{column_index}"
                );
            }
        }
        for row in 0..3 {
            let cell = &table.table.rows[row].cells[2];
            assert_eq!(
                (
                    cell.row_span,
                    cell.grid_span,
                    cell.horizontal_merge,
                    cell.vertical_merge,
                ),
                (1, 1, false, false),
                "outside cell {row},2"
            );
        }
    }

    #[test]
    fn merge_moves_formatted_content_to_origin_in_row_major_order() {
        let mut presentation = Presentation::new().expect("open bundled template");
        presentation.add_slide(0).expect("add slide");
        let mut slide = presentation.slide_mut(0).expect("borrow slide");
        let mut shape = slide
            .add_table(2, 2, Emu(0), Emu(0), Emu(200), Emu(200))
            .expect("add table");
        let mut table = shape.table_mut().expect("table handle");
        for (index, text) in ["one", "two", "three", "four"].into_iter().enumerate() {
            table.cell_mut(index / 2, index % 2).unwrap().set_text(text);
        }
        {
            let mut source = table.cell_mut(0, 1).unwrap();
            let mut frame = source.text_frame();
            let mut paragraph = frame.paragraph_mut(0).unwrap();
            let mut properties = CT_TextParagraphProperties::default();
            properties.level = Some(2);
            paragraph.set_properties(properties);
            let mut run = paragraph.add_run(" formatted");
            let mut properties = CT_TextCharacterProperties::default();
            properties.bold = Some(true);
            run.set_properties(properties);
        }
        table.cell_mut(1, 1).unwrap().merge_to(0, 0).unwrap();
        assert_eq!(
            table.cell(0, 0).unwrap().text(),
            "one\ntwo formatted\nthree\nfour"
        );
        let origin = &table.table.rows[0].cells[0];
        let paragraphs = origin.text_body.as_ref().unwrap().paragraphs();
        assert_eq!(paragraphs[1].properties.as_ref().unwrap().level, Some(2));
        let oxml_drawing::text::TextRun::Run(run) = &paragraphs[1].runs[1] else {
            panic!("expected regular run");
        };
        assert_eq!(run.properties.as_ref().unwrap().bold, Some(true));
        table.cell_mut(0, 0).unwrap().split().unwrap();
        assert_eq!(
            table.cell(0, 0).unwrap().text(),
            "one\ntwo formatted\nthree\nfour"
        );
        assert_eq!(table.cell(0, 1).unwrap().text(), "");
    }

    #[test]
    fn merge_materializes_minimal_text_bodies_for_absent_sources() {
        let mut presentation = Presentation::new().expect("open bundled template");
        presentation.add_slide(0).expect("add slide");
        let mut slide = presentation.slide_mut(0).expect("borrow slide");
        let mut shape = slide
            .add_table(2, 2, Emu(0), Emu(0), Emu(200), Emu(200))
            .expect("add table");
        let mut table = shape.table_mut().expect("table handle");
        for (row, column) in [(0, 1), (1, 0), (1, 1)] {
            table.table.rows[row].cells[column].text_body = None;
        }

        table.cell_mut(0, 0).unwrap().merge_to(1, 1).unwrap();

        for (row, column) in [(0, 1), (1, 0), (1, 1)] {
            let body = table.table.rows[row].cells[column]
                .text_body
                .as_ref()
                .expect("merge source receives a text body");
            assert_eq!(body.paragraph_count(), 1);
            assert_eq!(body.plain_text(), "");
        }
    }

    #[test]
    fn connector_constructor_normalizes_every_direction() {
        assert_eq!(ConnectorType::Straight.preset_name(), "line");
        assert_eq!(ConnectorType::Elbow.preset_name(), "bentConnector3");
        assert_eq!(ConnectorType::Curve.preset_name(), "curvedConnector3");
        let cases = [
            ((10, 20), (30, 40), (10, 20, 20, 20, false, false)),
            ((30, 20), (10, 40), (10, 20, 20, 20, true, false)),
            ((10, 40), (30, 20), (10, 20, 20, 20, false, true)),
            ((30, 40), (10, 20), (10, 20, 20, 20, true, true)),
            ((10, 20), (10, 40), (10, 20, 0, 20, false, false)),
            ((30, 20), (10, 20), (10, 20, 20, 0, true, false)),
        ];
        for (begin, end, expected) in cases {
            let transform =
                connector_transform(Emu(begin.0), Emu(begin.1), Emu(end.0), Emu(end.1)).unwrap();
            let offset = transform.offset.unwrap();
            let extent = transform.extent.unwrap();
            assert_eq!(
                (
                    offset.x.0,
                    offset.y.0,
                    extent.cx.0,
                    extent.cy.0,
                    transform.flip_horizontal,
                    transform.flip_vertical,
                ),
                expected
            );
        }

        assert!(connector_transform(Emu(i64::MIN), Emu(0), Emu(i64::MAX), Emu(0)).is_err());
    }

    #[test]
    fn every_validation_issue_variant_detects_its_corrupted_deck() {
        let mut duplicate_shape_package = package_after_save(presentation_with_slide());
        let slide_part = "/ppt/slides/slide1.xml";
        let slide_xml = String::from_utf8(
            duplicate_shape_package
                .get_part(slide_part)
                .unwrap()
                .to_vec(),
        )
        .unwrap();
        let duplicate_shape_xml = slide_xml.replacen(r#"id="1""#, r#"id="2""#, 1);
        assert_ne!(duplicate_shape_xml, slide_xml);
        duplicate_shape_package.set_part(slide_part, duplicate_shape_xml.into_bytes());
        let duplicate_shape = presentation_from_package(duplicate_shape_package);
        assert_has_issue(&duplicate_shape, |issue| {
            matches!(issue, ValidationIssue::DuplicateShapeId { slide: 0, id: 2 })
        });

        let mut out_of_range = presentation_with_slide();
        out_of_range.presentation.slide_ids[0].id = 1;
        assert_has_issue(&out_of_range, |issue| {
            matches!(
                issue,
                ValidationIssue::SlideIdOutOfRange { slide: 0, id: 1 }
            )
        });

        let mut duplicate_slide = presentation_with_slide();
        duplicate_slide.add_slide(0).unwrap();
        let duplicate_id = duplicate_slide.presentation.slide_ids[0].id;
        duplicate_slide.presentation.slide_ids[1].id = duplicate_id;
        assert_has_issue(
            &duplicate_slide,
            |issue| matches!(issue, ValidationIssue::DuplicateSlideId { id } if *id == duplicate_id),
        );

        let mut missing_content_type = Presentation::new().unwrap();
        missing_content_type
            .package
            .set_part("/ppt/untyped/payload.unknown", vec![1]);
        assert_has_issue(&missing_content_type, |issue| {
            matches!(
                issue,
                ValidationIssue::MissingContentTypeOverride { part }
                    if part == "/ppt/untyped/payload.unknown"
            )
        });

        let mut dangling_relationship_package = package_after_save(presentation_with_slide());
        let slide_part = "/ppt/slides/slide1.xml";
        let slide_xml = String::from_utf8(
            dangling_relationship_package
                .get_part(slide_part)
                .unwrap()
                .to_vec(),
        )
        .unwrap();
        let dangling_relationship_xml = slide_xml.replacen(
            "</p:sld>",
            r#"<x:probe xmlns:x="urn:validation" r:embed="rId999"/></p:sld>"#,
            1,
        );
        assert_ne!(dangling_relationship_xml, slide_xml);
        dangling_relationship_package.set_part(slide_part, dangling_relationship_xml.into_bytes());
        dangling_relationship_package.part_rels.remove(slide_part);
        let dangling_relationship = presentation_from_package(dangling_relationship_package);
        assert_has_issue(&dangling_relationship, |issue| {
            matches!(
                issue,
                ValidationIssue::DanglingRelationship { r_id, .. } if r_id == "rId999"
            )
        });

        let mut unreachable = presentation_with_slide();
        let slide_part = unreachable.slides[0].part_name.clone();
        unreachable
            .package
            .get_or_create_part_rels(&slide_part)
            .add(rel_types::IMAGE, "../media/missing.png");
        assert_has_issue(&unreachable, |issue| {
            matches!(
                issue,
                ValidationIssue::UnreachableRelationshipTarget { target, .. }
                    if target == "/ppt/media/missing.png"
            )
        });

        let mut empty_text_package = package_after_save(presentation_with_slide());
        let slide_part = "/ppt/slides/slide1.xml";
        let slide_xml =
            String::from_utf8(empty_text_package.get_part(slide_part).unwrap().to_vec()).unwrap();
        let empty_text_xml = slide_xml.replacen("<a:p/>", "", 1);
        assert_ne!(empty_text_xml, slide_xml);
        empty_text_package.set_part(slide_part, empty_text_xml.into_bytes());
        let empty_text = presentation_from_package(empty_text_package);
        assert_has_issue(&empty_text, |issue| {
            matches!(issue, ValidationIssue::EmptyTextBody { slide: 0, shape: 0 })
        });

        let mut duplicate_placeholder = presentation_with_slide();
        duplicate_placeholder.slides[0]
            .slide
            .common_slide_data
            .shape_tree
            .children
            .push(ShapeTreeChild::Shape(
                CT_Shape::new_placeholder(100, CT_Placeholder::new(Some(PhType::Body), Some(999)))
                    .unwrap(),
            ));
        duplicate_placeholder.slides[0]
            .slide
            .common_slide_data
            .shape_tree
            .children
            .push(ShapeTreeChild::Shape(
                CT_Shape::new_placeholder(
                    101,
                    CT_Placeholder::new(Some(PhType::Object), Some(999)),
                )
                .unwrap(),
            ));
        assert_has_issue(&duplicate_placeholder, |issue| {
            matches!(
                issue,
                ValidationIssue::DuplicatePlaceholderIdx { slide: 0, idx: 999 }
            )
        });

        let mut orphan_media = Presentation::new().unwrap();
        orphan_media
            .package
            .set_part("/ppt/media/orphan.png", b"orphan".to_vec());
        orphan_media
            .package
            .content_types
            .add_default("png", "image/png");
        assert_has_issue(&orphan_media, |issue| {
            matches!(
                issue,
                ValidationIssue::OrphanMedia { part } if part == "/ppt/media/orphan.png"
            )
        });

        let custom_show = presentation_with_custom_show("rId999");
        assert_has_issue(&custom_show, |issue| {
            matches!(
                issue,
                ValidationIssue::CustomShowReference { slide_id: 999 }
            )
        });

        let mut missing_layout = presentation_with_slide();
        let slide_part = missing_layout.slides[0].part_name.clone();
        missing_layout.package.part_rels.remove(&slide_part);
        assert_has_issue(&missing_layout, |issue| {
            matches!(issue, ValidationIssue::MissingLayoutRel { slide: 0 })
        });

        let mut missing_theme = Presentation::new().unwrap();
        let presentation_relationships = missing_theme
            .package
            .get_part_rels(&missing_theme.presentation_part)
            .unwrap();
        let master_relationship = presentation_relationships
            .get_by_id(&missing_theme.presentation.slide_master_ids[0].relationship_id)
            .unwrap();
        let master_part = OpcPackage::resolve_rel_target(
            &missing_theme.presentation_part,
            &master_relationship.target,
        );
        missing_theme
            .package
            .get_or_create_part_rels(&master_part)
            .items
            .retain(|relationship| relationship.rel_type != rel_types::THEME);
        assert_has_issue(&missing_theme, |issue| {
            matches!(issue, ValidationIssue::MissingThemeRel { master: 0 })
        });
    }

    #[test]
    fn validate_collects_all_issues_in_deterministic_order() {
        let mut presentation = presentation_with_slide();
        presentation.presentation.slide_ids[0].id = 1;
        presentation
            .package
            .set_part("/ppt/media/orphan.png", b"orphan".to_vec());
        presentation
            .package
            .content_types
            .add_default("png", "image/png");
        let first = presentation.validate();
        let second = presentation.validate();

        assert_eq!(first, second);
        assert!(matches!(
            first.first(),
            Some(ValidationIssue::SlideIdOutOfRange { .. })
        ));
        assert!(matches!(
            first.last(),
            Some(ValidationIssue::OrphanMedia { .. })
        ));
    }

    #[test]
    #[cfg(debug_assertions)]
    fn debug_save_boundaries_assert_on_invalid_presentations() {
        let mut presentation = presentation_with_slide();
        presentation.presentation.slide_ids[0].id = 1;
        let output = std::env::temp_dir().join(format!(
            "rpptx-f108-invalid-save-{}.pptx",
            std::process::id()
        ));

        assert!(std::panic::catch_unwind(|| presentation.to_bytes()).is_err());
        assert!(std::panic::catch_unwind(|| presentation.save(&output)).is_err());
        assert!(!output.exists());
    }

    #[test]
    fn validation_xml_scan_is_prefix_tolerant_and_non_mutating() {
        let presentation = presentation_with_slide();
        let relationship_id = presentation.presentation.slide_ids[0]
            .relationship_id
            .clone();
        let presentation = presentation_with_custom_show(&relationship_id);
        let before = presentation
            .package
            .get_part(&presentation.presentation_part)
            .unwrap()
            .to_vec();

        assert!(presentation.validate().is_empty());
        assert_eq!(
            presentation
                .package
                .get_part(&presentation.presentation_part)
                .unwrap(),
            before
        );
    }

    fn presentation_with_slide() -> Presentation {
        let mut presentation = Presentation::new().unwrap();
        presentation.add_slide(0).unwrap();
        presentation
    }

    fn package_after_save(presentation: Presentation) -> OpcPackage {
        OpcPackage::from_reader(Cursor::new(presentation.to_bytes().unwrap())).unwrap()
    }

    fn presentation_from_package(package: OpcPackage) -> Presentation {
        let mut bytes = Cursor::new(Vec::new());
        package.write_to(&mut bytes).unwrap();
        Presentation::from_bytes(bytes.get_ref()).unwrap()
    }

    fn presentation_with_custom_show(relationship_id: &str) -> Presentation {
        let mut package = package_after_save(presentation_with_slide());
        let presentation_part = "/ppt/presentation.xml";
        let xml = String::from_utf8(package.get_part(presentation_part).unwrap().to_vec()).unwrap();
        let custom_show = format!(
            r#"<q:custShowLst xmlns:q="http://schemas.openxmlformats.org/presentationml/2006/main" xmlns:rel="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><q:custShow name="validation"><q:sldLst><q:sld rel:id="{relationship_id}"/></q:sldLst></q:custShow></q:custShowLst>"#
        );
        package.set_part(
            presentation_part,
            xml.replacen(
                "</p:presentation>",
                &format!("{custom_show}</p:presentation>"),
                1,
            )
            .into_bytes(),
        );
        presentation_from_package(package)
    }

    fn assert_has_issue(presentation: &Presentation, predicate: impl Fn(&ValidationIssue) -> bool) {
        let issues = presentation.validate();
        assert!(issues.iter().any(predicate), "issues: {issues:?}");
    }

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
