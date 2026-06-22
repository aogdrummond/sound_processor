use std::sync::mpsc;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use super::source::AudioSource;
use super::wav::CHUNK_SIZE;

pub struct MicrophoneSource {
    rx: mpsc::Receiver<f32>,
    _stream: cpal::Stream,
}

impl MicrophoneSource {

    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {

        let host = cpal::default_host();

        let device = host
            .default_input_device()
            .ok_or("No input device found")?;

        let config = device.default_input_config()?;
        println!("Using device: {}", device.name()?);
        println!("Sample format: {:?}", config.sample_format());
        println!("Sample Rate: {}", config.sample_rate().0);
        println!("Channels: {}", config.channels());

        let (tx, rx) = mpsc::channel::<f32>();

        let stream = match config.sample_format() {

            cpal::SampleFormat::F32 => {

                device.build_input_stream(
                    &config.into(),
                    move |data: &[f32], _| {

                        for sample in data {
                            let _ = tx.send(*sample);
                        }

                    },
                    move |err| {
                        eprintln!("Audio error: {}", err);
                    },
                    None,
                )?
            }
    // cpal::SampleFormat::I16 => { ... }

    // cpal::SampleFormat::U16 => { ... }
            _ => return Err("Unsupported sample format".into()),
        };

        stream.play()?;

        Ok(Self {
            rx,
            _stream: stream,
        })
    }
}

impl AudioSource for MicrophoneSource {

    fn next_chunk(&mut self) -> Option<Vec<f32>> {

        let mut chunk = Vec::with_capacity(CHUNK_SIZE);

        for _ in 0..CHUNK_SIZE {

            match self.rx.recv() {

                Ok(sample) => chunk.push(sample),

                Err(_) => return None,
            }
        }

        Some(chunk)
    }
}