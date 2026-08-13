//! Borrowed access to preserved XML that the high-level facade does not model.

use std::ops::Deref;

use rdocx_oxml::raw_xml::RawXml;

/// A preserved XML subtree together with its resolved source identity.
#[derive(Debug, Clone, Copy)]
pub struct UnsupportedXmlRef<'a> {
    inner: &'a RawXml,
}

impl<'a> UnsupportedXmlRef<'a> {
    pub(crate) fn new(inner: &'a RawXml) -> Self {
        Self { inner }
    }

    /// Original subtree bytes.
    pub fn bytes(self) -> &'a [u8] {
        self.inner.bytes()
    }

    /// Qualified element name as it appeared in the source XML.
    pub fn qualified_name(self) -> &'a str {
        &self.inner.name().qualified
    }

    /// Local element name.
    pub fn local_name(self) -> &'a str {
        &self.inner.name().local
    }

    /// Namespace URI resolved at the element's source location.
    pub fn namespace_uri(self) -> Option<&'a str> {
        self.inner.name().namespace_uri.as_deref()
    }

    /// Namespace URI bound to `prefix` at the element's source location.
    pub fn namespace_uri_for_prefix(self, prefix: &str) -> Option<&'a str> {
        self.inner.namespaces().namespace_uri(prefix)
    }

    /// All namespace bindings in scope at the source element.
    pub fn namespace_bindings(self) -> impl Iterator<Item = (&'a str, &'a str)> {
        self.inner
            .namespaces()
            .bindings()
            .iter()
            .map(|(prefix, uri)| (prefix.as_str(), uri.as_str()))
    }
}

impl AsRef<[u8]> for UnsupportedXmlRef<'_> {
    fn as_ref(&self) -> &[u8] {
        self.inner.bytes()
    }
}

impl Deref for UnsupportedXmlRef<'_> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.inner.bytes()
    }
}
