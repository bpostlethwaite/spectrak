use super::common::Message;
use super::spectrum;
use crossbeam_channel;
use iced;
use iced::{executor, Application, Clipboard, Command, Element, Subscription};
use iced_futures::futures;
use std::hash::{Hash, Hasher};

pub struct Spectrak {
    spectrum: spectrum::State,
    rx: crossbeam_channel::Receiver<Vec<f32>>,
}

impl Application for Spectrak {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = crossbeam_channel::Receiver<Vec<f32>>;

    fn new(rx: crossbeam_channel::Receiver<Vec<f32>>) -> (Self, Command<Message>) {
        (
            Spectrak {
                rx,
                spectrum: spectrum::State::new(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Spectrak - Iced")
    }

    fn update(&mut self, message: Message, _clipboard: &mut Clipboard) -> Command<Message> {
        match message {
            Message::Tick(rfft_data) => self.spectrum.set_rfft(rfft_data),
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
        self.spectrum.view()
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
