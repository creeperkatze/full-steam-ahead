use image::imageops::FilterType;
use std::{fs, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=assets/logo.png");
    generate_logo();
    tauri_build::build();
}

fn generate_logo() {
    const ALPHA_THRESH: u8 = 128;
    const TARGET_COLS: u32 = 100;
    // Each terminal row covers 2 source pixel rows (top + bottom half-block).
    // With terminal chars ~2× taller than wide, this preserves aspect ratio.
    const CHAR_ASPECT: f64 = 0.45;
    const PAD: u32 = 2;

    let img = image::load_from_memory(include_bytes!("assets/logo.png"))
        .expect("failed to load assets/logo.png");

    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();

    // Bounding box of opaque pixels
    let (mut x0, mut y0) = (w, h);
    let (mut x1, mut y1) = (0u32, 0u32);
    for y in 0..h {
        for x in 0..w {
            if rgba.get_pixel(x, y)[3] >= ALPHA_THRESH {
                x0 = x0.min(x);
                x1 = x1.max(x);
                y0 = y0.min(y);
                y1 = y1.max(y);
            }
        }
    }
    assert!(x0 <= x1, "no opaque pixels in assets/logo.png");

    let cx = x0.saturating_sub(PAD);
    let cy = y0.saturating_sub(PAD);
    let cw = (x1 + PAD + 1).min(w) - cx;
    let ch = (y1 + PAD + 1).min(h) - cy;

    let cropped = img.crop_imm(cx, cy, cw, ch);

    // We need twice as many source rows as terminal rows.
    let term_rows =
        ((ch as f64 * TARGET_COLS as f64 / cw as f64 * CHAR_ASPECT).round() as u32).max(1);
    let src_rows = term_rows * 2;

    let resized = cropped.resize_exact(TARGET_COLS, src_rows, FilterType::Lanczos3);
    let px = resized.to_rgba8();

    let mut lines: Vec<String> = Vec::new();
    for row in 0..term_rows {
        let mut line = String::new();
        for x in 0..TARGET_COLS {
            let [tr, tg, tb, ta] = px.get_pixel(x, row * 2).0;
            let [br, bg, bb, ba] = px.get_pixel(x, row * 2 + 1).0;

            let top = ta >= ALPHA_THRESH;
            let bot = ba >= ALPHA_THRESH;

            match (top, bot) {
                (false, false) => line.push(' '),
                (true, false) => {
                    line.push_str(&format!("\x1b[38;2;{tr};{tg};{tb}m▀\x1b[0m"));
                }
                (false, true) => {
                    line.push_str(&format!("\x1b[38;2;{br};{bg};{bb}m▄\x1b[0m"));
                }
                (true, true) => {
                    // fg = top pixel colour, bg = bottom pixel colour
                    line.push_str(&format!(
                        "\x1b[38;2;{tr};{tg};{tb}m\x1b[48;2;{br};{bg};{bb}m▀\x1b[0m"
                    ));
                }
            }
        }
        lines.push(line.trim_end_matches(' ').to_string());
    }

    while lines.first().is_some_and(|l| l.is_empty()) {
        lines.remove(0);
    }
    while lines.last().is_some_and(|l| l.is_empty()) {
        lines.pop();
    }

    let out = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("logo.txt");
    fs::write(out, lines.join("\n")).unwrap();
}
