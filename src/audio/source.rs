pub trait AudioSource {
    fn next_chunk(&mut self) -> Option<Vec<f32>>;
}