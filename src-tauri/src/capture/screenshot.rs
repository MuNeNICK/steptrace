use image::RgbaImage;
use xcap::Monitor;

use crate::session::model::CaptureMode;

/// Capture a screenshot based on the current capture mode.
pub fn capture(mode: &CaptureMode) -> Result<Vec<u8>, String> {
    let img = match mode {
        CaptureMode::FullScreen => capture_full_screen()?,
        CaptureMode::Window(id) => capture_window(*id)?,
        CaptureMode::Region { x, y, w, h } => capture_region(*x, *y, *w, *h)?,
    };
    encode_png(&img)
}

fn capture_full_screen() -> Result<RgbaImage, String> {
    let monitors = Monitor::all().map_err(|e| format!("Failed to list monitors: {}", e))?;
    let monitor = monitors.into_iter().next().ok_or("No monitor found")?;
    monitor
        .capture_image()
        .map_err(|e| format!("Failed to capture screen: {}", e))
}

fn capture_window(window_id: u32) -> Result<RgbaImage, String> {
    use xcap::Window;
    let windows = Window::all().map_err(|e| format!("Failed to list windows: {}", e))?;
    let window = windows
        .into_iter()
        .find(|w| w.id().unwrap_or(0) == window_id)
        .ok_or_else(|| format!("Window {} not found", window_id))?;
    window
        .capture_image()
        .map_err(|e| format!("Failed to capture window: {}", e))
}

fn capture_region(x: i32, y: i32, w: u32, h: u32) -> Result<RgbaImage, String> {
    let full = capture_full_screen()?;
    let cropped = image::imageops::crop_imm(&full, x as u32, y as u32, w, h).to_image();
    Ok(cropped)
}

fn encode_png(img: &RgbaImage) -> Result<Vec<u8>, String> {
    let mut buf = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new(&mut buf);
    image::ImageEncoder::write_image(
        encoder,
        img.as_raw(),
        img.width(),
        img.height(),
        image::ExtendedColorType::Rgba8,
    )
    .map_err(|e| format!("PNG encode error: {}", e))?;
    Ok(buf)
}

/// List all visible windows with their IDs and titles.
pub fn list_windows() -> Result<Vec<WindowInfo>, String> {
    use xcap::Window;
    let windows = Window::all().map_err(|e| format!("Failed to list windows: {}", e))?;
    Ok(windows
        .into_iter()
        .filter(|w| !w.is_minimized().unwrap_or(false))
        .filter_map(|w| {
            Some(WindowInfo {
                id: w.id().ok()?,
                title: w.title().ok()?,
                x: w.x().ok()?,
                y: w.y().ok()?,
                width: w.width().ok()?,
                height: w.height().ok()?,
            })
        })
        .collect())
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WindowInfo {
    pub id: u32,
    pub title: String,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}
