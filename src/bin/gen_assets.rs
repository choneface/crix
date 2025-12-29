/// Simple asset generator for text input placeholder images.
/// Run with: cargo run --bin gen_assets

use image::{Rgb, RgbImage};

fn main() {
    let width = 520;
    let height = 220;

    // Text input normal (light gray background, dark gray border)
    generate_text_input(
        "skins/classic/images/input_normal.png",
        width,
        height,
        Rgb([220, 220, 220]),
        Rgb([128, 128, 128]),
    );

    // Text input hover (slightly darker background)
    generate_text_input(
        "skins/classic/images/input_hover.png",
        width,
        height,
        Rgb([200, 200, 200]),
        Rgb([100, 100, 100]),
    );

    // Text input focused (white background, blue border)
    generate_text_input(
        "skins/classic/images/input_focused.png",
        width,
        height,
        Rgb([255, 255, 255]),
        Rgb([66, 135, 245]),
    );

    // Text input invalid (light red background, red border)
    generate_text_input(
        "skins/classic/images/input_invalid.png",
        width,
        height,
        Rgb([255, 230, 230]),
        Rgb([255, 80, 80]),
    );

    // Calculate button (green)
    generate_button(
        "skins/classic/images/calc_normal.png",
        80,
        24,
        Rgb([74, 78, 105]),
    );
    generate_button(
        "skins/classic/images/calc_hover.png",
        80,
        24,
        Rgb([154, 140, 152]),
    );
    generate_button(
        "skins/classic/images/calc_pressed.png",
        80,
        24,
        Rgb([34, 34, 59]),
    );

    println!("Assets generated successfully!");
}

fn generate_text_input(path: &str, width: u32, height: u32, fill: Rgb<u8>, border: Rgb<u8>) {
    let mut img = RgbImage::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let is_border = x == 0 || x == width - 1 || y == 0 || y == height - 1;
            let color = if is_border { border } else { fill };
            img.put_pixel(x, y, color);
        }
    }

    img.save(path).expect(&format!("Failed to save {}", path));
    println!("Created {}", path);
}

fn generate_button(path: &str, width: u32, height: u32, fill: Rgb<u8>) {
    let mut img = RgbImage::new(width, height);

    for y in 0..height {
        for x in 0..width {
            img.put_pixel(x, y, fill);
        }
    }

    img.save(path).expect(&format!("Failed to save {}", path));
    println!("Created {}", path);
}
