use std::path::PathBuf;
use std::process::Command;

const MODEL_FILENAME: &str = "ggml-large-v3-turbo-q8_0.bin";
const MODEL_URL: &str = "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-turbo-q8_0.bin";
const MIN_EXPECTED_SIZE: u64 = 800 * 1024 * 1024;

fn main() {
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let resources_dir = manifest_dir.join("resources");
    let model_path = resources_dir.join(MODEL_FILENAME);

    std::fs::create_dir_all(&resources_dir).expect("failed to create resources dir");

    let needs_download = match std::fs::metadata(&model_path) {
        Ok(m) if m.len() >= MIN_EXPECTED_SIZE => false,
        Ok(_) => {
            println!(
                "cargo:warning=Whisper model at {} looks truncated, re-downloading",
                model_path.display()
            );
            let _ = std::fs::remove_file(&model_path);
            true
        }
        Err(_) => true,
    };

    if needs_download {
        println!("cargo:warning=Downloading Whisper GGML model (~874 MB, one-time). This may take a few minutes…");

        let tmp_path = model_path.with_extension("bin.part");
        let status = Command::new("curl")
            .args([
                "-L",
                "-f",
                "--progress-bar",
                "-o",
                tmp_path.to_str().unwrap(),
                MODEL_URL,
            ])
            .status()
            .expect("failed to invoke curl — ensure curl is installed");

        if !status.success() {
            let _ = std::fs::remove_file(&tmp_path);
            panic!("curl failed to download Whisper model from {}", MODEL_URL);
        }

        match std::fs::metadata(&tmp_path) {
            Ok(m) if m.len() >= MIN_EXPECTED_SIZE => {
                std::fs::rename(&tmp_path, &model_path)
                    .expect("failed to move model into place");
            }
            _ => {
                let _ = std::fs::remove_file(&tmp_path);
                panic!("downloaded model file is smaller than expected — aborting");
            }
        }
    }

    tauri_build::build();
}
