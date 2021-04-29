#[derive(Debug, Clone)]
pub enum Message {
    Tick(Vec<f32>),
}
