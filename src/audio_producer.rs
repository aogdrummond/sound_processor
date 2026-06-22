use hound::WavReader;

pub const CHUNK_SIZE:usize = 4096;
static WAV_FILE: &str = "./data/file_example_WAV_2MG.wav";

pub fn get_audio() -> Result<Vec<f32>, hound::Error> {
    let mut reader = WavReader::open(WAV_FILE)?;

    let samples: Vec<f32> = reader
        .samples::<i16>()
        .map(|s| s.map(|v| v as f32 / i16::MAX as f32))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(samples)
}

// pub trait AudioSource {
//     fn next_chunk(&mut self) -> Option<Vec<f32>>;
// }

// pub struct WavSource {
//     samples: Vec<f32>,
//     index: usize,
// }

// impl WavSource {
//     pub fn new(samples: Vec<f32>) -> Self {
//         Self { samples, index: 0 }
//     }
// }

// impl AudioSource for WavSource {
//     fn next_chunk(&mut self) -> Option<Vec<f32>> {

//         if self.samples.is_empty() {
//             return None;
//         }

//         let mut chunk = Vec::with_capacity(CHUNK_SIZE);

//         for _ in 0..CHUNK_SIZE {
//             chunk.push(self.samples[self.index]);

//             self.index += 1;

//             if self.index >= self.samples.len() {
//                 self.index = 0; // wrap around
//             }
//         }

//         Some(chunk)
//     }
// }