use image::codecs::png::{CompressionType, FilterType, PngEncoder};
use image::codecs::webp::WebPEncoder;
use image::{DynamicImage, ImageEncoder, ImageFormat, ImageReader};
use std::io::Cursor;

pub fn compress_image_lossless(
    image_bytes: Vec<u8>,
    format: image::ImageFormat
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // Load the image
    let mut reader = ImageReader::new(Cursor::new(&image_bytes));
    reader.set_format(format);
    let img = reader.decode()?;

    // Determine if we can apply lossless compression
    match format {
        ImageFormat::Jpeg => {
            // Already lossy - return original
            Ok(image_bytes)
        }
        ImageFormat::Gif => {
            // TODO: How do I compress gifs without killing the animation?
            Ok(image_bytes)
        }
        ImageFormat::WebP => {
            // WebP could be lossy or lossless, but we can't easily tell
            // Safer to return original to avoid re-encoding lossy data
            Ok(image_bytes)
        }
        ImageFormat::Png => {
            // Re-compress PNG with maximum compression
            let compressed = compress_to_png_max(&img)?;

            // Only return compressed version if it's actually smaller
            if compressed.len() < image_bytes.len() {
                Ok(compressed)
            } else {
                Ok(image_bytes)
            }
        }
        ImageFormat::Bmp
        | ImageFormat::Tiff
        | ImageFormat::Ico
        | ImageFormat::Pnm
        | ImageFormat::Tga
        | ImageFormat::Dds
        | ImageFormat::Hdr
        | ImageFormat::Farbfeld => {
            // Convert to lossless WebP for better compression
            let compressed = compress_to_webp_lossless(&img)?;
            Ok(compressed)
        }
        _ => {
            // Other formats - try to compress as WebP.
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

/// When passed a file, tries to figure out what kind of file it is. Returns the guessed type.
pub fn get_filetype(image_bytes: &Vec<u8>) -> Result<Filetype, Box<dyn std::error::Error>>{
    // Let's see if it's an image.
    if let Some(image_format) = ImageReader::new(Cursor::new(image_bytes))
                .with_guessed_format()
                .ok()
                .and_then(|r| r.format())
    {
        return Ok(Filetype::Image(image_format))
    };

    // TODO: TEST FOR VIDEO FILE TYPE

    // If we reached this point, it's not a filetype we can handle.
    Ok(Filetype::Unknown)
}

pub enum Filetype {
    /// An image of some sort. May be an animated file, like a gif.
    Image(image::ImageFormat),
    /// A filetype that is not currently handled by our system.
    Unknown
}
