use eframe::egui::{
    Color32, FontId,
    text::{LayoutJob, TextFormat},
};
use regex::Regex;

/// Convert ANSI colored text into an [`egui::text::LayoutJob`].
/// Supports basic 8-color foreground codes (30-37 and 90-97) and reset (0).
pub fn ansi_to_job(text: &str, default_color: Color32) -> LayoutJob {
    let re = Regex::new(r"\x1b\[([0-9;]*)m").unwrap();
    let mut job = LayoutJob::default();
    let mut format = TextFormat::simple(FontId::monospace(14.0), default_color);
    let mut last = 0;
    for caps in re.captures_iter(text) {
        let m = caps.get(0).unwrap();
        if m.start() > last {
            job.append(&text[last..m.start()], 0.0, format.clone());
        }
        for code in caps[1].split(';') {
            match code {
                "0" | "" => {
                    format = TextFormat::simple(FontId::monospace(14.0), default_color);
                }
                "30" | "90" => format.color = Color32::BLACK,
                "31" | "91" => format.color = Color32::from_rgb(220, 50, 47),
                "32" | "92" => format.color = Color32::from_rgb(0, 175, 0),
                "33" | "93" => format.color = Color32::from_rgb(175, 127, 0),
                "34" | "94" => format.color = Color32::from_rgb(36, 114, 200),
                "35" | "95" => format.color = Color32::from_rgb(188, 63, 188),
                "36" | "96" => format.color = Color32::from_rgb(17, 168, 205),
                "37" | "97" => format.color = Color32::WHITE,
                _ => {}
            }
        }
        last = m.end();
    }
    if last < text.len() {
        job.append(&text[last..], 0.0, format);
    }
    job
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_colored_segments() {
        let job = ansi_to_job("\x1b[31mred\x1b[0m plain", Color32::WHITE);
        assert_eq!(job.text, "red plain");
        assert_eq!(job.sections.len(), 2);
        assert_eq!(job.sections[0].byte_range, 0..3);
        assert_eq!(job.sections[0].format.color, Color32::from_rgb(220, 50, 47));
        assert_eq!(job.sections[1].format.color, Color32::WHITE);
    }
}
