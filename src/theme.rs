use eframe::egui::{Color32, Visuals};

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_lavender_selection() {
        let visuals = catppuccin_visuals();
        assert_eq!(visuals.selection.bg_fill, Color32::from_rgb(180, 190, 254));
    }
}
