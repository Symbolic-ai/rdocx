//! Read-only native facade for tracked revision metadata.

use rdocx_oxml::CT_Revision;
pub use rdocx_oxml::RevisionKind;

/// An immutable view of one tracked revision in the main document.
#[derive(Debug, Clone, Copy)]
pub struct RevisionRef<'a> {
    pub(crate) inner: &'a CT_Revision,
}

impl RevisionRef<'_> {
    pub fn id(&self) -> i32 {
        self.inner.id()
    }

    pub fn author(&self) -> &str {
        self.inner.author()
    }

    pub fn timestamp(&self) -> Option<&str> {
        self.inner.timestamp()
    }

    pub fn kind(&self) -> RevisionKind {
        self.inner.kind()
    }
}
