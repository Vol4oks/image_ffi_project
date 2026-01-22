pub mod error;
pub mod plugin_loader;

use error::ImageProcessorError;

pub fn load_image_as_rgba(
    path: &std::path::Path,
) -> Result<(u32, u32, Vec<u8>), ImageProcessorError> {
    log::debug!("Loading image from: {:?}", path);

    let img = image::open(path)?;
    let rgba_img = img.to_rgba8();

    let (width, height) = rgba_img.dimensions();

    if width == 0 || height == 0 {
        return Err(ImageProcessorError::InvalidImageDimensions(width, height));
    }

    let rgba_data = rgba_img.into_raw();

    log::debug!(
        "Image loaded: {}x{}, buffer size: {} bytes",
        width,
        height,
        rgba_data.len()
    );

    Ok((width, height, rgba_data))
}

pub fn save_rgba_as_png(
    path: &std::path::Path,
    width: u32,
    height: u32,
    rgba_data: &[u8],
) -> Result<(), ImageProcessorError> {
    log::debug!("Saving image to: {:?}", path);

    let img_buffer: image::ImageBuffer<image::Rgba<u8>, &[u8]> =
        image::ImageBuffer::from_raw(width, height, rgba_data).ok_or_else(|| {
            ImageProcessorError::BufferSizeMismatch((width * height * 4) as usize, rgba_data.len())
        })?;

    img_buffer.save(path)?;

    log::debug!("Image saved successfully");

    Ok(())
}
