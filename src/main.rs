use eframe::{App, Frame, egui};
use egui::{CentralPanel, Margin, RichText, TopBottomPanel};

#[cfg(target_arch = "wasm32")]
use console_error_panic_hook;
#[cfg(target_arch = "wasm32")]
use eframe::web_sys::HtmlCanvasElement;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local;

mod ansi;
mod application;
mod domain;
mod telemetry;
mod theme;

use ansi::ansi_to_job;
use application::Renamer;
use domain::Rule;
use std::sync::Arc;
use telemetry::{MemoryWriter, TracingLogger, init_tracing};
use theme::catppuccin_visuals;
use tracing::info;
use tracing_subscriber::filter::LevelFilter;

pub struct RegexApp {
    dry_run: bool,
    rules: Vec<Rule>,
    renamer: Renamer,
    log_writer: MemoryWriter,
}

impl RegexApp {
    pub fn new() -> Self {
        let log_writer = init_tracing(LevelFilter::INFO);
        info!("RegexApp started");
        let logger = Arc::new(TracingLogger);
        let renamer = Renamer::new(logger);
        Self {
            dry_run: false,
            rules: vec![Rule::default()],
            renamer,
            log_writer,
        }
    }

    #[cfg(test)]
    fn new_for_tests() -> Self {
        let logger = Arc::new(TracingLogger);
        let renamer = Renamer::new(logger);
        Self {
            dry_run: false,
            rules: vec![Rule::default()],
            renamer,
            log_writer: MemoryWriter::default(),
        }
    }

    pub fn add_rule(&mut self) {
        self.rules.push(Rule::default());
    }

    pub fn remove_rule(&mut self, index: usize) {
        if index < self.rules.len() {
            self.rules.remove(index);
        }
    }
}

impl Default for RegexApp {
    fn default() -> Self {
        Self::new()
    }
}

impl App for RegexApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        // --- one‑off global style tweaks ----------------------------------
        ctx.set_style({
            let mut style = (*ctx.style()).clone();
            style.spacing.item_spacing = egui::vec2(10.0, 8.0);
            style
        });
        ctx.set_visuals(catppuccin_visuals());

        // --- main UI -------------------------------------------------------
        CentralPanel::default()
            .frame(egui::Frame::NONE.inner_margin(Margin::same(16)))
            .show(ctx, |ui| {
                // title ----------------------------------------------------
                ui.vertical_centered(|ui| {
                    ui.heading(RichText::new("🔧  Regex Renamer").size(24.0).strong());
                });
                ui.add_space(8.0);
                let changed = ui
                    .checkbox(&mut self.dry_run, "Dry‑run (no files are actually renamed)")
                    .changed();
                if changed {
                    info!("dry_run toggled: {}", self.dry_run);
                }
                ui.separator();

                // rules table ---------------------------------------------
                ui.heading("Rules");
                egui::ScrollArea::vertical()
                    .max_height(200.0)
                    .show(ui, |ui| {
                        egui::Grid::new("rules_grid").striped(true).show(ui, |ui| {
                            ui.label(RichText::new("From Regex").strong());
                            ui.label(RichText::new("To Path").strong());
                            ui.label("");
                            ui.end_row();

                            let regex_width = 240.0;
                            let path_width = 240.0;

                            let mut index = 0usize;
                            while index < self.rules.len() {
                                let rule = &mut self.rules[index];
                                ui.add_sized(
                                    [regex_width, 0.0],
                                    egui::TextEdit::singleline(&mut rule.from).hint_text("regex"),
                                );
                                ui.add_sized(
                                    [path_width, 0.0],
                                    egui::TextEdit::singleline(&mut rule.to)
                                        .hint_text("destination"),
                                );
                                if ui.button("❌").on_hover_text("Remove rule").clicked() {
                                    self.remove_rule(index);
                                    continue;
                                }
                                ui.end_row();
                                index += 1;
                            }
                        });
                    });

                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button("➕  Add rule").clicked() {
                        self.add_rule();
                        info!("Added new rule");
                    }
                    if ui.button("▶  Execute").clicked() {
                        info!("execute clicked");
                        self.renamer.execute(&self.rules);
                    }
                });
            });

        TopBottomPanel::bottom("log_panel")
            .resizable(true)
            .default_height(200.0)
            .min_height(200.0)
            .show(ctx, |ui| {
                ui.heading("Logs");
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    let default_color = ui.visuals().text_color();
                    for line in self.log_writer.logs().iter().rev() {
                        let job = ansi_to_job(line, default_color);
                        ui.label(job);
                    }
                });
            });
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Regex GUI",
        native_options,
        Box::new(|_cc| Ok(Box::new(RegexApp::new()))),
    )
}

#[cfg(target_arch = "wasm32")]
fn main() {
    // Redirect log messages to the browser console and get readable panic traces
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();
    console_error_panic_hook::set_once();

    spawn_local(async {
        // Ensure we have a <canvas id="the_canvas_id">; create one if missing
        let window = eframe::web_sys::window().expect("no window");
        let document = window.document().expect("no document");
        let canvas: HtmlCanvasElement = document
            .get_element_by_id("the_canvas_id")
            .or_else(|| {
                // Dynamically create and attach a canvas
                let canvas = document.create_element("canvas").ok()?;
                canvas.set_id("the_canvas_id");
                document.body().unwrap().append_child(&canvas).ok()?;
                Some(canvas)
            })
            .expect("could not obtain or create canvas")
            .dyn_into()
            .expect("element is not a canvas");

        eframe::WebRunner::new()
            .start(
                canvas,
                eframe::WebOptions::default(),
                Box::new(|_cc| Ok(Box::new(RegexApp::new()))),
            )
            .await
            .expect("failed to start eframe");
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_rule_appends_default_rule() {
        let mut app = RegexApp::new_for_tests();
        let initial_len = app.rules.len();
        app.add_rule();
        assert_eq!(app.rules.len(), initial_len + 1);
        let rule = app.rules.last().unwrap();
        assert!(rule.from.is_empty() && rule.to.is_empty());
    }

    #[test]
    fn remove_rule_deletes_correct_index() {
        let mut app = RegexApp::new_for_tests();
        app.add_rule();
        let second = app.rules[1].from.clone();
        app.remove_rule(0);
        assert_eq!(app.rules.len(), 1);
        assert_eq!(app.rules[0].from, second);
    }
}
