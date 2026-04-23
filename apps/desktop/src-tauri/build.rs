use sha2::{Digest, Sha256};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

const MODEL_FILENAME: &str = "ggml-large-v3-turbo-q8_0.bin";
const MODEL_URL: &str = "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-turbo-q8_0.bin";
// SHA256 of ggml-large-v3-turbo-q8_0.bin from Hugging Face LFS (ggerganov/whisper.cpp).
// Update together with MODEL_URL if the upstream file ever changes.
const MODEL_SHA256: &str = "317eb69c11673c9de1e1f0d459b253999804ec71ac4c23c17ecf5fbe24e259a1";

fn compute_sha256(path: &Path) -> std::io::Result<String> {
    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 64 * 1024];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn main() {
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let resources_dir = manifest_dir.join("resources");
    let model_path = resources_dir.join(MODEL_FILENAME);

    std::fs::create_dir_all(&resources_dir).expect("failed to create resources dir");

    let needs_download = match compute_sha256(&model_path) {
        Ok(h) if h == MODEL_SHA256 => false,
        Ok(h) => {
            println!(
                "cargo:warning=Whisper model at {} has unexpected hash ({}), re-downloading",
                model_path.display(),
                h
            );
            let _ = std::fs::remove_file(&model_path);
            true
        }
        Err(_) => true, // file missing or unreadable
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

        let downloaded_hash = compute_sha256(&tmp_path)
            .expect("failed to hash downloaded Whisper model");
        if downloaded_hash != MODEL_SHA256 {
            let _ = std::fs::remove_file(&tmp_path);
            panic!(
                "downloaded Whisper model hash {} != expected {}",
                downloaded_hash, MODEL_SHA256
            );
        }

        std::fs::rename(&tmp_path, &model_path)
            .expect("failed to move model into place");
    }

    tauri_build::build();
}
