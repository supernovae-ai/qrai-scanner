use thiserror::Error;

/// Errors that can occur during QR validation
#[derive(Debug, Error)]
pub enum QraiError {
    /// Failed to load or parse image
    #[error("Failed to load image: {0}")]
    ImageLoad(String),

    /// No QR code found in the image
    #[error("No QR code found in image")]
    DecodeFailed,

    /// IO error (file operations)
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Image processing error
    #[error("Image processing error: {0}")]
    ImageProcessing(String),

    /// Image dimensions exceed safety limits
    #[error("Image too large: {width}x{height} exceeds maximum {max_dimension}x{max_dimension}")]
    DimensionsTooLarge {
        width: u32,
        height: u32,
        max_dimension: u32,
    },

    /// Failed to create image buffer (dimension/data mismatch)
    #[error("Failed to create image buffer: expected {expected} bytes, got {actual}")]
    BufferMismatch { expected: usize, actual: usize },

    /// Integer overflow in dimension calculation
    #[error("Dimension overflow: {width} x {height} overflows")]
    DimensionOverflow { width: u32, height: u32 },
}

pub type Result<T> = std::result::Result<T, QraiError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display_image_load() {
        let err = QraiError::ImageLoad("invalid format".to_string());
        let msg = err.to_string();
        assert!(msg.contains("Failed to load image"));
        assert!(msg.contains("invalid format"));
    }

    #[test]
    fn error_display_decode_failed() {
        let err = QraiError::DecodeFailed;
        assert!(err.to_string().contains("No QR code"));
    }

    #[test]
    fn error_display_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err = QraiError::Io(io_err);
        assert!(err.to_string().contains("IO error"));
    }

    #[test]
    fn error_display_image_processing() {
        let err = QraiError::ImageProcessing("resize failed".to_string());
        assert!(err.to_string().contains("Image processing error"));
    }
}
