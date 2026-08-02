//! Presentation rendering inputs and package assembly helpers.

use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::sync::Arc;

use oxml_drawing::theme::CT_OfficeStyleSheet;
use oxml_layout::{DocumentMetadata, FontFile, MediaId};
use rpptx_layout::ResolvedSlide;
use rpptx_oxml::notes_parts::CT_NotesSlide;
use rpptx_oxml::slide_parts::{CT_Slide, CT_SlideLayout, CT_SlideMaster};

/// The source part whose relationship map owns an identifier.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RelScope {
    Slide,
    Layout,
    Master,
}

impl fmt::Display for RelScope {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Slide => "slide",
            Self::Layout => "layout",
            Self::Master => "master",
        })
    }
}

/// A package relationship after its target has been resolved against its source part.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedRel {
    pub target: String,
    pub relationship_type: String,
}

/// Relationship maps kept separate by their source-part scope.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RelScopes {
    pub slide: HashMap<String, ResolvedRel>,
    pub layout: HashMap<String, ResolvedRel>,
    pub master: HashMap<String, ResolvedRel>,
}

impl RelScopes {
    /// Look up a relationship only in the explicitly selected source-part scope.
    pub fn get(
        &self,
        scope: RelScope,
        relationship_id: &str,
    ) -> Result<&ResolvedRel, RenderInputError> {
        let relationships = match scope {
            RelScope::Slide => &self.slide,
            RelScope::Layout => &self.layout,
            RelScope::Master => &self.master,
        };
        relationships
            .get(relationship_id)
            .ok_or_else(|| RenderInputError::MissingRelationship {
                scope,
                relationship_id: relationship_id.to_owned(),
            })
    }
}

/// Media bytes available to a renderer, with their package content type.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MediaData {
    pub bytes: Vec<u8>,
    pub content_type: String,
}

/// Package assembly failures that retain relationship source context.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RenderInputError {
    MissingRelationship {
        scope: RelScope,
        relationship_id: String,
    },
    MissingMediaTarget {
        scope: RelScope,
        relationship_id: String,
        target: String,
    },
}

impl fmt::Display for RenderInputError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingRelationship {
                scope,
                relationship_id,
            } => write!(formatter, "missing {scope} relationship {relationship_id}"),
            Self::MissingMediaTarget {
                scope,
                relationship_id,
                target,
            } => write!(
                formatter,
                "missing media target {target} for {scope} relationship {relationship_id}"
            ),
        }
    }
}

impl Error for RenderInputError {}

/// Raw package parts assembled before inheritance resolution.
#[derive(Clone, Debug)]
pub struct SlideBundle {
    pub slide: CT_Slide,
    pub layout: Arc<CT_SlideLayout>,
    pub master: Arc<CT_SlideMaster>,
    pub theme: Arc<CT_OfficeStyleSheet>,
    pub notes: Option<CT_NotesSlide>,
    pub hidden: bool,
    pub relationships: RelScopes,
}

/// Frozen, format-neutral input consumed by the rendering stage.
#[derive(Clone, Debug)]
pub struct RenderInput {
    pub slides: Vec<ResolvedSlide>,
    pub media: HashMap<MediaId, MediaData>,
    pub fonts: Vec<FontFile>,
    pub metadata: Option<DocumentMetadata>,
}

/// Resolve one scoped media relationship into the deck's content-addressed store.
pub fn resolve_media_relationship(
    relationships: &RelScopes,
    scope: RelScope,
    relationship_id: &str,
    package_media: &HashMap<String, MediaData>,
    deck_media: &mut HashMap<MediaId, MediaData>,
) -> Result<MediaId, RenderInputError> {
    let relationship = relationships.get(scope, relationship_id)?;
    let media = package_media.get(&relationship.target).ok_or_else(|| {
        RenderInputError::MissingMediaTarget {
            scope,
            relationship_id: relationship_id.to_owned(),
            target: relationship.target.clone(),
        }
    })?;
    let media_id = MediaId::from_bytes(&media.bytes);
    deck_media.entry(media_id).or_insert_with(|| media.clone());
    Ok(media_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    const IMAGE_RELATIONSHIP: &str =
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/image";

    fn media(bytes: &[u8]) -> MediaData {
        MediaData {
            bytes: bytes.to_vec(),
            content_type: "image/png".to_owned(),
        }
    }

    fn relationship(target: &str) -> ResolvedRel {
        ResolvedRel {
            target: target.to_owned(),
            relationship_type: IMAGE_RELATIONSHIP.to_owned(),
        }
    }

    #[test]
    fn same_relationship_id_resolves_independently_in_all_three_scopes() {
        let relationships = RelScopes {
            slide: HashMap::from([("rId2".to_owned(), relationship("slide.png"))]),
            layout: HashMap::from([("rId2".to_owned(), relationship("layout.png"))]),
            master: HashMap::from([("rId2".to_owned(), relationship("master.png"))]),
        };
        let package_media = HashMap::from([
            ("slide.png".to_owned(), media(b"slide image")),
            ("layout.png".to_owned(), media(b"layout image")),
            ("master.png".to_owned(), media(b"master image")),
        ]);
        let mut deck_media = HashMap::new();

        let slide = resolve_media_relationship(
            &relationships,
            RelScope::Slide,
            "rId2",
            &package_media,
            &mut deck_media,
        )
        .unwrap();
        let layout = resolve_media_relationship(
            &relationships,
            RelScope::Layout,
            "rId2",
            &package_media,
            &mut deck_media,
        )
        .unwrap();
        let master = resolve_media_relationship(
            &relationships,
            RelScope::Master,
            "rId2",
            &package_media,
            &mut deck_media,
        )
        .unwrap();

        assert_eq!(slide, MediaId::from_bytes(b"slide image"));
        assert_eq!(layout, MediaId::from_bytes(b"layout image"));
        assert_eq!(master, MediaId::from_bytes(b"master image"));
        assert_eq!(deck_media.len(), 3);
    }

    #[test]
    fn equal_media_bytes_deduplicate_to_one_media_entry() {
        let relationships = RelScopes {
            slide: HashMap::from([
                ("rId1".to_owned(), relationship("logo-a.png")),
                ("rId2".to_owned(), relationship("logo-b.png")),
            ]),
            ..RelScopes::default()
        };
        let package_media = HashMap::from([
            ("logo-a.png".to_owned(), media(b"shared logo")),
            ("logo-b.png".to_owned(), media(b"shared logo")),
        ]);
        let mut deck_media = HashMap::new();

        let first = resolve_media_relationship(
            &relationships,
            RelScope::Slide,
            "rId1",
            &package_media,
            &mut deck_media,
        )
        .unwrap();
        let second = resolve_media_relationship(
            &relationships,
            RelScope::Slide,
            "rId2",
            &package_media,
            &mut deck_media,
        )
        .unwrap();

        assert_eq!(first, second);
        assert_eq!(deck_media.len(), 1);
    }

    #[test]
    fn missing_relationship_reports_scope_and_id() {
        let error = resolve_media_relationship(
            &RelScopes::default(),
            RelScope::Layout,
            "rId9",
            &HashMap::new(),
            &mut HashMap::new(),
        )
        .unwrap_err();

        assert_eq!(
            error,
            RenderInputError::MissingRelationship {
                scope: RelScope::Layout,
                relationship_id: "rId9".to_owned(),
            }
        );
        assert!(error.to_string().contains("layout"));
        assert!(error.to_string().contains("rId9"));
    }

    #[test]
    fn render_input_contains_only_resolved_slides() {
        let input = RenderInput {
            slides: Vec::<ResolvedSlide>::new(),
            media: HashMap::new(),
            fonts: Vec::new(),
            metadata: None,
        };

        assert!(input.slides.is_empty());
        assert_eq!(
            std::any::type_name_of_val(&input.slides),
            "alloc::vec::Vec<rpptx_layout::ResolvedSlide>"
        );
    }

    #[test]
    fn rpptx_render_dependency_direction_is_one_way() {
        let manifest = include_str!("../Cargo.toml");
        for dependency in [
            "oxml-drawing.workspace = true",
            "oxml-layout.workspace = true",
            "oxml-media.workspace = true",
            "rpptx-layout.workspace = true",
            "rpptx-oxml.workspace = true",
        ] {
            assert!(manifest.contains(dependency), "missing {dependency}");
        }
        for oxml_manifest in [
            include_str!("../../oxml-core/Cargo.toml"),
            include_str!("../../oxml-drawing/Cargo.toml"),
            include_str!("../../oxml-layout/Cargo.toml"),
            include_str!("../../oxml-media/Cargo.toml"),
            include_str!("../../oxml-opc/Cargo.toml"),
            include_str!("../../oxml-pdf/Cargo.toml"),
        ] {
            assert!(!oxml_manifest.contains("rpptx-render"));
        }
        assert!(manifest.contains("version = \"0.0.0\""));
        assert!(manifest.contains("publish = false"));
    }
}
