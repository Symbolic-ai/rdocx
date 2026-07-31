//! Image embedding for PDF output (JPEG pass-through, PNG decompression).

use oxml_media::{ImageFormat, probe};

/// Decoded image data ready for PDF embedding.
pub(crate) struct DecodedImage {
    /// Raw pixel data (RGB or grayscale) or JPEG bytes for pass-through.
    pub data: Vec<u8>,
    /// Alpha channel data (if present), for a soft mask.
    pub alpha: Option<Vec<u8>>,
    /// Image width in pixels.
    pub width: u32,
    /// Image height in pixels.
    pub height: u32,
    /// Color space: "DeviceRGB", "DeviceGray".
    pub color_space: &'static str,
    /// Whether data is raw JPEG (pass through with DCTDecode).
    pub is_jpeg: bool,
}

/// Decode image bytes into a format suitable for PDF embedding.
///
/// JPEG images are passed through directly (the PDF viewer decodes them).
/// PNG images are decoded to raw RGB/RGBA pixels.
pub(crate) fn decode_image(data: &[u8], content_type: &str) -> Option<DecodedImage> {
    if content_type.contains("jpeg")
        || content_type.contains("jpg")
        || ImageFormat::sniff(data) == Some(ImageFormat::Jpeg)
    {
        decode_jpeg(data)
    } else {
        decode_png_or_other(data)
    }
}

fn decode_jpeg(data: &[u8]) -> Option<DecodedImage> {
    let info = probe(data)?;
    if info.format != ImageFormat::Jpeg {
        return None;
    }

    Some(DecodedImage {
        data: data.to_vec(),
        alpha: None,
        width: info.width_px,
        height: info.height_px,
        color_space: "DeviceRGB",
        is_jpeg: true,
    })
}

fn decode_png_or_other(data: &[u8]) -> Option<DecodedImage> {
    // Use a minimal PNG decoder. We need to decode to raw pixels.
    // For simplicity, we'll parse the PNG ourselves for common cases,
    // or fall back to a basic RGBA decode.
    decode_png(data)
}

/// Minimal PNG decoding to raw RGB + optional alpha.
fn decode_png(data: &[u8]) -> Option<DecodedImage> {
    // Validate PNG signature
    if data.len() < 8 || &data[0..8] != b"\x89PNG\r\n\x1a\n" {
        return None;
    }

    let mut pos = 8;
    let mut width = 0u32;
    let mut height = 0u32;
    let mut bit_depth = 0u8;
    let mut color_type = 0u8;
    let mut idat_data = Vec::new();
    // Palette entries as RGB triples, for indexed images.
    let mut palette: Vec<u8> = Vec::new();
    // Per-palette-entry alpha. Only meaningful for indexed images, where tRNS
    // is a list of alpha bytes indexed the same way as PLTE.
    let mut palette_alpha: Vec<u8> = Vec::new();

    while pos + 8 <= data.len() {
        let chunk_len =
            u32::from_be_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]) as usize;
        let chunk_type = &data[pos + 4..pos + 8];
        let chunk_data_start = pos + 8;
        let chunk_data_end = chunk_data_start + chunk_len;

        if chunk_data_end > data.len() {
            break;
        }

        match chunk_type {
            b"IHDR" => {
                if chunk_len >= 13 {
                    let d = &data[chunk_data_start..];
                    width = u32::from_be_bytes([d[0], d[1], d[2], d[3]]);
                    height = u32::from_be_bytes([d[4], d[5], d[6], d[7]]);
                    bit_depth = d[8];
                    color_type = d[9];
                }
            }
            b"IDAT" => {
                idat_data.extend_from_slice(&data[chunk_data_start..chunk_data_end]);
            }
            b"PLTE" => {
                palette.clear();
                palette.extend_from_slice(&data[chunk_data_start..chunk_data_end]);
            }
            b"tRNS" => {
                palette_alpha.clear();
                palette_alpha.extend_from_slice(&data[chunk_data_start..chunk_data_end]);
            }
            b"IEND" => break,
            _ => {}
        }

        pos = chunk_data_end + 4; // +4 for CRC
    }

    if width == 0 || height == 0 || idat_data.is_empty() || bit_depth != 8 {
        return None;
    }

    // Decompress the IDAT data (zlib deflate)
    let decompressed = miniz_oxide::inflate::decompress_to_vec_zlib(&idat_data).ok()?;

    // Unfilter the scanlines
    let channels: usize = match color_type {
        0 => 1, // Grayscale
        2 => 3, // RGB
        3 => 1, // Indexed: one palette index per pixel
        4 => 2, // Grayscale + Alpha
        6 => 4, // RGBA
        _ => return None,
    };

    let stride = width as usize * channels;
    let expected = (stride + 1) * height as usize; // +1 for filter byte per row
    if decompressed.len() < expected {
        return None;
    }

    let mut unfiltered = vec![0u8; stride * height as usize];
    let mut prev_row = vec![0u8; stride];

    for y in 0..height as usize {
        let row_start = y * (stride + 1);
        let filter_type = decompressed[row_start];
        let raw = &decompressed[row_start + 1..row_start + 1 + stride];

        let out_start = y * stride;
        let out = &mut unfiltered[out_start..out_start + stride];

        match filter_type {
            0 => {
                // None
                out.copy_from_slice(raw);
            }
            1 => {
                // Sub
                for i in 0..stride {
                    let a = if i >= channels { out[i - channels] } else { 0 };
                    out[i] = raw[i].wrapping_add(a);
                }
            }
            2 => {
                // Up
                for i in 0..stride {
                    out[i] = raw[i].wrapping_add(prev_row[i]);
                }
            }
            3 => {
                // Average
                for i in 0..stride {
                    let a = if i >= channels {
                        out[i - channels] as u16
                    } else {
                        0
                    };
                    let b = prev_row[i] as u16;
                    out[i] = raw[i].wrapping_add(((a + b) / 2) as u8);
                }
            }
            4 => {
                // Paeth
                for i in 0..stride {
                    let a = if i >= channels {
                        out[i - channels] as i32
                    } else {
                        0
                    };
                    let b = prev_row[i] as i32;
                    let c = if i >= channels {
                        prev_row[i - channels] as i32
                    } else {
                        0
                    };
                    out[i] = raw[i].wrapping_add(paeth_predictor(a, b, c));
                }
            }
            _ => {
                out.copy_from_slice(raw);
            }
        }

        prev_row.copy_from_slice(out);
    }

    // Separate color and alpha channels
    match color_type {
        0 => {
            // Grayscale
            Some(DecodedImage {
                data: unfiltered,
                alpha: None,
                width,
                height,
                color_space: "DeviceGray",
                is_jpeg: false,
            })
        }
        2 => {
            // RGB
            Some(DecodedImage {
                data: unfiltered,
                alpha: None,
                width,
                height,
                color_space: "DeviceRGB",
                is_jpeg: false,
            })
        }
        3 => {
            // Indexed colour: every byte is an index into the palette.
            // Word embeds these routinely, because most tools default to a
            // palette when an image has few colours.
            if palette.len() < 3 {
                return None;
            }
            let entries = palette.len() / 3;
            let pixel_count = (width * height) as usize;
            let mut rgb = Vec::with_capacity(pixel_count * 3);
            let mut alpha = Vec::with_capacity(pixel_count);
            let mut all_opaque = true;
            for &index in unfiltered.iter().take(pixel_count) {
                let i = index as usize;
                if i < entries {
                    rgb.extend_from_slice(&palette[i * 3..i * 3 + 3]);
                } else {
                    // An index past the end of the palette is malformed. Emit
                    // black rather than dropping the whole image.
                    rgb.extend_from_slice(&[0, 0, 0]);
                }
                // tRNS may cover only the first few entries. Anything it does
                // not mention is opaque.
                let a = palette_alpha.get(i).copied().unwrap_or(255);
                if a != 255 {
                    all_opaque = false;
                }
                alpha.push(a);
            }
            Some(DecodedImage {
                data: rgb,
                // Match the RGBA path: no SMask when nothing is transparent.
                alpha: if all_opaque { None } else { Some(alpha) },
                width,
                height,
                color_space: "DeviceRGB",
                is_jpeg: false,
            })
        }
        4 => {
            // Grayscale + Alpha
            let pixel_count = (width * height) as usize;
            let mut gray = Vec::with_capacity(pixel_count);
            let mut alpha = Vec::with_capacity(pixel_count);
            for i in 0..pixel_count {
                gray.push(unfiltered[i * 2]);
                alpha.push(unfiltered[i * 2 + 1]);
            }
            Some(DecodedImage {
                data: gray,
                alpha: Some(alpha),
                width,
                height,
                color_space: "DeviceGray",
                is_jpeg: false,
            })
        }
        6 => {
            // RGBA
            let pixel_count = (width * height) as usize;
            let mut rgb = Vec::with_capacity(pixel_count * 3);
            let mut alpha = Vec::with_capacity(pixel_count);
            let mut all_opaque = true;
            for i in 0..pixel_count {
                rgb.push(unfiltered[i * 4]);
                rgb.push(unfiltered[i * 4 + 1]);
                rgb.push(unfiltered[i * 4 + 2]);
                let a = unfiltered[i * 4 + 3];
                alpha.push(a);
                if a != 255 {
                    all_opaque = false;
                }
            }
            // Skip alpha channel if fully opaque — avoids unnecessary SMask
            // that can cause subtle color rendering differences in PDF viewers
            Some(DecodedImage {
                data: rgb,
                alpha: if all_opaque { None } else { Some(alpha) },
                width,
                height,
                color_space: "DeviceRGB",
                is_jpeg: false,
            })
        }
        _ => None,
    }
}

fn paeth_predictor(a: i32, b: i32, c: i32) -> u8 {
    let p = a + b - c;
    let pa = (p - a).abs();
    let pb = (p - b).abs();
    let pc = (p - c).abs();
    if pa <= pb && pa <= pc {
        a as u8
    } else if pb <= pc {
        b as u8
    } else {
        c as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_jpeg_pass_through() {
        // Minimal JPEG: SOI + SOF0 header + EOI
        // SOI marker
        let mut jpeg = vec![0xFF, 0xD8];
        // APP0 marker (dummy)
        jpeg.extend_from_slice(&[0xFF, 0xE0, 0x00, 0x02]);
        // SOF0 marker: length=17, precision=8, height=2, width=3, components=3
        jpeg.extend_from_slice(&[
            0xFF, 0xC0, 0x00, 0x11, 0x08, 0x00, 0x02, 0x00, 0x03, 0x03, 0x01, 0x11, 0x00, 0x02,
            0x11, 0x00, 0x03, 0x11, 0x00,
        ]);
        // EOI
        jpeg.extend_from_slice(&[0xFF, 0xD9]);

        let result = decode_image(&jpeg, "image/jpeg");
        assert!(result.is_some());
        let decoded = result.unwrap();
        assert!(decoded.is_jpeg);
        assert_eq!(decoded.width, 3);
        assert_eq!(decoded.height, 2);
        assert_eq!(decoded.color_space, "DeviceRGB");
        assert!(decoded.alpha.is_none());
    }

    #[test]
    fn decode_invalid_data_returns_none() {
        let result = decode_image(b"not an image", "image/png");
        assert!(result.is_none());
    }

    /// Build a PNG chunk. The decoder does not verify CRCs, so a zero CRC is
    /// enough to keep these fixtures readable.
    fn png_chunk(kind: &[u8; 4], body: &[u8]) -> Vec<u8> {
        let mut c = Vec::new();
        c.extend_from_slice(&(body.len() as u32).to_be_bytes());
        c.extend_from_slice(kind);
        c.extend_from_slice(body);
        c.extend_from_slice(&[0, 0, 0, 0]);
        c
    }

    /// Build an 8-bit indexed PNG from a palette and one index per pixel.
    fn indexed_png(
        width: u32,
        height: u32,
        palette: &[u8],
        trns: Option<&[u8]>,
        indices: &[u8],
    ) -> Vec<u8> {
        let mut ihdr = Vec::new();
        ihdr.extend_from_slice(&width.to_be_bytes());
        ihdr.extend_from_slice(&height.to_be_bytes());
        ihdr.push(8); // bit depth
        ihdr.push(3); // colour type: indexed
        ihdr.extend_from_slice(&[0, 0, 0]); // compression, filter, interlace

        // One filter byte per scanline, filter 0 meaning no filtering.
        let mut raw = Vec::new();
        for y in 0..height as usize {
            raw.push(0);
            raw.extend_from_slice(&indices[y * width as usize..(y + 1) * width as usize]);
        }
        let idat = miniz_oxide::deflate::compress_to_vec_zlib(&raw, 6);

        let mut out = b"\x89PNG\r\n\x1a\n".to_vec();
        out.extend_from_slice(&png_chunk(b"IHDR", &ihdr));
        if !palette.is_empty() {
            out.extend_from_slice(&png_chunk(b"PLTE", palette));
        }
        if let Some(t) = trns {
            out.extend_from_slice(&png_chunk(b"tRNS", t));
        }
        out.extend_from_slice(&png_chunk(b"IDAT", &idat));
        out.extend_from_slice(&png_chunk(b"IEND", &[]));
        out
    }

    /// Indexed PNGs used to fall into the catch-all and be dropped, so any
    /// document containing one lost that image with no warning.
    #[test]
    fn indexed_png_expands_palette_to_rgb() {
        // red, green, blue
        let palette = [255, 0, 0, 0, 255, 0, 0, 0, 255];
        let png = indexed_png(2, 2, &palette, None, &[0, 1, 2, 0]);

        let decoded = decode_image(&png, "image/png").expect("indexed PNG must decode");
        assert_eq!((decoded.width, decoded.height), (2, 2));
        assert_eq!(decoded.color_space, "DeviceRGB");
        assert!(!decoded.is_jpeg);
        assert_eq!(
            decoded.data,
            vec![255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 0, 0]
        );
        assert!(
            decoded.alpha.is_none(),
            "a fully opaque image should not carry an alpha channel"
        );
    }

    /// tRNS on an indexed image is per palette entry, and it may cover only
    /// the first few entries.
    #[test]
    fn indexed_png_honours_trns_alpha() {
        let palette = [255, 0, 0, 0, 255, 0];
        // Entry 0 fully transparent. Entry 1 is not mentioned, so opaque.
        let png = indexed_png(2, 1, &palette, Some(&[0]), &[0, 1]);

        let decoded = decode_image(&png, "image/png").expect("indexed PNG must decode");
        assert_eq!(decoded.data, vec![255, 0, 0, 0, 255, 0]);
        assert_eq!(decoded.alpha, Some(vec![0, 255]));
    }

    /// Colour type 3 without a PLTE chunk is malformed. Returning None is
    /// right, but it must not be confused with the palette case above.
    #[test]
    fn indexed_png_without_palette_is_rejected() {
        let png = indexed_png(1, 1, &[], None, &[0]);
        assert!(decode_image(&png, "image/png").is_none());
    }

    /// An index past the end of the palette is malformed input. It should not
    /// panic and should not discard the rest of the image.
    #[test]
    fn indexed_png_out_of_range_index_falls_back_to_black() {
        let palette = [255, 0, 0]; // a single entry, so index 5 is invalid
        let png = indexed_png(2, 1, &palette, None, &[0, 5]);

        let decoded = decode_image(&png, "image/png").expect("should still decode");
        assert_eq!(decoded.data, vec![255, 0, 0, 0, 0, 0]);
    }
}
