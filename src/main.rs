mod audio;
mod audio_processing;
use audio_processing::Processor;
mod display;

use std::sync::mpsc;
use std::thread;
use std::time::Duration;

const SAMPLE_RATE: usize = 48000;

fn produce_audio<S>(
    mut source: S,
    tx_chunk: mpsc::Sender<Vec<f32>>)
where
    S: audio::source::AudioSource,
{
    let chunk_time = Duration::from_secs_f64(audio::wav::CHUNK_SIZE as f64 / SAMPLE_RATE as f64);
    while let Some(chunk) = source.next_chunk() {

        if tx_chunk.send(chunk).is_err() {
            break;
        }

        // thread::sleep(chunk_time);
    }
}

fn process_audio(rx_chunk: mpsc::Receiver<Vec<f32>>,
                 tx_bands: mpsc::Sender<Vec<f32>>){
    let mut processor = Processor::new(audio::wav::CHUNK_SIZE);

    while let Ok(chunk) = rx_chunk.recv() {

        let bands = processor.process(&chunk);

        if tx_bands.send(bands).is_err() {
            break;
        }
    }

    println!("Processing finished");
}

fn display_results<S>(
    mut source: S,
    rx_bands: mpsc::Receiver<Vec<f32>>)
where
    S: display::source::DisplaySource,
{
    source.display_results(rx_bands);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {

    // Channel: Audio -> DSP
    let (tx_chunk, rx_chunk) = mpsc::channel::<Vec<f32>>();

    // Channel: DSP -> Display
    let (tx_bands, rx_bands) = mpsc::channel::<Vec<f32>>();

    println!("Starting...");   
    let audio_source = audio::mic::MicrophoneSource::new()?;
    println!("Microphone initialized");
  
    // let audio_source = audio::wav::WavSource::new()?;
    let display_source = display::terminal::TerminalDisplay::new()?;
    
    let producer_thread = thread::spawn(move || produce_audio(audio_source, tx_chunk));
    let processing_thread = thread::spawn(move || process_audio(rx_chunk, tx_bands));
    let display_thread = thread::spawn(move || display_results(display_source,rx_bands));

    producer_thread.join().unwrap();
    processing_thread.join().unwrap();
    display_thread.join().unwrap();

    Ok(())
}