use image::codecs::png::{CompressionType, FilterType, PngEncoder};
use image::codecs::webp::WebPEncoder;
use image::{DynamicImage, ImageEncoder, ImageFormat, ImageReader};
use std::io::Cursor;

pub fn compress_image_lossless(
    image_bytes: Vec<u8>,
    format: infer::Type
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // Convert the infer type to ImageReader's.
    let format = ImageFormat::from_extension(format.extension()).unwrap();

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

pub struct LossyCompressionSettings {
    pub max_width: Option<u32>,
    pub max_height: Option<u32>,
    pub quality: u8, // 1-100, higher = better quality
}

impl Default for LossyCompressionSettings {
    fn default() -> Self {
        Self {
            max_width: None,
            max_height: None,
            quality: 85,
        }
    }
}

pub fn compress_image_lossy(
    image_bytes: Vec<u8>,
    format: infer::Type,
    settings: Option<LossyCompressionSettings>,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // Use default settings if none provided
    let settings = settings.unwrap_or_default();

    // Convert the infer type to ImageReader's format
    let format = ImageFormat::from_extension(format.extension()).unwrap();

    // Load the image
    let mut reader = ImageReader::new(Cursor::new(&image_bytes));
    reader.set_format(format);
    let mut img = reader.decode()?;

    // Resize if dimensions are specified
    if let (Some(max_w), Some(max_h)) = (settings.max_width, settings.max_height) {
        img = resize_if_needed(img, max_w, max_h);
    } else if let Some(max_w) = settings.max_width {
        if img.width() > max_w {
            let ratio = max_w as f32 / img.width() as f32;
            let new_height = (img.height() as f32 * ratio) as u32;
            img = img.resize(max_w, new_height, image::imageops::FilterType::Lanczos3);
        }
    } else if let Some(max_h) = settings.max_height {
        if img.height() > max_h {
            let ratio = max_h as f32 / img.height() as f32;
            let new_width = (img.width() as f32 * ratio) as u32;
            img = img.resize(new_width, max_h, image::imageops::FilterType::Lanczos3);
        }
    }

    // Compress to lossy WebP
    let compressed = compress_to_webp_lossy(&img, settings.quality)?;
    
    Ok(compressed)
}

fn resize_if_needed(img: DynamicImage, max_width: u32, max_height: u32) -> DynamicImage {
    let (width, height) = (img.width(), img.height());
    
    // Calculate if resizing is needed
    if width <= max_width && height <= max_height {
        return img;
    }
    
    // Calculate the scaling ratio to fit within bounds
    let width_ratio = max_width as f32 / width as f32;
    let height_ratio = max_height as f32 / height as f32;
    let ratio = width_ratio.min(height_ratio);
    
    let new_width = (width as f32 * ratio) as u32;
    let new_height = (height as f32 * ratio) as u32;
    
    img.resize(new_width, new_height, image::imageops::FilterType::Lanczos3)
}

fn compress_to_webp_lossy(img: &DynamicImage, quality: u8) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let rgba = img.to_rgba8();
    let encoder = webp::Encoder::from_rgba(&rgba, img.width(), img.height());
    let webp = encoder.encode(quality as f32);
    Ok(webp.to_vec())
}