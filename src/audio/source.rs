use std::time::Instant;

pub trait AudioSource {
    fn next_chunk(&mut self) -> Option<Vec<f32>>;
}

pub struct AudioFrame {
    pub timestamp: Instant,
    pub samples: Vec<f32>,
}