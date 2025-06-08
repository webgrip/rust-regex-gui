use eframe::{egui, App, Frame};

#[derive(Default)]
struct Rule {
    from: String,
    to: String,
}

#[derive(Default)]
struct RegexApp {
    dry_run: bool,
    rules: Vec<Rule>,
}

impl App for RegexApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Regex Rules");
            ui.checkbox(&mut self.dry_run, "Dry Run");
            ui.separator();

            egui::Grid::new("rules_grid").show(ui, |ui| {
                ui.heading("From Regex");
                ui.heading("To Path");
                ui.end_row();

                for rule in &mut self.rules {
                    ui.text_edit_singleline(&mut rule.from);
                    ui.text_edit_singleline(&mut rule.to);
                    ui.end_row();
                }
            });

            if ui.button("Add Rule").clicked() {
                self.rules.push(Rule::default());
            }
        });
    }
}

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Regex GUI",
        native_options,
        Box::new(|_cc| Ok(Box::new(RegexApp::default()))),
    )
}
