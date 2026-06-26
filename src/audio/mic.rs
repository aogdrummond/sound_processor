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

        println!("Using device: {}", device.name()?);

        // Pick a supported config (IMPORTANT FIX)
        let supported_config = device
            .supported_input_configs()?
            .find(|c| c.channels() == 1)
            .ok_or("No mono input config found")?
            .with_max_sample_rate();

        let config: cpal::StreamConfig = supported_config.clone().into();
        let sample_format = supported_config.sample_format();

        println!("Sample format: {:?}", sample_format);
        println!("Sample rate: {}", config.sample_rate.0);
        println!("Channels: {}", config.channels);

        let (tx, rx) = mpsc::channel::<f32>();

        let stream = match sample_format {
            cpal::SampleFormat::F32 => build_stream::<f32>(&device, &config, tx)?,
            cpal::SampleFormat::I16 => build_stream::<i16>(&device, &config, tx)?,
            cpal::SampleFormat::U16 => build_stream::<u16>(&device, &config, tx)?,
            _ => return Err("Unsupported sample format".into()),
        };

        stream.play()?;

        Ok(Self {
            rx,
            _stream: stream,
        })
    }
}

fn build_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    tx: mpsc::Sender<f32>,
) -> Result<cpal::Stream, Box<dyn std::error::Error>>
where
    T: cpal::Sample + cpal::SizedSample + Send + 'static,
{
    let channels = config.channels as usize;

    let stream = device.build_input_stream(
        config,
        move |data: &[T], _| {
            for frame in data.chunks_exact(channels) {
                let sample: f32 = cpal::Sample::to_f32(&frame[0]);
                let _ = tx.send(sample);
            }
        },
        |err| {
            eprintln!("Audio stream error: {}", err);
        },
        None,
    )?;

    Ok(stream)
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

// use std::sync::mpsc;
// use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

// use super::source::AudioSource;
// use super::wav::CHUNK_SIZE;

// pub struct MicrophoneSource {
//     rx: mpsc::Receiver<f32>,
//     _stream: cpal::Stream,
// }

// impl MicrophoneSource {

//     pub fn new() -> Result<Self, Box<dyn std::error::Error>> {

//         let host = cpal::default_host();

//         let device = host
//             .default_input_device()
//             .ok_or("No input device found")?;

//         // println!("Supported configs:");

//         // for cfg in device.supported_input_configs()? {
//         //     println!("{:?}", cfg);
//         // }

//         let config = device
//                         .supported_input_configs()?
//                         .next()
//                         .ok_or("No supported input config")?
//                         .with_max_sample_rate();
//         println!("Using device: {}", device.name()?);
//         println!("Sample format: {:?}", config.sample_format());
//         println!("Sample Rate: {}", config.sample_rate().0);
//         println!("Channels: {}", config.channels());

//         let (tx, rx) = mpsc::channel::<f32>();
//         let stream = match sample_format {
//             cpal::SampleFormat::F32 => build_stream::<f32>(&device, &config, tx)?,
//             cpal::SampleFormat::I16 => build_stream::<i16>(&device, &config, tx)?,
//             cpal::SampleFormat::U16 => build_stream::<u16>(&device, &config, tx)?,
// };
//     //     let stream = match config.sample_format() {

//     //         cpal::SampleFormat::F32 => {

//     //             let channels = config.channels() as usize;
//     //             device.build_input_stream(
//     //             &config.into(),
//     //             move |data: &[f32], _| {
//     //                 for frame in data.chunks_exact(channels) {
//     //                     let sample = frame[0];
//     //                     let _ = tx.send(sample);
//     //                 }
//     //             },
//     //             move |err| {
//     //                 eprintln!("Audio error: {}", err);
//     //             },
//     //             None,
//     //         )?
//     //         }
//     // // cpal::SampleFormat::I16 => { ... }

//     // // cpal::SampleFormat::U16 => { ... }
//     //         _ => return Err("Unsupported sample format".into()),
//     //     };

//         stream.play()?;

//         Ok(Self {
//             rx,
//             _stream: stream,
//         })
//     }
// }

// impl AudioSource for MicrophoneSource {

//     fn next_chunk(&mut self) -> Option<Vec<f32>> {

//         let mut chunk = Vec::with_capacity(CHUNK_SIZE);

//         for _ in 0..CHUNK_SIZE {

//             match self.rx.recv() {

//                 Ok(sample) => chunk.push(sample),

//                 Err(_) => return None,
//             }
//         }

//         Some(chunk)
//     }
// }