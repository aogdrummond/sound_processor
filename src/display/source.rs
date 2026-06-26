use std::sync::mpsc;
use std::time::Instant;

pub struct AudioFrame {
    pub timestamp: Instant,
    pub samples: Vec<f32>,
}

pub trait DisplaySource {
    fn display_results(&mut self, rx_bands: mpsc::Receiver<AudioFrame>);
}