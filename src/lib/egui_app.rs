use eframe::egui::plot::{Curve, Plot};
use eframe::egui::*;
use eframe::{egui, epi};

pub struct TemplateApp {
    // Example stuff:
    pub label: String,
    pub value: f32,
    pub plot: PlotDemo,
    pub rx: crossbeam_channel::Receiver<Vec<f32>>,
}

impl epi::App for TemplateApp {
    fn name(&self) -> &str {
        "egui template"
    }

    /// Called ea4ch time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
        let TemplateApp {
            label,
            value,
            plot,
            rx,
        } = self;

        if let Ok(data) = rx.try_recv() {
            plot.rfft = data;
            //ui.ctx().request_repaint();
        }

        // Examples of how to create different panels and windows.
        // Pick whichever suits you.
        // Tip: a good default choice is to just keep the `CentralPanel`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::SidePanel::left("side_panel", 200.0).show(ctx, |ui| {
            ui.heading("Side Panel");

            ui.horizontal(|ui| {
                ui.label("Write something: ");
                ui.text_edit_singleline(label);
            });

            ui.add(egui::Slider::new(value, 0.0..=10.0).text("value"));
            if ui.button("Increment").clicked() {
                *value += 1.0;
            }

            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                ui.add(
                    egui::Hyperlink::new("https://github.com/emilk/egui/").text("powered by egui"),
                );
            });
        });

        egui::TopPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                egui::menu::menu(ui, "File", |ui| {
                    if ui.button("Quit").clicked() {
                        frame.quit();
                    }
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("egui template");
            ui.hyperlink("https://github.com/emilk/egui_template");
            ui.add(egui::github_link_file_line!(
                "https://github.com/emilk/egui_template/blob/master/",
                "Direct link to source code."
            ));
            egui::warn_if_debug_build(ui);

            ui.separator();

            egui::Frame::dark_canvas(ui.style()).show(ui, |ui| {
                plot.ui(ui);
            });
        });

        if false {
            egui::Window::new("Window").show(ctx, |ui| {
                ui.label("Windows can be moved by dragging them.");
                ui.label("They are automatically sized based on contents.");
                ui.label("You can turn on resizing and scrolling if you like.");
                ui.label("You would normally chose either panels OR windows.");
            });
        }
    }
}

// ----------------------------------------------------------------------------

#[derive(PartialEq)]
pub struct PlotDemo {
    rfft: Vec<f32>,
}

impl Default for PlotDemo {
    fn default() -> Self {
        Self { rfft: vec![] }
    }
}

impl PlotDemo {
    // fn options_ui(&mut self, ui: &mut Ui) {
    //     let Self { proportional, .. } = self;

    //     ui.horizontal(|ui| {
    //         ui.checkbox(proportional, "proportional data axes");
    //     });
    // }

    fn sin(&self) -> Curve {
        Curve::from_ys_f32(&self.rfft)
            .color(Color32::from_rgb(200, 100, 100))
            .name("0.5 * sin(2x) * sin(t)")
    }

    fn ui(&mut self, ui: &mut Ui) {
        // self.options_ui(ui);

        let plot = Plot::new("Demo Plot")
            .curve(self.sin())
            .min_size(Vec2::new(5000., 1.0))
            .data_aspect(self.rfft.len() as f32);
        ui.add(plot);
    }
}
