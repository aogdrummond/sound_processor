pub fn to_db_display(
    amplitude: f32
) -> f32 {

    let db = 20.0 * amplitude.max(1e-10).log10();
    db.clamp(-80.0, 0.0) + 80.0
}
// Low-pass filter for smoothing noisy signals
pub fn exponential_moving_average(
    current_value: f32,
    average: f32
) -> f32 {
    let ema: f32 = 0.8 * current_value + 0.2 * average;
    ema
}