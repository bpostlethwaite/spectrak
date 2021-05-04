use glium::glutin;

pub const AUDIO_BUFF_SIZE: usize = 8192;
pub const FFT_MAX_SIZE: usize = 8192;
pub const FFT_MAX_BUFF_SIZE: usize = 4097;
pub const MAX_DATA_LENGTH: usize = 10000;
pub const APP_WIDTH: f32 = 1200.0;
pub const APP_HEIGHT: f32 = 800.0;

#[derive(Debug, Clone)]
pub enum Message {
    Tick(Vec<f32>),
}

pub struct RequestRepaintEvent;
pub struct GliumRepaintSignal(
    pub std::sync::Mutex<glutin::event_loop::EventLoopProxy<RequestRepaintEvent>>,
);

impl GliumRepaintSignal {
    pub fn request_repaint(&self) {
        self.0.lock().unwrap().send_event(RequestRepaintEvent).ok();
    }
}
