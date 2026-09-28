use anstream::ColorChoice;
use anstyle::{Ansi256Color, Color, RgbColor, Style};
use std::io::IsTerminal;

const ICON: &[&str] = &[
    ".####...............",
    "##..##..............",
    "#.##.#..............",
    "#.##.#..............",
    ".#.##.....##...##...",
    "..###.....####.####.",
    "...###....##########",
    "...###....####.####.",
    "...###....##...##...",
    "....###.............",
    "....###.............",
    ".....###............",
    ".....###............",
    "......###...........",
    "......##.#....####..",
    ".....#.##.#####..##.",
    ".....#.##.####.##.#.",
    "......#..#####.##.#.",
    ".......##....##..##.",
    "..............####..",
];
const BLUE: RgbColor = RgbColor(26, 159, 255);

/// Prints the app icon with the name and version beside it.
pub fn print() {
    let stdout = std::io::stdout();
    if !stdout.is_terminal() || anstream::AutoStream::choice(&stdout) == ColorChoice::Never {
        return;
    }

    let icon = render_icon();
    let width = icon
        .iter()
        .map(|line| line.chars().count())
        .max()
        .unwrap_or_default();
    let blue = Style::new().fg_color(Some(brand_color()));
    let title = blue.bold();
    let dim = Style::new().dimmed();
    let text = [
        format!(
            "{title}Full Steam Ahead{title:#} {dim}v{}{dim:#}",
            env!("CARGO_PKG_VERSION")
        ),
        "Import games from other launchers into Steam".to_string(),
    ];
    let first_text_row = icon.len().saturating_sub(text.len()) / 2;

    anstream::println!();
    for (row, line) in icon.iter().enumerate() {
        let text = row
            .checked_sub(first_text_row)
            .and_then(|index| text.get(index))
            .map_or("", String::as_str);
        let line = format!("  {blue}{line:<width$}{blue:#}   {text}");
        anstream::println!("{}", line.trim_end());
    }
    anstream::println!();
}

/// Terminals without a truecolor hint get the nearest 256 color instead.
/// The Windows console supports truecolor but never sets the hint.
fn brand_color() -> Color {
    if cfg!(windows) || anstyle_query::truecolor() {
        BLUE.into()
    } else {
        Ansi256Color(33).into()
    }
}

/// Draws two pixel rows per line with half blocks, so pixels stay square.
fn render_icon() -> Vec<String> {
    ICON.chunks(2)
        .map(|rows| {
            let mut bottom = rows.get(1).map_or("", |row| row).chars();
            rows[0]
                .chars()
                .map(|top| match (top == '#', bottom.next() == Some('#')) {
                    (true, true) => '█',
                    (true, false) => '▀',
                    (false, true) => '▄',
                    (false, false) => ' ',
                })
                .collect()
        })
        .collect()
}
