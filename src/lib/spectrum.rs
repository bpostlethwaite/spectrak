use super::common::Message;
use iced;
use iced::{
    canvas::{self, Cache, Canvas, Cursor, Geometry, LineCap, LineJoin, Path, Stroke},
    Color, Container, Element, Length, Point, Rectangle,
};

#[derive(Default)]
pub struct State {
    cache: Cache,
}

impl State {
    pub fn view(&mut self) -> Element<Message> {
        let canvas = Canvas::new(Spectrum {
            state: self,
            rfft: vec![],
        })
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

    pub fn request_redraw(&mut self) {
        self.cache.clear()
    }
}

pub struct Spectrum<'a> {
    state: &'a mut State,
    rfft: Vec<f32>,
}

impl<'a> canvas::Program<Message> for Spectrum<'a> {
    fn draw(&self, bounds: Rectangle, _cursor: Cursor) -> Vec<Geometry> {
        let fft = self.state.cache.draw(bounds.size(), |frame| {
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
