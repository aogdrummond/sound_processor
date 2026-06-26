use super::source::DisplaySource;

use std::sync::mpsc;
use std::time::{Duration, Instant};

use embedded_graphics::{pixelcolor::BinaryColor,prelude::*,
    primitives::{PrimitiveStyle,Rectangle}
};

use linux_embedded_hal::I2cdev;
use ssd1306::mode::BufferedGraphicsMode;
use ssd1306::{prelude::*,I2CDisplayInterface,Ssd1306};

const NUM_BANDS: usize = 8;
const UPDATE_INTERVAL_MS: u64 = 100;

pub struct OledBars {
    display: Ssd1306<
        I2CInterface<I2cdev>,
        DisplaySize128x64,
        BufferedGraphicsMode<DisplaySize128x64>,
    >,
}

use super::super::audio::source::AudioFrame;


impl OledBars {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {

        let i2c = I2cdev::new("/dev/i2c-1")?; //RPi I2C peripheral

        let interface = I2CDisplayInterface::new(i2c);

        let mut display = Ssd1306::new(
            interface,
            DisplaySize128x64,
            DisplayRotation::Rotate0,
        )
        .into_buffered_graphics_mode();

        display.init().unwrap();
        display.flush().unwrap();

        Ok(Self { display })
    }
}

impl DisplaySource for OledBars {

    fn display_results(
        &mut self,
        rx_bands: mpsc::Receiver<AudioFrame>
    ) {

        let mut last_update = Instant::now();
        let mut band_acc = vec![0.0f32; NUM_BANDS];
        let mut count = 0usize;
        let mut displayed = vec![0.0f32; NUM_BANDS];

        while let Ok(frame) = rx_bands.recv() {
            println!("Latency Display: {:.3} ms",
            frame.timestamp.elapsed().as_secs_f64() * 1000.0
        );
            //Consume from channel
            for i in 0..NUM_BANDS {
                band_acc[i] += frame.samples[i];
            }

            count += 1; //Every time count increases it means a new "chunk"

            // //time_to_upate
            // if last_update.elapsed() >= Duration::from_millis(UPDATE_INTERVAL_MS)
            //     && count > 0
            if self.is_time_to_update(last_update,count) {
                self.update_display(&mut band_acc,&mut count,&mut displayed);
                last_update = Instant::now();
            }
        }
    }
}

impl OledBars {

    fn is_time_to_update(&mut self, last_update: Instant, count:usize)-> bool{
        let is_time: bool = (last_update.elapsed() >= Duration::from_millis(UPDATE_INTERVAL_MS) && count > 0);        
        is_time
    }

    fn update_display(&mut self, band_acc: &mut Vec<f32>,
        count: &mut usize,
        displayed: &mut Vec<f32> ){
        // last_update = Instant::now();

        for i in 0..NUM_BANDS {
            let avg = to_db_display(band_acc[i] / *count as f32); // db of average
            displayed[i] = 0.8 * displayed[i] + 0.2 * avg; //This is an exponential moving average (EMA), a common low-pass filter for smoothing noisy signals.
        }

        self.draw_bars(&displayed);
        band_acc.fill(0.0);
        *count = 0;

    }

}

impl OledBars {

    fn draw_bars(&mut self,bands: &[f32]) {

        self.display.clear(BinaryColor::Off).unwrap();

        let width = 128;
        let height = 64;

        let bar_width = width / NUM_BANDS;

        for (i, value) in bands.iter().enumerate() {

            let normalized = (value / 40.0).clamp(0.0, 1.0);

            let bar_height = (normalized * height as f32) as i32;

            let x = (i * bar_width) as i32;

            let y = height as i32 - bar_height;

            Rectangle::new(Point::new(x, y),Size::new((bar_width - 2) as u32,
                    bar_height as u32,
                ),
            )
            .into_styled(PrimitiveStyle::with_fill(BinaryColor::On),
            ).draw(&mut self.display).unwrap();
        }

        self.display.flush().unwrap();
    }
}

fn to_db_display(
    amplitude: f32
) -> f32 {

    let db = 20.0 * amplitude.max(1e-10).log10();

    db.clamp(-80.0, 0.0) + 80.0
}