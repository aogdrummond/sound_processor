mod audio;
mod audio_processing;
use audio_processing::Processor;
mod display;
use audio::source::AudioFrame;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use std::time::Instant;

const SAMPLE_RATE: usize = 48000;

fn produce_audio<S>(
    mut source: S,
    tx_chunk: mpsc::Sender<AudioFrame>)
where
    S: audio::source::AudioSource,
{
    let chunk_time = Duration::from_secs_f64(audio::wav::CHUNK_SIZE as f64 / SAMPLE_RATE as f64);
    while let Some(chunk) = source.next_chunk() {
        // Include here the timestamp
        let frame = AudioFrame{timestamp: Instant::now(),
                               samples: chunk};
        if tx_chunk.send(frame).is_err() {
            break;
        }
    }
}

fn process_audio(rx_chunk: mpsc::Receiver<AudioFrame>,
                 tx_bands: mpsc::Sender<AudioFrame>){
    let mut processor = Processor::new(audio::wav::CHUNK_SIZE);

    while let Ok(frame) = rx_chunk.recv() {
        println!("Latency Processing: {:.3} ms",
            frame.timestamp.elapsed().as_secs_f64() * 1000.0
        );
        let start = Instant::now();

        let bands = processor.process(&frame.samples);
        let elapsed = start.elapsed().as_secs_f64() * 1000.0;
        println!("Elapsed: {:?} ms", elapsed);
        let frame2 = AudioFrame{timestamp: Instant::now(),
                                    samples: bands};
        if tx_bands.send(frame2).is_err() {
            break;
        }
    }

    println!("Processing finished");
}

fn display_results<S>(
    mut source: S,
    rx_bands: mpsc::Receiver<AudioFrame>)
where
    S: display::source::DisplaySource,
{
    source.display_results(rx_bands);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {

    // Channel: Audio -> DSP
    let (tx_chunk, rx_chunk) = mpsc::channel::<AudioFrame>();

    // Channel: DSP -> Display
    let (tx_bands, rx_bands) = mpsc::channel::<AudioFrame>();

    println!("Starting...");   
    let audio_source = audio::mic::MicrophoneSource::new()?;
    println!("Microphone initialized");
  
    // let audio_source = audio::wav::WavSource::new()?;
    // let display_source = display::terminal::TerminalDisplay::new()?;
    // let display_source = display::terminal_bars::TerminalBars::new()?;
    let display_source = display::oled_bars::OledBars::new()?;
    let producer_thread = thread::spawn(move || produce_audio(audio_source, tx_chunk));
    let processing_thread = thread::spawn(move || process_audio(rx_chunk, tx_bands));
    let display_thread = thread::spawn(move || display_results(display_source,rx_bands));

    producer_thread.join().unwrap();
    processing_thread.join().unwrap();
    display_thread.join().unwrap();

    Ok(())
}