use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ImageProcessorError {
    #[error("Invalid image dimensions:{0} {1}")]
    InvalidImageDimensions(u32, u32),
    #[error("Buffer size mismatch: expected {0}, got {1}")]
    BufferSizeMismatch(usize, usize),
    #[error("Image processing error: {0}")]
    Image(#[from] image::ImageError),
    #[error("Plugin loading error: {0}")]
    PluginLoading(#[from] libloading::Error),
    #[error("Invalid plagin path: {0}")]
    InvalidPlaginPath(PathBuf),
}
