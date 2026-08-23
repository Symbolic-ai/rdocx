//! Error types for OPC operations.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum OpcError {
    #[error("ZIP error: {0}")]
    Zip(#[from] zip::result::ZipError),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("XML error: {0}")]
    Xml(#[from] quick_xml::Error),

    #[error("XML attribute error: {0}")]
    XmlAttr(#[from] quick_xml::events::attributes::AttrError),

    #[error("part not found: {0}")]
    PartNotFound(String),

    #[error("OPC package exceeds {kind} limit of {limit}")]
    PackageLimitExceeded { kind: &'static str, limit: u64 },

    #[error("invalid content types XML")]
    InvalidContentTypes,

    #[error("invalid relationship XML")]
    InvalidRelationship,

    #[cfg(feature = "agile-encryption")]
    #[error("unsupported encryption: {0}")]
    UnsupportedEncryption(&'static str),

    #[cfg(feature = "agile-encryption")]
    #[error("unsupported encryption algorithm: {0}")]
    UnsupportedEncryptionAlgorithm(String),

    #[cfg(feature = "agile-encryption")]
    #[error("invalid agile encryption information")]
    InvalidEncryptionInfo,

    #[cfg(feature = "agile-encryption")]
    #[error("invalid password")]
    InvalidPassword,

    #[cfg(feature = "agile-encryption")]
    #[error("encrypted package failed integrity verification")]
    EncryptedPackageIntegrity,

    #[cfg(feature = "agile-encryption")]
    #[error("invalid encrypted package stream")]
    InvalidEncryptedPackage,

    #[cfg(feature = "digital-signatures")]
    #[error("invalid digital signature XML: {0}")]
    InvalidSignatureXml(String),

    #[cfg(feature = "digital-signatures")]
    #[error("unsupported digital signature {kind}: {algorithm}")]
    UnsupportedSignatureAlgorithm {
        kind: &'static str,
        algorithm: String,
    },

    #[cfg(feature = "digital-signatures")]
    #[error("invalid signing certificate: {0}")]
    InvalidSigningCertificate(String),

    #[cfg(feature = "digital-signatures")]
    #[error("invalid signing key: {0}")]
    InvalidSigningKey(String),

    #[cfg(feature = "digital-signatures")]
    #[error("digital signature creation failed: {0}")]
    SignatureCreationFailed(String),

    #[error("UTF-8 error: {0}")]
    Utf8(#[from] std::str::Utf8Error),
}

pub type Result<T> = std::result::Result<T, OpcError>;
