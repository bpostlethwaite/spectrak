use super::spectrum::Spectrum;

pub struct App {
    // Example stuff:
    pub label: String,
    pub value: f32,
    pub plot: Spectrum,
    pub rx: crossbeam_channel::Receiver<Vec<f32>>,
}

impl App {
    /// Called ea4ch time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    pub fn update(&mut self, ctx: &egui::CtxRef) {
        let App {
            label,
            value,
            plot,
            rx,
        } = self;

        if let Ok(data) = rx.try_recv() {
            plot.rfft = data;
            //ui.ctx().request_repaint();
        }

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
                        // TODO quit
                        //frame.quit();
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
