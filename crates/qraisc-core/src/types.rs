use serde::{Deserialize, Serialize};
use std::fmt;

/// Result of QR code validation including score and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Scannability score from 0-100
    pub score: u8,
    /// Whether at least one decoder successfully read the QR
    pub decodable: bool,
    /// Decoded content of the QR code
    pub content: Option<String>,
    /// QR code technical metadata
    pub metadata: Option<QrMetadata>,
    /// Results of stress tests used for scoring
    pub stress_results: StressResults,
}

/// Technical metadata about the QR code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QrMetadata {
    /// QR code version (1-40, determines size)
    pub version: u8,
    /// Error correction level
    pub error_correction: ErrorCorrectionLevel,
    /// Number of modules (21, 25, 29, etc.)
    pub modules: u8,
    /// List of decoders that successfully decoded this QR
    pub decoders_success: Vec<String>,
}

/// Results of stress tests for scannability scoring
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StressResults {
    /// Decoded at original resolution
    pub original: bool,
    /// Decoded at 50% scale
    pub downscale_50: bool,
    /// Decoded at 25% scale
    pub downscale_25: bool,
    /// Decoded with light blur (σ=1.0)
    pub blur_light: bool,
    /// Decoded with medium blur (σ=2.0)
    pub blur_medium: bool,
    /// Decoded with reduced contrast
    pub low_contrast: bool,
}

/// QR code error correction level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorCorrectionLevel {
    /// ~7% recovery capacity
    L,
    /// ~15% recovery capacity
    M,
    /// ~25% recovery capacity
    Q,
    /// ~30% recovery capacity
    H,
}

impl Default for ErrorCorrectionLevel {
    fn default() -> Self {
        Self::M
    }
}

impl fmt::Display for ErrorCorrectionLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::L => write!(f, "L"),
            Self::M => write!(f, "M"),
            Self::Q => write!(f, "Q"),
            Self::H => write!(f, "H"),
        }
    }
}

/// Simple decode result (without stress tests)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecodeResult {
    /// Decoded content
    pub content: String,
    /// Metadata if available
    pub metadata: Option<QrMetadata>,
}

/// Internal result from multi-decoder
#[derive(Debug, Clone)]
pub struct MultiDecodeResult {
    pub content: String,
    pub metadata: Option<QrMetadata>,
    pub decoders_success: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn validation_result_serializes_to_json() {
        let result = ValidationResult {
            score: 85,
            decodable: true,
            content: Some("https://example.com".to_string()),
            metadata: Some(QrMetadata {
                version: 3,
                error_correction: ErrorCorrectionLevel::H,
                modules: 29,
                decoders_success: vec!["rxing".to_string()],
            }),
            stress_results: StressResults::default(),
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"score\":85"));
        assert!(json.contains("\"decodable\":true"));
        assert!(json.contains("\"content\":\"https://example.com\""));
    }

    #[test]
    fn stress_results_default_all_false() {
        let sr = StressResults::default();
        assert!(!sr.original);
        assert!(!sr.downscale_50);
        assert!(!sr.downscale_25);
        assert!(!sr.blur_light);
        assert!(!sr.blur_medium);
        assert!(!sr.low_contrast);
    }

    #[test]
    fn error_correction_level_display() {
        assert_eq!(format!("{}", ErrorCorrectionLevel::L), "L");
        assert_eq!(format!("{}", ErrorCorrectionLevel::M), "M");
        assert_eq!(format!("{}", ErrorCorrectionLevel::Q), "Q");
        assert_eq!(format!("{}", ErrorCorrectionLevel::H), "H");
    }

    #[test]
    fn error_correction_level_default_is_m() {
        assert_eq!(ErrorCorrectionLevel::default(), ErrorCorrectionLevel::M);
    }

    #[test]
    fn qr_metadata_serializes() {
        let meta = QrMetadata {
            version: 5,
            error_correction: ErrorCorrectionLevel::Q,
            modules: 37,
            decoders_success: vec!["rxing".to_string(), "rqrr".to_string()],
        };

        let json = serde_json::to_string(&meta).unwrap();
        assert!(json.contains("\"version\":5"));
        assert!(json.contains("\"modules\":37"));
    }
}
