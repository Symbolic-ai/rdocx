//! Open Packaging Convention (OPC) reader/writer for OOXML packages.
//!
//! This crate handles the ZIP archive layer of OOXML files, including:
//! - Reading and writing ZIP-based OPC packages
//! - Parsing `[Content_Types].xml`
//! - Parsing `.rels` relationship files
//! - Navigating parts by URI and resolving relationships

pub mod content_types;
#[cfg(feature = "agile-encryption")]
mod encryption;
mod error;
mod package;
pub mod relationship;
#[cfg(feature = "digital-signatures")]
mod signature;

pub use content_types::{ContentType, ContentTypes};
pub use error::OpcError;
pub use package::{OpcPackage, PackagePart, PackageReadLimits};
pub use relationship::{Relationship, Relationships};
#[cfg(feature = "digital-signatures")]
pub use signature::{
    CoveredRelationship, SignatureIssue, SignatureReport, SignerCertificateIdentity,
};
