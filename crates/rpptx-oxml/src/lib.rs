#![allow(non_camel_case_types)]

pub mod connector;
pub mod graphic_frame;
pub mod namespace;
pub mod notes_parts;
pub mod picture;
pub mod placeholder;
pub mod presentation;
pub mod shape_tree;
pub mod slide_parts;

/// The canonical PresentationML main-part name.
pub const PRESENTATION_PART: &str = "/ppt/presentation.xml";
