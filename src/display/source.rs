use std::sync::mpsc;
use std::time::Instant;
use super::super::audio::source::AudioFrame;


pub trait DisplaySource {
    fn display_results(&mut self, rx_bands: mpsc::Receiver<AudioFrame>);
}