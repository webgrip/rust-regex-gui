use eframe::egui::{self, Color32, FontFamily, FontId, TextStyle, Visuals};
use eframe::egui::{Context, Style};

/// Catppuccin Mocha inspired visuals.
pub fn catppuccin_visuals() -> Visuals {
    let mut v = Visuals::dark();
    // palette based on https://github.com/catppuccin/palette
    v.override_text_color = Some(Color32::from_rgb(205, 214, 244));
    v.widgets.noninteractive.bg_fill = Color32::from_rgb(30, 30, 46); // base
    v.widgets.inactive.bg_fill = Color32::from_rgb(49, 50, 68); // surface0
    v.widgets.hovered.bg_fill = Color32::from_rgb(69, 71, 90); // surface1
    v.widgets.active.bg_fill = Color32::from_rgb(88, 91, 112); // surface2
    v.selection.bg_fill = Color32::from_rgb(180, 190, 254); // lavender
    v.hyperlink_color = Color32::from_rgb(180, 190, 254);
    v.code_bg_color = Color32::from_rgb(49, 50, 68);
    v
}

/// Apply Catppuccin visuals and font styling to the given [`Context`].
pub fn apply_catppuccin(ctx: &Context) {
    ctx.set_visuals(catppuccin_visuals());

    let mut style: Style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(10.0, 8.0);
    style.text_styles = [
        (
            TextStyle::Heading,
            FontId::new(24.0, FontFamily::Proportional),
        ),
        (TextStyle::Body, FontId::new(16.0, FontFamily::Proportional)),
        (
            TextStyle::Monospace,
            FontId::new(14.0, FontFamily::Monospace),
        ),
        (
            TextStyle::Button,
            FontId::new(16.0, FontFamily::Proportional),
        ),
        (
            TextStyle::Small,
            FontId::new(12.0, FontFamily::Proportional),
        ),
    ]
    .into();
    ctx.set_style(style);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_lavender_selection() {
        let visuals = catppuccin_visuals();
        assert_eq!(visuals.selection.bg_fill, Color32::from_rgb(180, 190, 254));
    }

    #[test]
    fn apply_catppuccin_sets_heading_size() {
        let ctx = Context::default();
        apply_catppuccin(&ctx);
        let style = ctx.style();
        let heading = style.text_styles.get(&TextStyle::Heading).unwrap();
        assert_eq!(heading.size, 24.0);
    }
}
