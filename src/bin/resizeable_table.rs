#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;
use egui_extras::{TableBuilder, Column};

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(800.0, 600.0)),
        ..Default::default()
    };
    eframe::run_native(
        "Resizable Table Example",
        options,
        Box::new(|cc| Box::new(MyApp::new(cc))),
    )
}

#[derive(Default)]
struct MyApp {
    row_heights: Vec<f32>,
    data: Vec<Vec<String>>,
}

impl MyApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let num_rows = 5;
        let num_cols = 2;

        let mut row_heights = Vec::new();
        for _ in 0..num_rows {
            row_heights.push(30.0); // Default row height
        }

        let mut data = Vec::new();
        for i in 0..num_rows {
            let mut row = Vec::new();
            for j in 0..num_cols {
                row.push(format!("Row {}, Col {}", i, j));
            }
            data.push(row);
        }

        MyApp {
            row_heights,
            data,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self,ui:egui::Ui, ctx: &egui::Context ) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let num_rows = self.data.len();
            let num_cols = self.data[0].len();

            TableBuilder::new(ui)
                .column(Column::auto().resizable(true))
                .column(Column::remainder())
                .header(20.0, |mut header| {
                    for col_idx in 0..num_cols {
                        header.col(|ui| {
                            ui.heading(format!("Column {}", col_idx));
                        });
                    }
                })
                .body(|mut body| {
                    for row_idx in 0..num_rows {
                        let height = &mut self.row_heights[row_idx];
                        body.row(*height, |mut row| {
                            for col_idx in 0..num_cols {
                                row.col(|ui| {
                                    ui.label(self.data[row_idx][col_idx].clone());
                                });
                            }
                        });
                        // Simple row height editor
                        ui.horizontal(|ui|{
                            ui.label(format!("Row {} Height:",row_idx));
                            ui.add(egui::Slider::new(height, 20.0..=100.0).suffix(" px"));
                        });
                    }
                });
        });
    }
}

