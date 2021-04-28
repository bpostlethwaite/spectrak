use anyhow::Result;
use crossbeam_channel;
use iced;
use iced::{
    canvas::{self, Cache, Canvas, Cursor, Geometry, LineCap, LineJoin, Path, Stroke},
    executor, Application, Clipboard, Color, Command, Container, Element, Length, Point, Rectangle,
    Settings, Subscription,
};
use iced_futures::futures;
use jack;
use std::hash::{Hash, Hasher};
use std::thread;
use std::time::Duration;
mod audio;

use audio::controllers::{FFTProc, PlaybackSystem, PortConnector, SineGen, DEFAULT_FFT_SIZE};

#[allow(dead_code)]
enum EngMsg {
    Pause,
    Stop,
    Start,
}

fn main() -> Result<()> {
    let fft_size = 2usize.pow(DEFAULT_FFT_SIZE) * 32;
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

    Spectrum::run(Settings {
        antialiasing: true,
        ..Settings::with_flags(rx)
    })?;

    // sine_gen.jack_client.deactivate().unwrap();
    // fft_proc.jack_client.deactivate().unwrap();

    return Ok(());
}

struct Notifications;

impl jack::NotificationHandler for Notifications {
    fn thread_init(&self, _: &jack::Client) {
        println!("JACK: thread init");
    }

    fn shutdown(&mut self, status: jack::ClientStatus, reason: &str) {
        println!(
            "JACK: shutdown with status {:?} because \"{}\"",
            status, reason
        );
    }

    fn freewheel(&mut self, _: &jack::Client, is_enabled: bool) {
        println!(
            "JACK: freewheel mode is {}",
            if is_enabled { "on" } else { "off" }
        );
    }

    fn sample_rate(&mut self, _: &jack::Client, srate: jack::Frames) -> jack::Control {
        println!("JACK: sample rate changed to {}", srate);
        jack::Control::Continue
    }

    fn client_registration(&mut self, _: &jack::Client, name: &str, is_reg: bool) {
        println!(
            "JACK: {} client with name \"{}\"",
            if is_reg { "registered" } else { "unregistered" },
            name
        );
    }

    fn port_registration(&mut self, _: &jack::Client, port_id: jack::PortId, is_reg: bool) {
        println!(
            "JACK: {} port with id {}",
            if is_reg { "registered" } else { "unregistered" },
            port_id
        );
    }

    fn port_rename(
        &mut self,
        _: &jack::Client,
        port_id: jack::PortId,
        old_name: &str,
        new_name: &str,
    ) -> jack::Control {
        println!(
            "JACK: port with id {} renamed from {} to {}",
            port_id, old_name, new_name
        );
        jack::Control::Continue
    }

    fn ports_connected(
        &mut self,
        _: &jack::Client,
        port_id_a: jack::PortId,
        port_id_b: jack::PortId,
        are_connected: bool,
    ) {
        println!(
            "JACK: ports with id {} and {} are {}",
            port_id_a,
            port_id_b,
            if are_connected {
                "connected"
            } else {
                "disconnected"
            }
        );
    }

    fn graph_reorder(&mut self, _: &jack::Client) -> jack::Control {
        println!("JACK: graph reordered");
        jack::Control::Continue
    }

    fn xrun(&mut self, _: &jack::Client) -> jack::Control {
        println!("JACK: xrun occurred");
        jack::Control::Continue
    }

    fn latency(&mut self, _: &jack::Client, mode: jack::LatencyType) {
        println!(
            "JACK: {} latency has changed",
            match mode {
                jack::LatencyType::Capture => "capture",
                jack::LatencyType::Playback => "playback",
            }
        );
    }
}

struct Spectrum {
    rfft: Vec<f32>,
    spectrum: Cache,
    rx: crossbeam_channel::Receiver<Vec<f32>>,
}

#[derive(Debug, Clone)]
enum Message {
    Tick(Vec<f32>),
}

impl Application for Spectrum {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = crossbeam_channel::Receiver<Vec<f32>>;

    fn new(rx: crossbeam_channel::Receiver<Vec<f32>>) -> (Self, Command<Message>) {
        (
            Spectrum {
                rfft: vec![],
                spectrum: Default::default(),
                rx,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Spectrum - Iced")
    }

    fn update(&mut self, message: Message, _clipboard: &mut Clipboard) -> Command<Message> {
        match message {
            Message::Tick(rfft_data) => {
                self.rfft = rfft_data;
                self.spectrum.clear();
            }
        }

        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::from_recipe(CrossbeamReceiver {
            rx: self.rx.clone(),
        })
        .map(|data| Message::Tick(data))
    }

    fn view(&mut self) -> Element<Message> {
        let canvas = Canvas::new(self)
            .width(Length::Units(800))
            .height(Length::Units(400));

        Container::new(canvas)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .center_x()
            .center_y()
            .into()
    }
}

impl canvas::Program<Message> for Spectrum {
    fn draw(&self, bounds: Rectangle, _cursor: Cursor) -> Vec<Geometry> {
        let fft = self.spectrum.draw(bounds.size(), |frame| {
            let width = frame.width();
            let height = frame.height();

            let ndata = self.rfft.len();
            let pixel_spacing = width / (ndata as f32);
            let mut xn = 0.0;

            let curves = Path::new(|p| {
                for y in self.rfft.iter() {
                    p.line_to(Point {
                        x: xn,
                        y: height - y * height,
                    });
                    xn += pixel_spacing;
                    //print!("{} ", val);
                }
            });

            frame.stroke(
                &curves,
                Stroke {
                    color: Color::from_rgb(0.0, 0.0, 139.0),
                    width: 2.0,
                    line_cap: LineCap::Round,
                    line_join: LineJoin::Round,
                },
            );
        });

        vec![fft]
    }
}

pub struct CrossbeamReceiver {
    pub rx: crossbeam_channel::Receiver<Vec<f32>>,
}

impl<H, I> iced_native::subscription::Recipe<H, I> for CrossbeamReceiver
where
    H: Hasher,
{
    type Output = Vec<f32>;

    fn hash(&self, state: &mut H) {
        struct Marker;
        std::any::TypeId::of::<Marker>().hash(state);
    }

    fn stream(
        self: Box<Self>,
        _input: futures::stream::BoxStream<'static, I>,
    ) -> futures::stream::BoxStream<'static, Self::Output> {
        Box::pin(futures::stream::unfold(self.rx, move |state| async {
            let receiver = &state;
            let result = receiver.recv();
            if result.is_ok() {
                Some((result.unwrap(), state))
            } else {
                None
            }
        }))
    }
}
