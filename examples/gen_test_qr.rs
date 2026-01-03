//! Generate test QR codes for validation testing

use image::{DynamicImage, Luma};

fn main() {
    let content = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "https://qrcodeai.supernovae.studio".to_string());
    let output = std::env::args()
        .nth(2)
        .unwrap_or_else(|| "test-qr.png".to_string());

    let code = qrcode::QrCode::new(content.as_bytes()).expect("Failed to create QR code");
    let img = code.render::<Luma<u8>>().min_dimensions(400, 400).build();

    let dyn_img = DynamicImage::ImageLuma8(img);
    dyn_img.save(&output).expect("Failed to save image");

    eprintln!("✓ Generated QR: {} -> {}", content, output);
}
