//! Format-neutral OOXML primitives.

pub mod core_properties;
pub mod error;
mod length;
pub mod raw_xml;
pub mod units;
pub mod xml;
pub mod xml_text;

pub use error::{OxmlError, Result};
pub use length::Length;
pub use units::{Emu, HalfPoint, Twips};
