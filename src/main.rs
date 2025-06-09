use eframe::{egui, App, Frame};
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
use application::{Renamer, StdFileSystem};
use domain::Rule;
use std::sync::Arc;
use telemetry::{init_tracing, MemoryWriter, TracingLogger};
use theme::apply_catppuccin;
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
        let fs = Arc::new(StdFileSystem);
        let renamer = Renamer::new(logger, fs);
        Self {
            dry_run: false,
            rules: vec![Rule::default()],
            renamer,
            log_writer,
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
        apply_catppuccin(ctx);

        // --- main UI -------------------------------------------------------
        CentralPanel::default()
            .frame(egui::Frame::NONE.inner_margin(Margin::same(16)))
            .show(ctx, |ui| {
                // title ----------------------------------------------------
                ui.heading(RichText::new("🔧  Regex Renamer").size(24.0).strong());
                ui.add_space(4.0);

                // dry‑run toggle -----------------------------------------
                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.dry_run, "Dry‑run (preview only)");
                });

                ui.separator();

                // rules table ---------------------------------------------
                ui.heading("Rules");
                egui::Grid::new("rules_grid").striped(true).show(ui, |ui| {
                    ui.label(RichText::new("From Regex").strong());
                    ui.label(RichText::new("To Path").strong());
                    ui.label(RichText::new("Matches").strong());
                    ui.end_row();

                    for rule in &mut self.rules {
                        ui.text_edit_singleline(&mut rule.from);
                        ui.text_edit_singleline(&mut rule.to);
                        if ui.button("Count").clicked() {
                            let _ = self.renamer.count_matches(rule);
                        }
                        let label = rule
                            .file_match_count
                            .or(rule.dir_match_count)
                            .map(|c| c.to_string())
                            .unwrap_or_else(|| "-".into());
                        ui.label(label);
                        ui.end_row();
                    }
                });

                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button("➕  Add rule").clicked() {
                        self.rules.push(Rule::default());
                        info!("Added new rule");
                    }
                    if ui.button("🔍  Count all").clicked() {
                        info!("count all clicked");
                        let _ = self.renamer.count_all_matches(&mut self.rules);
                    }
                    if ui.button("▶  Execute").clicked() {
                        info!("execute clicked");
                        let _ = self.renamer.execute(&self.rules);
                    }
                });
            });

        // --- log panel -----------------------------------------------------
        TopBottomPanel::bottom("log_panel")
            .resizable(true)
            .min_height(200.0)
            .max_height(600.0)
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
