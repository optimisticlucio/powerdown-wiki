use image::codecs::png::{CompressionType, FilterType, PngEncoder};
use image::codecs::webp::WebPEncoder;
use image::{DynamicImage, ImageEncoder, ImageFormat, ImageReader};
use std::io::Cursor;

pub fn compress_image_lossless(
    image_bytes: Vec<u8>,
    extension: Option<&str>,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // Try to determine format from extension first, then fall back to guessing
    let format = extension
        .and_then(|ext| {
            ImageFormat::from_mime_type(ext)
                .or(ImageFormat::from_extension(ext.trim_start_matches('.')))
        })
        .or_else(|| {
            ImageReader::new(Cursor::new(&image_bytes))
                .with_guessed_format()
                .ok()
                .and_then(|r| r.format())
        });

    // Load the image
    let mut reader = ImageReader::new(Cursor::new(&image_bytes));
    if let Some(fmt) = format {
        reader.set_format(fmt);
    }
    let img = reader.decode()?;

    // Determine if we can apply lossless compression
    match format {
        Some(ImageFormat::Jpeg) => {
            // Already lossy - return original
            Ok(image_bytes)
        }
        Some(ImageFormat::Gif) => {
            // TODO: How do I compress gifs without killing the animation?
            Ok(image_bytes)
        }
        Some(ImageFormat::WebP) => {
            // WebP could be lossy or lossless, but we can't easily tell
            // Safer to return original to avoid re-encoding lossy data
            Ok(image_bytes)
        }
        Some(ImageFormat::Png) => {
            // Re-compress PNG with maximum compression
            let compressed = compress_to_png_max(&img)?;

            // Only return compressed version if it's actually smaller
            if compressed.len() < image_bytes.len() {
                Ok(compressed)
            } else {
                Ok(image_bytes)
            }
        }
        Some(ImageFormat::Bmp)
        | Some(ImageFormat::Tiff)
        | Some(ImageFormat::Ico)
        | Some(ImageFormat::Pnm)
        | Some(ImageFormat::Tga)
        | Some(ImageFormat::Dds)
        | Some(ImageFormat::Hdr)
        | Some(ImageFormat::Farbfeld) => {
            // Convert to lossless WebP for better compression
            let compressed = compress_to_webp_lossless(&img)?;
            Ok(compressed)
        }
        Some(_) => {
            // Other formats - try to compress as WebP.
            compress_to_webp_lossless(&img)
        }
        None => {
            // Unknown format, try to compress as WebP
            println!("Unknown format. Attempting lossless WebP conversion...");
            compress_to_webp_lossless(&img)
        }
    }
}

fn compress_to_webp_lossless(img: &DynamicImage) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut output = Vec::new();
    let encoder = WebPEncoder::new_lossless(&mut output);
    encoder.encode(
        img.as_bytes(),
        img.width(),
        img.height(),
        img.color().into(),
    )?;
    Ok(output)
}

fn compress_to_png_max(img: &DynamicImage) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut output = Vec::new();
    let encoder =
        PngEncoder::new_with_quality(&mut output, CompressionType::Best, FilterType::Adaptive);
    encoder.write_image(
        img.as_bytes(),
        img.width(),
        img.height(),
        img.color().into(),
    )?;
    Ok(output)
}
