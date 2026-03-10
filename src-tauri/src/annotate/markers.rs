use image::{Rgba, RgbaImage};
use imageproc::drawing;

const RED: Rgba<u8> = Rgba([255, 50, 50, 230]);
const GREEN: Rgba<u8> = Rgba([50, 200, 50, 200]);
const WHITE: Rgba<u8> = Rgba([255, 255, 255, 255]);

/// Draw a click marker (red circle with step number) at the given position.
pub fn draw_click_marker(img: &mut RgbaImage, x: i32, y: i32, step_number: u32) {
    let radius = 18i32;

    // Draw filled circle
    drawing::draw_filled_circle_mut(img, (x, y), radius, RED);

    // Draw step number text (simple: single/double digit centered)
    let text = step_number.to_string();
    let text_x = x - (text.len() as i32 * 4);
    let text_y = y - 6;

    // Draw text background for readability
    for dx in -1..=1i32 {
        for dy in -1..=1i32 {
            draw_simple_text(img, (text_x + dx) as u32, (text_y + dy) as u32, &text, WHITE);
        }
    }
}

/// Draw a window highlight border around the given rectangle.
pub fn draw_window_highlight(img: &mut RgbaImage, x: i32, y: i32, w: u32, h: u32) {
    let thickness = 3;
    for t in 0..thickness {
        let rect = imageproc::rect::Rect::at(x - t as i32, y - t as i32)
            .of_size(w + 2 * t, h + 2 * t);
        drawing::draw_hollow_rect_mut(img, rect, GREEN);
    }
}

/// Draw step number badge in the top-left corner.
pub fn draw_step_badge(img: &mut RgbaImage, step_number: u32) {
    let badge_w = 60u32;
    let badge_h = 28u32;
    let rect = imageproc::rect::Rect::at(8, 8).of_size(badge_w, badge_h);
    drawing::draw_filled_rect_mut(img, rect, RED);

    let text = format!("#{}", step_number);
    draw_simple_text(img, 16, 14, &text, WHITE);
}

/// Very simple text rendering using pixel drawing.
/// This is a placeholder until ab_glyph font rendering is set up.
fn draw_simple_text(img: &mut RgbaImage, x: u32, y: u32, text: &str, color: Rgba<u8>) {
    // Simple 5x7 pixel font for digits and #
    let _x = x;
    let _y = y;
    // For now, just draw a small marker - proper font rendering will be added
    // when we bundle a TTF font with ab_glyph
    for (i, _ch) in text.chars().enumerate() {
        let cx = x + (i as u32 * 8);
        if cx + 5 < img.width() && y + 7 < img.height() {
            // Draw a small dot as placeholder per character
            drawing::draw_filled_circle_mut(img, (cx as i32 + 3, _y as i32 + 3), 3, color);
        }
    }
}
