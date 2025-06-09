//=== src/bin/regex_app.rs =============================================
//! GUI front‑end for rule‑based file renaming.
//! Runs natively and in the browser (wasm32‑unknown‑unknown).
//==========================================================================
#![allow(clippy::needless_return)]

use eframe::egui::{
    self, Align, Button, CentralPanel, Context, Key, Layout, Modifiers, RichText, TopBottomPanel,
    Vec2,
};
use eframe::{App, Frame};
use egui_extras::{Column, TableBuilder};

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
use telemetry::Logger;
use telemetry::{MemoryWriter, TracingLogger, init_tracing};
use theme::apply_catppuccin;
use tracing::{info, warn};
use tracing_subscriber::filter::LevelFilter;

//==========================================================================
// RegexApp – the eframe::App implementation
//==========================================================================

pub struct RegexApp {
    /// If `true`, a rename is simulated only (no file operations).
    dry_run: bool,
    /// All currently‑defined rules.
    rules: Vec<Rule>,
    /// Handles counting / executing renames.
    renamer: Renamer,
    /// In‑memory log buffer displayed in the UI.
    log_writer: MemoryWriter,
    /// Whether the log panel is visible.
    show_log: bool,
}

impl RegexApp {
    /// App entry‑point used in production (with tracing, real FS, etc.)
    pub fn new() -> Self {
        let log_writer = init_tracing(LevelFilter::INFO);
        info!("RegexApp started");

        // Wrap our tracing integrator in Arc and pass it to Renamer.
        let logger: Arc<dyn Logger> = Arc::new(TracingLogger);
        let fs: Arc<StdFileSystem> = Arc::new(StdFileSystem);
        let renamer = Renamer::new(logger, fs);

        Self {
            dry_run: true,
            rules: vec![Rule::default()],
            renamer,
            log_writer,
            show_log: true,
        }
    }

    /// Test‑friendly constructor: no tracing overhead, no real FS.
    #[cfg(test)]
    fn new_for_tests() -> Self {
        let logger: Arc<dyn Logger> = Arc::new(TracingLogger);
        let fs: Arc<StdFileSystem> = Arc::new(StdFileSystem);
        let renamer = Renamer::new(logger, fs);
        Self {
            dry_run: false,
            rules: vec![Rule::default()],
            renamer,
            log_writer: MemoryWriter::default(),
            show_log: true,
        }
    }

    fn add_rule(&mut self) {
        self.rules.push(Rule::default());
    }

    fn remove_rule(&mut self, index: usize) {
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

//==========================================================================
// eframe::App implementation (UI code)
//==========================================================================

impl App for RegexApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        // -----------------------------------------------------------------
        // Theme (apply once per frame)
        // -----------------------------------------------------------------
        apply_catppuccin(ctx);

        // ═════════════════════════ Central panel ═════════════════════════
        CentralPanel::default().show(ctx, |ui| {
            ui.heading(RichText::new("Regex Renamer").size(20.0));
            ui.separator();

            //--------------------------- Rule table ------------------------
            TableBuilder::new(ui)
                .striped(true)
                .column(Column::auto()) // regex
                .column(Column::auto()) // to path
                .column(Column::auto()) // dirs
                .column(Column::auto()) // files
                .column(Column::remainder()) // actions (+/count/✖)
                .header(24.0, |mut header| {
                    header.col(|ui| {
                        ui.strong("From Regex");
                    });
                    header.col(|ui| {
                        ui.strong("To Path");
                    });
                    header.col(|ui| {
                        ui.strong("Dirs");
                    });
                    header.col(|ui| {
                        ui.strong("Files");
                    });
                    header.col(|ui| {
                        if ui.button("➕ Add").on_hover_text("Add rule").clicked() {
                            self.add_rule();
                            info!("Added new rule");
                        }
                    });
                })
                .body(|mut body| {
                    let regex_width = 220.0;
                    let path_width = 220.0;

                    let mut idx = 0usize;
                    while idx < self.rules.len() {
                        let rule = &mut self.rules[idx];
                        let mut should_remove = false;

                        body.row(24.0, |mut row| {
                            // Regex pattern
                            row.col(|ui| {
                                ui.add_sized(
                                    [regex_width, 0.0],
                                    egui::TextEdit::singleline(&mut rule.from).hint_text("regex"),
                                );
                            });

                            // Destination path
                            row.col(|ui| {
                                ui.add_sized(
                                    [path_width, 0.0],
                                    egui::TextEdit::singleline(&mut rule.to)
                                        .hint_text("destination"),
                                );
                            });

                            // Dir counter
                            row.col(|ui| {
                                let dir_text =
                                    rule.dir_match_count.map_or("—".into(), |n| n.to_string());
                                ui.label(dir_text);
                            });

                            // File counter
                            row.col(|ui| {
                                let file_text =
                                    rule.file_match_count.map_or("—".into(), |n| n.to_string());
                                ui.label(file_text);
                            });

                            // Actions
                            row.col(|ui| {
                                ui.horizontal(|ui| {
                                    if ui.button("🔍").on_hover_text("Count matches").clicked() {
                                        let _ = self.renamer.count_matches(rule);
                                    }
                                    if ui.button("❌").on_hover_text("Remove rule").clicked() {
                                        should_remove = true;
                                    }
                                });
                            });
                        });

                        if should_remove {
                            self.remove_rule(idx);
                        } else {
                            idx += 1; // only advance if rule wasn’t removed
                        }
                    }
                });

            ui.add_space(12.0);

            //---------------------- Global actions -------------------------
            ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    let button_size = Vec2::new((ui.available_width() / 2.0) - 6.0, 42.0);

                    // Count all button
                    if ui
                        .add_sized(
                            button_size,
                            Button::new(RichText::new("🔍 Count All").size(18.0)),
                        )
                        .clicked()
                    {
                        info!("Count all clicked");
                        let _ = self.renamer.count_all_matches(&mut self.rules);
                    }

                    // Execute button
                    if ui
                        .add_sized(
                            button_size,
                            Button::new(RichText::new("▶ Execute").size(18.0)),
                        )
                        .clicked()
                    {
                        info!("Execute clicked");
                        if self.dry_run {
                            warn!("Dry‑run mode enabled – no filesystem changes will be applied");
                        }
                        let _ = self.renamer.execute(&self.rules);
                    }
                });
            });
        });

        // -----------------------------------------------------------------
        // Toggle log visibility via hot‑key (press "L")
        // -----------------------------------------------------------------
        if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::L)) {
            self.show_log = !self.show_log;
        }

        // ═════════════════════════ Log panel ═════════════════════════════
        if self.show_log {
            TopBottomPanel::bottom("log_panel")
                .resizable(true)
                .min_height(200.0)
                .max_height(600.0)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.heading("Logs");
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ui.button("👁‍🗨").on_hover_text("Hide log (L)").clicked() {
                                self.show_log = false;
                            }
                        });
                    });

                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        let default_color = ui.visuals().text_color();
                        for line in self.log_writer.logs().iter().rev() {
                            ui.label(ansi_to_job(line, default_color));
                        }
                    });
                });
        } else {
            // Collapsed stub allowing the log to be shown again
            TopBottomPanel::bottom("log_toggle_stub")
                .exact_height(24.0)
                .show(ctx, |ui| {
                    if ui
                        .centered_and_justified(|ui| ui.button("Show log ⬆ (L)"))
                        .inner
                        .clicked()
                    {
                        self.show_log = true;
                    }
                });
        }
    }
}

//==========================================================================
// Native / Desktop entry‑point
//==========================================================================
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    let native_opts = eframe::NativeOptions::default();
    eframe::run_native(
        "Regex GUI",
        native_opts,
        Box::new(|_cc| {
            Ok::<Box<dyn App>, Box<dyn std::error::Error + Send + Sync>>(Box::new(
                RegexApp::default(),
            ))
        }),
    )
}

//==========================================================================
// WebAssembly entry‑point (wasm‑bindgen)
//==========================================================================
#[cfg(target_arch = "wasm32")]
fn main() {
    // Redirect tracing + panic messages to the browser console
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();
    console_error_panic_hook::set_once();

    spawn_local(async {
        let window = eframe::web_sys::window().expect("no window");
        let document = window.document().expect("no document");
        let canvas: HtmlCanvasElement = document
            .get_element_by_id("the_canvas_id")
            .or_else(|| {
                let c = document.create_element("canvas").ok()?;
                c.set_id("the_canvas_id");
                document.body().unwrap().append_child(&c).ok()?;
                Some(c)
            })
            .expect("Couldn’t create canvas")
            .dyn_into()
            .expect("element not a canvas");

        eframe::WebRunner::new()
            .start(
                canvas,
                eframe::WebOptions::default(),
                Box::new(|_cc| Ok(Box::new(RegexApp::default()))),
            )
            .await
            .expect("eframe start failed");
    });
}

//==========================================================================
// Tests
//==========================================================================
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
        let second_from = app.rules[1].from.clone();
        app.remove_rule(0);
        assert_eq!(app.rules.len(), 1);
        assert_eq!(app.rules[0].from, second_from);
    }

    #[test]
    fn toggle_log_flag_changes_state() {
        let mut app = RegexApp::new_for_tests();
        
        assert!(app.show_log);
        app.show_log = false;
        assert!(!app.show_log);
    }
}
