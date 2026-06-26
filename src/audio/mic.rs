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
        .input_devices()?
        .find(|d| {
            d.name()
                .map(|n| n.contains("USB") || n.contains("Device"))
                .unwrap_or(false)
        })
        .ok_or("No usable input device found")?;

    println!("Using device: {}", device.name()?);

    let mut configs = device.supported_input_configs()?;

    for cfg in configs.clone() {
        println!("{:?}", cfg);
    }

    let supported_config = device
        .supported_input_configs()?
        .next()
        .ok_or("No supported input config")?
        .with_max_sample_rate();

    let config: cpal::StreamConfig = supported_config.clone().into();
    let sample_format = supported_config.sample_format();

    println!("Sample format: {:?}", sample_format);
    println!("Sample Rate: {}", config.sample_rate.0);
    println!("Channels: {}", config.channels);

    let (tx, rx) = mpsc::channel::<f32>();

    let stream = match sample_format {
        cpal::SampleFormat::F32 => {
            let channels = config.channels as usize;

            device.build_input_stream(
                &config,
                move |data: &[f32], _| {
                    for frame in data.chunks_exact(channels) {
                        let sample = frame[0];
                        let _ = tx.send(sample);
                    }
                },
                move |err| {
                    eprintln!("Audio error: {}", err);
                },
                None,
            )?
        }
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