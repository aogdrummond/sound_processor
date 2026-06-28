use super::source::DisplaySource;
use std::sync::mpsc;

use std::time::{Duration, Instant};

pub struct TerminalDisplay {}

impl TerminalDisplay{
    pub fn new() -> Result<Self, hound::Error> {     
        Ok(Self{})
    }
}

impl DisplaySource for TerminalDisplay {
    fn display_results(&mut self, rx_bands: mpsc::Receiver<Vec<f32>>){

        let mut last_update = Instant::now();
        let mut band_acc = vec![0.0f32; 8];
        let mut count = 0;

        while let Ok(bands) = rx_bands.recv() {

            for i in 0..bands.len() {
                band_acc[i] += bands[i];
            }

            count += 1;
            if last_update.elapsed() >= Duration::from_secs(1)
                && count > 0
            {
                last_update = Instant::now();

                let mut avg = vec![0.0; 8];

                for i in 0..8 {
                    avg[i] = to_db_display(
                            band_acc[i] / count as f32
                        );
                }

                print!("Bands: ");

                for (i, v) in avg.iter().enumerate() {
                    print!("B{}: {:.2}dB ; ", i + 1, v);
                }

                println!();
                band_acc.fill(0.0);
                count = 0;
            }
        }

        println!("Display finished");
    }

}

pub fn to_db_display(amplitude: f32) -> f32 {
    let db = 20.0 * amplitude.max(1e-10).log10();

    // clamp to a useful visual range
    let db = db.clamp(-80.0, 0.0);

    // shift to positive range (0–80)
    db + 80.0
    // db
}