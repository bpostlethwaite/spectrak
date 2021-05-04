use super::common::*;
use super::spectrograph::Spectrograph;
use super::spectrum::Spectrum;
use anyhow::Result;
use std::sync::{Arc, RwLock};
use std::thread;

pub struct State {
    data: Vec<f32>,
    fft_size: usize,
}

impl State {
    pub fn new() -> State {
        State {
            data: Vec::with_capacity(MAX_DATA_LENGTH * FFT_MAX_BUFF_SIZE),
            fft_size: FFT_MAX_BUFF_SIZE,
        }
    }

    pub fn head_vec(&self) -> Vec<f32> {
        if (self.data.len() < self.fft_size) {
            return vec![];
        }
        let start = self.data.len() - self.fft_size;
        let slice = &self.data[start..];
        return slice.to_vec();
    }
}

pub struct App {
    // Example stuff:
    pub label: String,
    pub value: f32,
    pub plot: Spectrum,
    pub spectrograph: Spectrograph,
    pub last_head: usize,
}

impl App {
    pub fn listen(
        &self,
        state: Arc<RwLock<State>>,
        rx: crossbeam_channel::Receiver<Vec<f32>>,
        repaint_signal: Arc<GliumRepaintSignal>,
    ) {
        thread::spawn(move || loop {
            if let Ok(mut data) = rx.recv() {
                {
                    let mut lock = state.write().expect("mutex is poisoned");
                    lock.data.append(&mut data);
                }
                repaint_signal.request_repaint();
            }
        });
    }

    pub fn update(&mut self, state: Arc<RwLock<State>>, ctx: &egui::CtxRef) {
        let App {
            label,
            value,
            plot,
            spectrograph,
            ..
        } = self;

        let (data, current_head) = {
            let lock = state.read().expect("mutex poisoned");
            (lock.head_vec(), lock.data.len())
        };

        let app_rect = ctx.available_rect();

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::Frame::dark_canvas(ui.style()).show(ui, |ui| {
                let avail_size = ui.available_size();

                let plot_height = avail_size.y * 0.3;
                let spec_height = avail_size.y * 0.7;
                let (_, place_rect) = ui.allocate_space(egui::Vec2 {
                    x: avail_size.x,
                    y: spec_height,
                });
                plot.ui(ui, plot_height, &data);

                spectrograph.set_vertex_position(place_rect, app_rect);
            });
        });

        if self.last_head != current_head {
            spectrograph.update(data);
        }

        self.last_head = current_head;
    }

    pub fn draw(&mut self, target: &mut glium::Frame) {
        // draw things on top of egui here
        self.spectrograph.draw(target);
    }
}
