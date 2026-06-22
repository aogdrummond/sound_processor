use std::sync::mpsc;

pub trait DisplaySource {
    fn display_results(&mut self, rx_bands: mpsc::Receiver<Vec<f32>>);
}