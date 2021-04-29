use anyhow::Result;
use crossbeam_channel;
// use iced;
// use iced::{Application, Settings};
use jack;
use std::thread;
use std::time::Duration;
mod lib;
//use lib::app::Spectrak;
use lib::controllers::{FFTProc, PlaybackSystem, PortConnector, SineGen, FFT_MAX_SIZE};
use lib::egui_app;

#[allow(dead_code)]
enum EngMsg {
    Pause,
    Stop,
    Start,
}

fn main() -> Result<()> {
    let fft_size = FFT_MAX_SIZE;
    let (tx, rx) = crossbeam_channel::unbounded();

    let sine_gen = SineGen::new(
        "sine_gen",
        jack::AudioOut::default(),
        jack::AudioOut::default(),
        220.0,
        440.0,
    )?;

    let fft_proc = FFTProc::new(
        "fft_proc",
        jack::AudioIn::default(),
        jack::AudioIn::default(),
        fft_size,
        tx,
    )?;

    thread::sleep(Duration::from_secs(1));

    sine_gen.connect_to(&PlaybackSystem::new())?;
    sine_gen.connect_to(&fft_proc)?;

    let app = egui_app::TemplateApp {
        rx,
        label: "Heck ya".to_owned(),
        value: 2.7,
        plot: egui_app::PlotDemo::default(),
    };
    eframe::run_native(Box::new(app));

    // Spectrak::run(Settings {
    //     antialiasing: true,
    //     ..Settings::with_flags(rx)
    // })?;

    // sine_gen.jack_client.deactivate().unwrap();
    // fft_proc.jack_client.deactivate().unwrap();
}
