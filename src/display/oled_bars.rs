use super::source::DisplaySource;

use std::sync::mpsc;
use std::time::{Duration, Instant};

use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{
        PrimitiveStyle,
        Rectangle,
    },
};

use linux_embedded_hal::I2cdev;

use ssd1306::{
    prelude::*,
    I2CDisplayInterface,
    Ssd1306,
};

const NUM_BANDS: usize = 8;

pub struct OledBars {
    display: Ssd1306<
        I2CInterface<I2cdev>,
        DisplaySize128x64,
        BufferedGraphicsMode<DisplaySize128x64>,
    >,
}

impl OledBars {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {

        let i2c = I2cdev::new("/dev/i2c-1")?;

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
        rx_bands: mpsc::Receiver<Vec<f32>>
    ) {

        let mut last_update = Instant::now();

        let mut band_acc = vec![0.0f32; NUM_BANDS];
        let mut count = 0usize;

        let mut displayed = vec![0.0f32; NUM_BANDS];

        while let Ok(bands) = rx_bands.recv() {

            for i in 0..NUM_BANDS {
                band_acc[i] += bands[i];
            }

            count += 1;

            if last_update.elapsed() >= Duration::from_millis(100)
                && count > 0
            {
                last_update = Instant::now();

                for i in 0..NUM_BANDS {

                    let avg =
                        to_db_display(
                            band_acc[i] / count as f32
                        );

                    displayed[i] =
                        0.8 * displayed[i]
                        + 0.2 * avg;
                }

                self.draw_bars(&displayed);

                band_acc.fill(0.0);
                count = 0;
            }
        }
    }
}

impl OledBars {

    fn draw_bars(
        &mut self,
        bands: &[f32]
    ) {

        self.display
            .clear(BinaryColor::Off)
            .unwrap();

        let width = 128;
        let height = 64;

        let bar_width = width / NUM_BANDS;

        for (i, value) in bands.iter().enumerate() {

            let normalized =
                (value / 80.0)
                .clamp(0.0, 1.0);

            let bar_height =
                (normalized * height as f32)
                as i32;

            let x =
                (i * bar_width) as i32;

            let y =
                height as i32 - bar_height;

            Rectangle::new(
                Point::new(x, y),
                Size::new(
                    (bar_width - 2) as u32,
                    bar_height as u32,
                ),
            )
            .into_styled(
                PrimitiveStyle::with_fill(
                    BinaryColor::On
                ),
            )
            .draw(&mut self.display)
            .unwrap();
        }

        self.display.flush().unwrap();
    }
}

fn to_db_display(
    amplitude: f32
) -> f32 {

    let db =
        20.0 * amplitude.max(1e-10).log10();

    db.clamp(-80.0, 0.0) + 80.0
}