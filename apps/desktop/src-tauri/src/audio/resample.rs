/// Resample audio from source sample rate to 16kHz mono using linear interpolation.
/// Simple and reliable — avoids FFT artifacts that cause Whisper to miss speech.
pub fn resample_to_16khz(samples: &[f32], source_rate: u32) -> Result<Vec<f32>, anyhow::Error> {
    if source_rate == 16000 {
        return Ok(samples.to_vec());
    }

    let ratio = source_rate as f64 / 16000.0;
    let output_len = (samples.len() as f64 / ratio).ceil() as usize;
    let mut output = Vec::with_capacity(output_len);

    for i in 0..output_len {
        let src_pos = i as f64 * ratio;
        let idx = src_pos as usize;
        let frac = (src_pos - idx as f64) as f32;

        let sample = if idx + 1 < samples.len() {
            samples[idx] * (1.0 - frac) + samples[idx + 1] * frac
        } else if idx < samples.len() {
            samples[idx]
        } else {
            0.0
        };

        output.push(sample);
    }

    Ok(output)
}
