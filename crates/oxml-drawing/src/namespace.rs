use oxml_core::OxmlError;
use quick_xml::XmlVersion;
use quick_xml::events::BytesStart;

pub const A_NS: &str = "http://schemas.openxmlformats.org/drawingml/2006/main";
pub const A_PREFIX: &str = "a";
pub const PIC_NS: &str = "http://schemas.openxmlformats.org/drawingml/2006/picture";
pub const PIC_PREFIX: &str = "pic";

/// Rejects a local binding that would change the meaning of a fixed `a:` tag.
///
/// Call this only for elements that a typed parser rewrites. Opaque elements
/// keep their producer namespace declarations and are not inspected.
pub(crate) fn reject_conflicting_a_prefix(element: &BytesStart<'_>) -> Result<(), OxmlError> {
    for attribute in element.attributes() {
        let attribute = attribute.map_err(OxmlError::from)?;
        if attribute.key.as_ref() == b"xmlns:a" {
            let value = attribute
                .decoded_and_normalized_value(XmlVersion::Implicit1_0, element.decoder())
                .map_err(OxmlError::from)?;
            if value != A_NS {
                return Err(OxmlError::InvalidValue(
                    "xmlns:a conflicts with the fixed DrawingML writer namespace".to_owned(),
                ));
            }
        }
    }
    Ok(())
}
