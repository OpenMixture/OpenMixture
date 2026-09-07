//! PNG encoding and transfer metadata, with no material computation.
use mixture_core::{Diagnostic, DiagnosticCode, Stage};
pub(super) fn encoding_error(
    message: &str,
    error: impl std::error::Error + Send + Sync + 'static,
) -> Diagnostic {
    Diagnostic::error(DiagnosticCode::EncodingFailed, Stage::Encoding, message)
        .with_evidence("sourceMessage", error.to_string())
        .with_source(error)
        .with_suggestion(
            "Check the output path, parent directory, permissions, and available disk space.",
        )
}

pub(super) fn encode_png(
    width: u32,
    height: u32,
    pixels: &[u8],
    encoding: mixture_wgpu::OutputEncoding,
) -> Result<Vec<u8>, Box<Diagnostic>> {
    let mut bytes = Vec::new();
    let mut encoder = png::Encoder::new(&mut bytes, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    match encoding {
        mixture_wgpu::OutputEncoding::Srgb => {
            encoder.set_source_srgb(png::SrgbRenderingIntent::Perceptual)
        }
        mixture_wgpu::OutputEncoding::Linear => {
            encoder.set_source_gamma(png::ScaledFloat::new(1.0))
        }
    }
    let mut writer = encoder
        .write_header()
        .map_err(|error| encoding_error("Could not encode PNG header.", error))?;
    writer
        .write_image_data(pixels)
        .map_err(|error| encoding_error("Could not encode PNG pixels.", error))?;
    writer
        .finish()
        .map_err(|error| encoding_error("Could not finish PNG encoding.", error))?;
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn png_transfer_metadata_distinguishes_color_from_linear_data() {
        for encoding in [
            mixture_wgpu::OutputEncoding::Srgb,
            mixture_wgpu::OutputEncoding::Linear,
        ] {
            let bytes = encode_png(1, 1, &[137, 188, 225, 64], encoding).unwrap();
            let mut reader = png::Decoder::new(std::io::Cursor::new(bytes))
                .read_info()
                .unwrap();
            if encoding == mixture_wgpu::OutputEncoding::Linear {
                assert!(reader.info().srgb.is_none());
                assert_eq!(reader.info().gama_chunk.unwrap().into_scaled(), 100000);
            } else {
                assert!(reader.info().srgb.is_some());
            }
            let mut pixels = [0; 4];
            reader.next_frame(&mut pixels).unwrap();
            assert_eq!(pixels, [137, 188, 225, 64]);
        }
    }
}
