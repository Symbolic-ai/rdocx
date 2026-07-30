//! Format-neutral image metadata and media naming.

/// An image format supported by OOXML consumers.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ImageFormat {
    /// Portable Network Graphics.
    Png,
    /// Joint Photographic Experts Group.
    Jpeg,
    /// Graphics Interchange Format.
    Gif,
    /// Windows bitmap.
    Bmp,
    /// Tagged Image File Format.
    Tiff,
    /// WebP.
    Webp,
    /// Scalable Vector Graphics.
    Svg,
    /// Enhanced Metafile.
    Emf,
    /// Windows Metafile.
    Wmf,
}

impl ImageFormat {
    /// Identifies an image format from its magic bytes.
    pub fn sniff(data: &[u8]) -> Option<Self> {
        if data.starts_with(b"\x89PNG\r\n\x1a\n") {
            Some(Self::Png)
        } else if data.starts_with(b"\xff\xd8") {
            Some(Self::Jpeg)
        } else if data.starts_with(b"GIF87a") || data.starts_with(b"GIF89a") {
            Some(Self::Gif)
        } else if data.starts_with(b"BM") {
            Some(Self::Bmp)
        } else if data.starts_with(b"II*\0") || data.starts_with(b"MM\0*") {
            Some(Self::Tiff)
        } else if data.starts_with(b"RIFF") && data.get(8..12) == Some(b"WEBP") {
            Some(Self::Webp)
        } else if is_svg(data) {
            Some(Self::Svg)
        } else if data.get(..4) == Some(&[1, 0, 0, 0]) && data.get(40..44) == Some(b" EMF") {
            Some(Self::Emf)
        } else if data.starts_with(b"\xd7\xcd\xc6\x9a") || is_standard_wmf(data) {
            Some(Self::Wmf)
        } else {
            None
        }
    }

    /// Identifies an image format from a case-insensitive extension.
    pub fn from_extension(extension: &str) -> Option<Self> {
        match extension.to_ascii_lowercase().as_str() {
            "png" => Some(Self::Png),
            "jpg" | "jpeg" => Some(Self::Jpeg),
            "gif" => Some(Self::Gif),
            "bmp" => Some(Self::Bmp),
            "tif" | "tiff" => Some(Self::Tiff),
            "webp" => Some(Self::Webp),
            "svg" => Some(Self::Svg),
            "emf" => Some(Self::Emf),
            "wmf" | "wmz" => Some(Self::Wmf),
            _ => None,
        }
    }

    /// Returns the canonical filename extension without a leading dot.
    pub const fn extension(self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Jpeg => "jpeg",
            Self::Gif => "gif",
            Self::Bmp => "bmp",
            Self::Tiff => "tiff",
            Self::Webp => "webp",
            Self::Svg => "svg",
            Self::Emf => "emf",
            Self::Wmf => "wmf",
        }
    }

    /// Returns the canonical MIME content type.
    pub const fn content_type(self) -> &'static str {
        match self {
            Self::Png => "image/png",
            Self::Jpeg => "image/jpeg",
            Self::Gif => "image/gif",
            Self::Bmp => "image/bmp",
            Self::Tiff => "image/tiff",
            Self::Webp => "image/webp",
            Self::Svg => "image/svg+xml",
            Self::Emf => "image/emf",
            Self::Wmf => "image/wmf",
        }
    }
}

/// Resolves an image format from bytes first, then its filename extension.
///
/// Unknown images use PNG as the compatibility default.
pub fn resolve(data: &[u8], filename: &str) -> ImageFormat {
    ImageFormat::sniff(data)
        .or_else(|| {
            filename
                .rsplit_once('.')
                .and_then(|(_, extension)| ImageFormat::from_extension(extension))
        })
        .unwrap_or(ImageFormat::Png)
}

fn is_svg(data: &[u8]) -> bool {
    let data = data.strip_prefix(b"\xef\xbb\xbf").unwrap_or(data);
    let mut data = trim_ascii_start(data);
    loop {
        data = if data.starts_with(b"<?") {
            let Some(rest) = strip_through(data, b"?>") else {
                return false;
            };
            trim_ascii_start(rest)
        } else if data.starts_with(b"<!--") {
            let Some(rest) = strip_through(data, b"-->") else {
                return false;
            };
            trim_ascii_start(rest)
        } else if data.starts_with(b"<!DOCTYPE") {
            let Some(rest) = strip_doctype(data) else {
                return false;
            };
            trim_ascii_start(rest)
        } else {
            break;
        };
    }
    starts_with_svg_element(data)
}

fn starts_with_svg_element(data: &[u8]) -> bool {
    data.strip_prefix(b"<svg").is_some_and(|rest| {
        matches!(rest.first(), Some(b'>'))
            || rest.starts_with(b"/>")
            || matches!(rest.first(), Some(byte) if byte.is_ascii_whitespace())
    })
}

fn trim_ascii_start(mut data: &[u8]) -> &[u8] {
    while matches!(data.first(), Some(byte) if byte.is_ascii_whitespace()) {
        data = &data[1..];
    }
    data
}

fn strip_through<'a>(data: &'a [u8], terminator: &[u8]) -> Option<&'a [u8]> {
    let end = data
        .windows(terminator.len())
        .position(|window| window == terminator)?;
    data.get(end + terminator.len()..)
}

fn strip_doctype(data: &[u8]) -> Option<&[u8]> {
    let mut quote = None;
    let mut subset_depth = 0usize;
    let mut index = b"<!DOCTYPE".len();
    while index < data.len() {
        let byte = data[index];
        if let Some(delimiter) = quote {
            if byte == delimiter {
                quote = None;
            }
            index += 1;
            continue;
        }

        if data[index..].starts_with(b"<!--") {
            let rest = data.get(index + 4..)?;
            let end = rest.windows(3).position(|window| window == b"-->")?;
            index += 4 + end + 3;
            continue;
        }
        if data[index..].starts_with(b"<?") {
            let rest = data.get(index + 2..)?;
            let end = rest.windows(2).position(|window| window == b"?>")?;
            index += 2 + end + 2;
            continue;
        }

        match byte {
            b'\'' | b'"' => quote = Some(byte),
            b'[' => subset_depth += 1,
            b']' => subset_depth = subset_depth.saturating_sub(1),
            b'>' if subset_depth == 0 => return data.get(index + 1..),
            _ => {}
        }
        index += 1;
    }
    None
}

fn is_standard_wmf(data: &[u8]) -> bool {
    matches!(data.get(..2), Some([1, 0] | [2, 0]))
        && data.get(2..4) == Some(&[9, 0])
        && matches!(data.get(4..6), Some([0, 1] | [0, 3]))
}

#[cfg(test)]
mod tests {
    use super::{ImageFormat, resolve};

    const SVG_WITH_PROLOG: &[u8] = br#"<?xml version="1.0"?>
        <!-- generated -->
        <!DOCTYPE svg [
            <!-- a closing bracket ] and > inside a comment -->
            <?generated value="[ ] >"?>
            <!ELEMENT svg ANY>
        ]>
        <svg xmlns="http://www.w3.org/2000/svg">"#;

    #[test]
    fn every_supported_format_sniffs_from_magic_bytes() {
        let cases = [
            (ImageFormat::Png, b"\x89PNG\r\n\x1a\n".as_slice()),
            (ImageFormat::Jpeg, b"\xff\xd8\xff\xe0".as_slice()),
            (ImageFormat::Gif, b"GIF89a".as_slice()),
            (ImageFormat::Bmp, b"BM".as_slice()),
            (ImageFormat::Tiff, b"II*\0".as_slice()),
            (ImageFormat::Webp, b"RIFF\0\0\0\0WEBP".as_slice()),
            (ImageFormat::Svg, b"<svg xmlns=\"http://www.w3.org/2000/svg\">".as_slice()),
            (ImageFormat::Emf, b"\x01\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0 EMF".as_slice()),
            (ImageFormat::Wmf, b"\xd7\xcd\xc6\x9a".as_slice()),
        ];

        for (expected, data) in cases {
            assert_eq!(ImageFormat::sniff(data), Some(expected));
        }
    }

    #[test]
    fn extensions_and_content_types_are_canonical() {
        let cases = [
            ("png", ImageFormat::Png, "png", "image/png"),
            ("JPG", ImageFormat::Jpeg, "jpeg", "image/jpeg"),
            ("gif", ImageFormat::Gif, "gif", "image/gif"),
            ("bmp", ImageFormat::Bmp, "bmp", "image/bmp"),
            ("tif", ImageFormat::Tiff, "tiff", "image/tiff"),
            ("webp", ImageFormat::Webp, "webp", "image/webp"),
            ("svg", ImageFormat::Svg, "svg", "image/svg+xml"),
            ("emf", ImageFormat::Emf, "emf", "image/emf"),
            ("wmz", ImageFormat::Wmf, "wmf", "image/wmf"),
        ];

        for (alias, expected, extension, content_type) in cases {
            let actual = ImageFormat::from_extension(alias).expect("known extension");
            assert_eq!(actual, expected);
            assert_eq!(actual.extension(), extension);
            assert_eq!(actual.content_type(), content_type);
        }
    }

    #[test]
    fn sniffed_jpeg_overrides_a_misleading_png_extension() {
        assert_eq!(resolve(b"\xff\xd8\xff\xe0", "photo.png"), ImageFormat::Jpeg);
    }

    #[test]
    fn unknown_image_defaults_to_png() {
        assert_eq!(resolve(b"not an image", "photo.unknown"), ImageFormat::Png);
    }

    #[test]
    fn unknown_bytes_with_a_known_extension_resolve_from_the_extension() {
        assert_eq!(resolve(b"not an image", "photo.TIF"), ImageFormat::Tiff);
    }

    #[test]
    fn standard_nonplaceable_wmf_sniffs_from_its_meta_header() {
        assert_eq!(
            ImageFormat::sniff(b"\x01\0\x09\0\0\x03"),
            Some(ImageFormat::Wmf)
        );
    }

    #[test]
    fn svg_with_a_comment_and_doctype_in_its_prolog_is_sniffed() {
        assert_eq!(ImageFormat::sniff(SVG_WITH_PROLOG), Some(ImageFormat::Svg));
    }

    #[test]
    fn slash_after_svg_name_requires_an_empty_element_terminator() {
        assert_eq!(ImageFormat::sniff(b"<svg/not-an-image"), None);
        assert_eq!(ImageFormat::sniff(b"<svg/>"), Some(ImageFormat::Svg));
    }

    #[test]
    fn every_signature_prefix_returns_without_panicking() {
        let signatures: &[&[u8]] = &[
            b"\x89PNG\r\n\x1a\n",
            b"\xff\xd8\xff\xe0",
            b"GIF89a",
            b"BM",
            b"II*\0",
            b"RIFF\0\0\0\0WEBP",
            b"<svg xmlns=\"http://www.w3.org/2000/svg\">",
            SVG_WITH_PROLOG,
            b"\x01\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0 EMF",
            b"\xd7\xcd\xc6\x9a",
            b"\x01\0\x09\0\0\x03",
        ];

        for signature in signatures {
            for length in 0..signature.len() {
                let _ = ImageFormat::sniff(&signature[..length]);
            }
        }
    }
}
