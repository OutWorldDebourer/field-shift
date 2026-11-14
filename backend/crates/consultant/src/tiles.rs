//! Detail crops for the model. Claude reduces each image to about 1.15 megapixels, which can erase tiny insects
//! (spider mites, whitefly nymphs). For large photos we also send the four quarters at native resolution.
use base64::Engine;
use std::io::Cursor;

pub struct Tile { pub label: &'static str, pub jpeg_b64: String }

const LABELS: [&str; 4] = ["top-left quarter", "top-right quarter", "bottom-left quarter", "bottom-right quarter"];

/// Four native-resolution quarters of a photo larger than ~1800 px on its long side; empty for small photos.
pub fn quarters(bytes: &[u8]) -> Vec<Tile> {
    let Ok(img) = image::load_from_memory(bytes) else { return vec![] };
    let (w, h) = (img.width(), img.height());
    if w.max(h) < 1800 { return vec![]; }
    let (hw, hh) = (w / 2, h / 2);
    let boxes = [(0, 0), (hw, 0), (0, hh), (hw, hh)];
    boxes.iter().zip(LABELS).filter_map(|((x, y), label)| {
        let crop = img.crop_imm(*x, *y, hw, hh).to_rgb8();
        let mut out = Cursor::new(Vec::new());
        let mut enc = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, 90);
        enc.encode_image(&crop).ok()?;
        Some(Tile { label, jpeg_b64: base64::engine::general_purpose::STANDARD.encode(out.into_inner()) })
    }).collect()
}

/// The whole photo as the model will see it: unchanged when small, otherwise a 1568 px JPEG (q 92), which is what
/// Claude reduces it to anyway and keeps each image under the API's 5 MB limit. Returns (media type, base64).
pub fn model_view(bytes: &[u8], mime: &str) -> (String, String) {
    let b64 = |b: &[u8]| base64::engine::general_purpose::STANDARD.encode(b);
    let Ok(img) = image::load_from_memory(bytes) else { return (mime.to_string(), b64(bytes)) };
    if img.width().max(img.height()) <= 1568 && bytes.len() <= 3_500_000 { return (mime.to_string(), b64(bytes)); }
    let small = img.resize(1568, 1568, image::imageops::FilterType::Lanczos3).to_rgb8();
    let mut out = Cursor::new(Vec::new());
    let mut enc = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, 92);
    if enc.encode_image(&small).is_err() { return (mime.to_string(), b64(bytes)); }
    ("image/jpeg".to_string(), b64(&out.into_inner()))
}

#[cfg(test)]
mod tests {
    #[test]
    fn small_photos_get_no_tiles_and_large_ones_get_four() {
        let mut small = std::io::Cursor::new(Vec::new());
        image::RgbImage::new(800, 600).write_to(&mut small, image::ImageFormat::Png).unwrap();
        assert!(super::quarters(small.get_ref()).is_empty());
        let mut big = std::io::Cursor::new(Vec::new());
        image::RgbImage::new(2400, 1800).write_to(&mut big, image::ImageFormat::Png).unwrap();
        assert_eq!(super::quarters(big.get_ref()).len(), 4);
    }
}
