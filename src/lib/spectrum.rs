use egui::widgets::plot::{Curve, Plot};

#[derive(PartialEq)]
pub struct Spectrum {
    pub rfft: Vec<f32>,
}

impl Default for Spectrum {
    fn default() -> Self {
        Self { rfft: vec![] }
    }
}

impl Spectrum {
    // fn options_ui(&mut self, ui: &mut Ui) {
    //     let Self { proportional, .. } = self;

    //     ui.horizontal(|ui| {
    //         ui.checkbox(proportional, "proportional data axes");
    //     });
    // }

    fn sin(&self) -> Curve {
        Curve::from_ys_f32(&self.rfft)
            .color(egui::Color32::from_rgb(200, 100, 100))
            .name("0.5 * sin(2x) * sin(t)")
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        // self.options_ui(ui);

        let plot = Plot::new("Demo Plot")
            .curve(self.sin())
            .min_size(egui::Vec2::new(5000., 1.0))
            .data_aspect(self.rfft.len() as f32);
        ui.add(plot);
    }
}
