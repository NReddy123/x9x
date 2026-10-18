//! Decoding of check images (TIFF G4/JPEG/PNG) into GPU-uploadable frames.

use gpui_kit::RenderImage;
use image::{Frame, RgbaImage};
use std::sync::Arc;

pub struct Decoded {
    pub image: Arc<RenderImage>,
    pub width: u32,
    pub height: u32,
}

/// Decode image bytes. Runs off the UI thread.
pub fn decode(bytes: &[u8]) -> Result<Decoded, String> {
    let img = image::load_from_memory(bytes).map_err(|e| e.to_string())?;
    let mut rgba: RgbaImage = img.to_rgba8();
    let (width, height) = rgba.dimensions();
    // GPUI expects BGRA.
    for p in rgba.pixels_mut() {
        p.0.swap(0, 2);
    }
    let image = Arc::new(RenderImage::new([Frame::new(rgba)]));
    Ok(Decoded { image, width, height })
}

#[cfg(test)]
mod tests {
    #[test]
    fn decodes_group4_tiff() {
        let bytes = include_bytes!("../testdata/g4.tif");
        let d = super::decode(bytes).expect("decode");
        assert_eq!((d.width, d.height), (1200, 500));
    }
}
