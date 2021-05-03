#[macro_use]
extern crate glium;

use anyhow::Result;
use crossbeam_channel;
use glium::glutin;
use jack;
use std::thread;
use std::time::Duration;
mod lib;
use lib::controllers::{FFTProc, PlaybackSystem, PortConnector, SineGen, FFT_MAX_SIZE};
use lib::egui_app::App;
use lib::spectrograph::Spectrograph;

fn create_display(event_loop: &glutin::event_loop::EventLoop<()>) -> glium::Display {
    let window_builder = glutin::window::WindowBuilder::new()
        .with_resizable(true)
        .with_inner_size(glutin::dpi::LogicalSize {
            width: 800.0,
            height: 800.0,
        })
        .with_title("egui_glium example");

    let context_builder = glutin::ContextBuilder::new()
        .with_depth_buffer(0)
        .with_srgb(true)
        .with_stencil_buffer(0)
        .with_vsync(true);

    glium::Display::new(window_builder, context_builder, &event_loop).unwrap()
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

    let mut app = App {
        label: "THanks".to_owned(),
        value: 2.17,
        plot: Default::default(),
        rx,
    };

    let event_loop = glutin::event_loop::EventLoop::with_user_event();
    let display = create_display(&&event_loop);

    let mut egui = egui_glium::EguiGlium::new(&display);
    let mut spectograph = Spectrograph::new(&display, 300, 600);

    event_loop.run(move |event, _, control_flow| {
        let mut redraw = || {
            egui.begin_frame(&display);

            // TODO: some mechanism like frame.quit()
            let mut quit = false;

            app.update(egui.ctx());

            let (needs_repaint, shapes) = egui.end_frame(&display);

            *control_flow = if quit {
                glutin::event_loop::ControlFlow::Exit
            } else if needs_repaint {
                display.gl_window().window().request_redraw();
                glutin::event_loop::ControlFlow::Poll
            } else {
                glutin::event_loop::ControlFlow::Wait
            };

            {
                use glium::Surface as _;
                let mut target = display.draw();

                let clear_color = egui::Rgba::from_rgb(0.1, 0.3, 0.2);
                target.clear_color(
                    clear_color[0],
                    clear_color[1],
                    clear_color[2],
                    clear_color[3],
                );

                // draw things behind egui here

                egui.paint(&display, &mut target, shapes);

                // draw things on top of egui here
                spectograph.draw(&mut target);

                target.finish().unwrap();
            }
        };

        match event {
            // Platform-dependent event handlers to workaround a winit bug
            // See: https://github.com/rust-windowing/winit/issues/987
            // See: https://github.com/rust-windowing/winit/issues/1619
            glutin::event::Event::RedrawEventsCleared if cfg!(windows) => redraw(),
            glutin::event::Event::RedrawRequested(_) if !cfg!(windows) => redraw(),

            glutin::event::Event::WindowEvent { event, .. } => {
                egui.on_event(event, control_flow);
                display.gl_window().window().request_redraw(); // TODO: ask egui if the events warrants a repaint instead
            }

            _ => (),
        }
    });
}
