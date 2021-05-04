use egui::widgets::plot::{Curve, Plot};

#[derive(PartialEq)]
pub struct Spectrum {}

impl Default for Spectrum {
    fn default() -> Self {
        Self {}
    }
}

impl Spectrum {
    // fn options_ui(&mut self, ui: &mut Ui) {
    //     let Self { proportional, .. } = self;

    //     ui.horizontal(|ui| {
    //         ui.checkbox(proportional, "proportional data axes");
    //     });
    // }

    fn sin(&self, data: &Vec<f32>) -> Curve {
        Curve::from_ys_f32(&data)
            .color(egui::Color32::from_rgb(200, 100, 100))
            .name("0.5 * sin(2x) * sin(t)")
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, height: f32, data: &Vec<f32>) {
        // self.options_ui(ui);

        let plot = Plot::new("Demo Plot")
            .curve(self.sin(data))
            .allow_drag(false)
            .include_y(1.0)
            .include_y(0.0)
            .height(height);
        ui.add(plot);
    }
}
