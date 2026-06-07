use eframe::egui;
use std::collections::HashSet;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([480.0, 360.0]),
        ..Default::default()
    };

    eframe::run_native("CineForm File Browser", options, Box::new(|_cc| Ok(Box::new(App::default()))))
}

struct App {
    files: Vec<String>,
    selected: HashSet<usize>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            files: vec!["example.raw".to_string(), "sample.mov".to_string()],
            selected: HashSet::new(),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::bottom("bottom_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let btn = egui::Button::new("Remove Selected");
                if self.selected.is_empty() {
                    ui.add_enabled(false, btn);
                } else if ui.add(btn).clicked() {
                    let mut indices: Vec<usize> = self.selected.iter().copied().collect();
                    indices.sort_unstable();
                    for &i in indices.iter().rev() {
                        self.files.remove(i);
                    }
                    self.selected.clear();
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Files");
                if ui.button("Pick Files").clicked() {
                    if let Some(paths) = rfd::FileDialog::new().pick_files() {
                        for path in paths {
                            self.files.push(path.display().to_string());
                        }
                    }
                }
            });

            ui.separator();

            egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                if self.files.is_empty() {
                    ui.label("(empty)");
                }
                for (i, file) in self.files.iter().enumerate() {
                    let checked = self.selected.contains(&i);
                    if ui.selectable_label(checked, file).clicked() {
                        if checked {
                            self.selected.remove(&i);
                        } else {
                            self.selected.insert(i);
                        }
                    }
                }
            });
        });
    }
}
