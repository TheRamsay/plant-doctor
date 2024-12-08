use embedded_graphics::{
    pixelcolor::{BinaryColor, Rgb565},
    prelude::*,
    primitives::Rectangle,
};
use epd_waveshare::color::Color;
use image::{DynamicImage, GenericImageView, ImageFormat};

pub fn load_and_process_image(
    file_path: &str,
    display_width: u32,
    display_height: u32,
) -> DynamicImage {
    // Load the JPG image
    let img = image::open(file_path).expect("Failed to open image");

    // Resize the image to match the display resolution
    let resized = img.resize(
        display_width,
        display_height,
        image::imageops::FilterType::Nearest,
    );

    // Convert the image to grayscale or appropriate format
    let grayscale = resized.to_luma8(); // Convert to 8-bit grayscale
    DynamicImage::ImageLuma8(grayscale)
}

pub fn image_to_binary(image: &DynamicImage, threshold: u8) -> Vec<u8> {
    let mut binary_image = Vec::new();

    // Loop through each pixel
    for pixel in image.to_luma8().pixels() {
        let color = if pixel[0] > threshold { 255 } else { 0 };
        binary_image.push(color);
    }

    binary_image
}
