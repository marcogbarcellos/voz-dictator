/// Simple energy-based Voice Activity Detection
/// A proper implementation would use the Silero ONNX model,
/// but this provides a reasonable starting point.

const ENERGY_THRESHOLD: f32 = 0.01;
const MIN_SPEECH_FRAMES: usize = 10;
const MIN_SILENCE_FRAMES: usize = 30;

pub struct VoiceActivityDetector {
    speech_frames: usize,
    silence_frames: usize,
    is_speech: bool,
}

impl VoiceActivityDetector {
    pub fn new() -> Self {
        Self {
            speech_frames: 0,
            silence_frames: 0,
            is_speech: false,
        }
    }

    /// Process a frame of audio and return whether speech is detected
    pub fn process_frame(&mut self, samples: &[f32]) -> bool {
        let energy = (samples.iter().map(|s| s * s).sum::<f32>() / samples.len() as f32).sqrt();

        if energy > ENERGY_THRESHOLD {
            self.speech_frames += 1;
            self.silence_frames = 0;

            if self.speech_frames >= MIN_SPEECH_FRAMES {
                self.is_speech = true;
            }
        } else {
            self.silence_frames += 1;

            if self.silence_frames >= MIN_SILENCE_FRAMES && self.is_speech {
                self.is_speech = false;
                self.speech_frames = 0;
            }
        }

        self.is_speech
    }

    pub fn reset(&mut self) {
        self.speech_frames = 0;
        self.silence_frames = 0;
        self.is_speech = false;
    }
}
