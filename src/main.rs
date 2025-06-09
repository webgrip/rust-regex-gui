use eframe::{App, Frame, egui};
use egui::{CentralPanel, Margin, RichText};

#[cfg(target_arch = "wasm32")]
use console_error_panic_hook;
#[cfg(target_arch = "wasm32")]
use eframe::web_sys::HtmlCanvasElement;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local;

mod application;
mod domain;
mod telemetry;

use application::Renamer;
use domain::Rule;
use std::sync::Arc;
use telemetry::{MemoryWriter, TracingLogger, init_tracing};
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
}

impl App for RegexApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        // --- one‑off global style tweaks ----------------------------------
        ctx.set_style({
            let mut style = (*ctx.style()).clone();
            style.spacing.item_spacing = egui::vec2(10.0, 8.0);
            style
        });

        // --- main UI -------------------------------------------------------
        CentralPanel::default()
            .frame(egui::Frame::NONE.inner_margin(Margin::same(16)))
            .show(ctx, |ui| {
                // title ----------------------------------------------------
                ui.heading(RichText::new("🔧  Regex Renamer").size(24.0).strong());
                ui.add_space(4.0);
                ui.checkbox(&mut self.dry_run, "Dry‑run (no files are actually renamed)");
                ui.separator();

                // rules table ---------------------------------------------
                ui.heading("Rules");
                egui::Grid::new("rules_grid").striped(true).show(ui, |ui| {
                    ui.label(RichText::new("From Regex").strong());
                    ui.label(RichText::new("To Path").strong());
                    ui.end_row();

                    for rule in &mut self.rules {
                        ui.text_edit_singleline(&mut rule.from);
                        ui.text_edit_singleline(&mut rule.to);
                        ui.end_row();
                    }
                });

                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button("➕  Add rule").clicked() {
                        self.rules.push(Rule::default());
                        info!("Added new rule");
                    }
                    if ui.button("▶  Execute").clicked() {
                        self.renamer.execute(&self.rules);
                    }
                });

                // log view -------------------------------------------------
                ui.separator();
                ui.heading("Logs");
                egui::ScrollArea::vertical()
                    .max_height(200.0)
                    .show(ui, |ui| {
                        for line in self.log_writer.logs() {
                            ui.monospace(line);
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
