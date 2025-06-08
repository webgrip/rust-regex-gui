use eframe::{App, Frame, egui};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use eframe::web_sys::HtmlCanvasElement;

mod application;
mod domain;
mod telemetry;

use application::Renamer;
use domain::Rule;
use std::sync::Arc;
use telemetry::{MemoryWriter, TracingLogger, init_tracing};
use tracing_subscriber::filter::LevelFilter;

struct RegexApp {
    dry_run: bool,
    rules: Vec<Rule>,
    renamer: Renamer,
    log_writer: MemoryWriter,
}

impl RegexApp {
    fn new() -> Self {
        let log_writer = init_tracing(LevelFilter::INFO);
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

            if ui.button("Execute").clicked() {
                self.renamer.execute(&self.rules);
            }

            ui.separator();
            ui.heading("Logs");
            egui::ScrollArea::vertical().show(ui, |ui| {
                for log in self.log_writer.logs() {
                    ui.label(log);
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
    // Redirect `log` messages to the browser console
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();
    let web_options = eframe::WebOptions::default();
    spawn_local(async {
        let canvas = eframe::web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id("the_canvas_id"))
            .and_then(|e| e.dyn_into::<HtmlCanvasElement>().ok())
            .expect("canvas not found");
        eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|_cc| Ok(Box::new(RegexApp::new()))),
            )
            .await
            .expect("failed to start eframe");
    });
}
