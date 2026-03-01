use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::SampleFormat;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

use super::resample::resample_to_16khz;

/// Unsafe wrapper to make the state usable across threads.
/// The AudioRecorder manages stream lifecycle on the main thread.
unsafe impl Send for AudioRecorder {}
unsafe impl Sync for AudioRecorder {}

pub struct AudioRecorder {
    buffer: Arc<Mutex<Vec<f32>>>,
    /// Audio level as atomic f32 bits — lock-free, safe to read from any thread
    pub level: Arc<AtomicU32>,
    /// Whether we're actively recording (capturing to buffer)
    is_recording: Arc<AtomicBool>,
    sample_rate: u32,
    /// Persistent audio stream — stays alive to keep Bluetooth HFP active
    _stream: Option<cpal::Stream>,
}

impl AudioRecorder {
    pub fn new() -> Self {
        let buffer = Arc::new(Mutex::new(Vec::new()));
        let level = Arc::new(AtomicU32::new(0));
        let is_recording = Arc::new(AtomicBool::new(false));

        // Set up persistent audio stream immediately
        let host = cpal::default_host();
        let (stream, sample_rate) = match host.default_input_device() {
            Some(device) => {
                let device_name = device.name().unwrap_or_else(|_| "unknown".to_string());
                match device.default_input_config() {
                    Ok(config) => {
                        let sr = config.sample_rate().0;
                        let channels = config.channels();
                        log::info!(
                            "Audio device: {}, {}Hz, {} ch, {:?}",
                            device_name, sr, channels, config.sample_format()
                        );

                        let buf = buffer.clone();
                        let lvl = level.clone();
                        let rec = is_recording.clone();

                        let stream = match config.sample_format() {
                            SampleFormat::F32 => device.build_input_stream(
                                &config.into(),
                                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                                    let rms = (data.iter().map(|s| s * s).sum::<f32>()
                                        / data.len() as f32)
                                        .sqrt();
                                    lvl.store(rms.min(1.0).to_bits(), Ordering::Relaxed);

                                    if rec.load(Ordering::Relaxed) {
                                        if let Ok(mut b) = buf.lock() {
                                            if channels > 1 {
                                                for chunk in data.chunks(channels as usize) {
                                                    let mono = chunk.iter().sum::<f32>()
                                                        / chunk.len() as f32;
                                                    b.push(mono);
                                                }
                                            } else {
                                                b.extend_from_slice(data);
                                            }
                                        }
                                    }
                                },
                                |err| log::error!("Audio stream error: {}", err),
                                None,
                            ),
                            SampleFormat::I16 => {
                                let buf2 = buffer.clone();
                                let lvl2 = level.clone();
                                let rec2 = is_recording.clone();
                                device.build_input_stream(
                                    &config.into(),
                                    move |data: &[i16], _: &cpal::InputCallbackInfo| {
                                        let float_data: Vec<f32> = data
                                            .iter()
                                            .map(|&s| s as f32 / i16::MAX as f32)
                                            .collect();
                                        let rms = (float_data
                                            .iter()
                                            .map(|s| s * s)
                                            .sum::<f32>()
                                            / float_data.len() as f32)
                                            .sqrt();
                                        lvl2.store(rms.min(1.0).to_bits(), Ordering::Relaxed);

                                        if rec2.load(Ordering::Relaxed) {
                                            if let Ok(mut b) = buf2.lock() {
                                                for chunk in
                                                    float_data.chunks(channels as usize)
                                                {
                                                    let mono = chunk.iter().sum::<f32>()
                                                        / chunk.len() as f32;
                                                    b.push(mono);
                                                }
                                            }
                                        }
                                    },
                                    |err| log::error!("Audio stream error: {}", err),
                                    None,
                                )
                            }
                            format => {
                                log::error!("Unsupported sample format: {:?}", format);
                                Err(cpal::BuildStreamError::StreamConfigNotSupported)
                            }
                        };

                        match stream {
                            Ok(s) => {
                                if let Err(e) = s.play() {
                                    log::error!("Failed to start audio stream: {}", e);
                                    (None, sr)
                                } else {
                                    log::info!("Persistent audio stream started");
                                    (Some(s), sr)
                                }
                            }
                            Err(e) => {
                                log::error!("Failed to build audio stream: {}", e);
                                (None, 24000)
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to get input config: {}", e);
                        (None, 24000)
                    }
                }
            }
            None => {
                log::error!("No input device found");
                (None, 24000)
            }
        };

        Self {
            buffer,
            level,
            is_recording,
            sample_rate,
            _stream: stream,
        }
    }

    pub fn start(&mut self, _language: &str) -> Result<(), anyhow::Error> {
        if self.is_recording.load(Ordering::Relaxed) {
            return Err(anyhow::anyhow!("Already recording"));
        }

        if self._stream.is_none() {
            return Err(anyhow::anyhow!("No audio stream available"));
        }

        // Clear buffer and start capturing
        self.buffer.lock().unwrap().clear();
        self.is_recording.store(true, Ordering::Relaxed);

        log::info!("Recording started: {}Hz", self.sample_rate);
        Ok(())
    }

    pub fn stop(&mut self) -> Result<Vec<u8>, anyhow::Error> {
        if !self.is_recording.load(Ordering::Relaxed) {
            return Err(anyhow::anyhow!("Not recording"));
        }

        // Stop capturing (stream stays alive)
        self.is_recording.store(false, Ordering::Relaxed);

        let samples = {
            let buf = self.buffer.lock().unwrap();
            buf.clone()
        };

        if samples.is_empty() {
            return Err(anyhow::anyhow!("No audio captured"));
        }

        let sr = self.sample_rate;

        let duration_secs = samples.len() as f64 / sr as f64;
        log::info!(
            "Recording stopped: {} samples, {}Hz, {:.1}s",
            samples.len(),
            sr,
            duration_secs
        );

        // Resample to 16kHz mono for Whisper
        let resampled = if sr != 16000 {
            resample_to_16khz(&samples, sr)?
        } else {
            samples
        };

        // Encode as WAV
        let wav_data = encode_wav(&resampled, 16000)?;

        Ok(wav_data)
    }

    /// Lock-free level read — safe to call from any thread without touching the recorder
    pub fn get_level(&self) -> f32 {
        f32::from_bits(self.level.load(Ordering::Relaxed))
    }
}

fn encode_wav(samples: &[f32], sample_rate: u32) -> Result<Vec<u8>, anyhow::Error> {
    let mut cursor = std::io::Cursor::new(Vec::new());

    let spec = hound::WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = hound::WavWriter::new(&mut cursor, spec)?;

    for &sample in samples {
        let clamped = sample.clamp(-1.0, 1.0);
        let int_sample = (clamped * i16::MAX as f32) as i16;
        writer.write_sample(int_sample)?;
    }

    writer.finalize()?;
    Ok(cursor.into_inner())
}
