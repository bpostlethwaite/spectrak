use super::common::Message;
use crossbeam_channel;
use iced;
use iced::{
    canvas::{self, Cache, Canvas, Cursor, Geometry, LineCap, LineJoin, Path, Stroke},
    executor, Application, Clipboard, Color, Command, Container, Element, Length, Point, Rectangle,
    Subscription,
};
use iced_futures::futures;
use std::hash::{Hash, Hasher};

pub struct Spectrak {
    rfft: Vec<f32>,
    spectrum: Cache,
    rx: crossbeam_channel::Receiver<Vec<f32>>,
}

impl Application for Spectrak {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = crossbeam_channel::Receiver<Vec<f32>>;

    fn new(rx: crossbeam_channel::Receiver<Vec<f32>>) -> (Self, Command<Message>) {
        (
            Spectrak {
                rfft: vec![],
                spectrum: Default::default(),
                rx,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Spectrak - Iced")
    }

    fn update(&mut self, message: Message, _clipboard: &mut Clipboard) -> Command<Message> {
        match message {
            Message::Tick(rfft_data) => {
                self.rfft = rfft_data;
                //self.spectrum.clear();
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

impl canvas::Program<Message> for Spectrak {
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
