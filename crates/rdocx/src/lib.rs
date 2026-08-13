//! High-level API for reading, writing, and converting DOCX documents.
//!
//! Provides a Python-docx-like interface for working with Word documents.
//!
//! # Examples
//!
//! ```no_run
//! use rdocx::Document;
//!
//! // Create a new document
//! let mut doc = Document::new();
//! doc.add_paragraph("Hello, World!");
//! doc.save("output.docx").unwrap();
//!
//! // Open an existing document
//! let doc = Document::open("existing.docx").unwrap();
//! for para in doc.paragraphs() {
//!     println!("{}", para.text());
//! }
//! ```

#![allow(clippy::too_many_arguments)]

mod document;
mod error;
pub mod paragraph;
pub mod run;
pub mod style;
pub mod table;

pub use document::{
    AccessibilityIssue, BodyContentRef, Document, ImageInfo, IssueSeverity, LinkInfo, ListLevel,
    ListNumberFormat, NumberingLevel, OutlineNode,
};
pub use error::{Error, Result};
pub use oxml_core::Length;
pub use paragraph::{
    Alignment, BorderStyle, HyperlinkRef, InlineContentRef, Paragraph, ParagraphContentRef,
    ParagraphRef, SectionBreak, SimpleFieldRef, TabAlignment, TabLeader,
};
pub use run::{BreakKind, DrawingRef, FieldKind, Run, RunContentRef, RunRef, UnderlineStyle};
pub use style::{Style, StyleBuilder};
pub use table::{Cell, CellContentRef, CellRef, Row, RowRef, Table, TableRef, VerticalAlignment};

#[cfg(test)]
mod tests {
    use super::Length;

    #[test]
    fn public_length_is_the_shared_type() {
        let _: oxml_core::Length = Length::emu(1);
    }
}
