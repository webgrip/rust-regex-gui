use eframe::{App, Frame, egui};
use rfd::{FileDialog, MessageDialog, MessageLevel};

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
            let response = ui.checkbox(&mut self.dry_run, "Dry Run");
            if response.clicked() {
                let msg = if self.dry_run {
                    "Dry run enabled"
                } else {
                    "Dry run disabled"
                };
                MessageDialog::new()
                    .set_level(MessageLevel::Info)
                    .set_title("Dry Run")
                    .set_description(msg)
                    .show();
            }
            ui.separator();

            egui::Grid::new("rules_grid").show(ui, |ui| {
                ui.heading("From Regex");
                ui.heading("To Path");
                ui.end_row();

                for rule in &mut self.rules {
                    ui.horizontal(|ui| {
                        if ui.button("Select file").clicked() {
                            if let Some(path) = FileDialog::new().pick_file() {
                                rule.from = path.display().to_string();
                            }
                        }
                        ui.label(&rule.from);
                    });
                    ui.horizontal(|ui| {
                        if ui.button("Select destination").clicked() {
                            if let Some(path) = FileDialog::new().pick_folder() {
                                rule.to = path.display().to_string();
                            }
                        }
                        ui.label(&rule.to);
                    });
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
