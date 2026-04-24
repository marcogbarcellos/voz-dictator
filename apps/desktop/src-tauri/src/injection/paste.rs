use std::process::Command;

/// Inject text into the active application via clipboard paste
pub fn inject(text: &str) -> Result<(), anyhow::Error> {
    // Save current clipboard content
    let original_clipboard = get_clipboard();

    // Set clipboard to our text
    set_clipboard(text)?;

    // Small delay to ensure clipboard is set
    std::thread::sleep(std::time::Duration::from_millis(50));

    // Simulate Cmd+V paste via AppleScript
    // (enigo crashes on non-main threads due to macOS TSM API requirements)
    let output = Command::new("osascript")
        .arg("-e")
        .arg(r#"tell application "System Events" to keystroke "v" using command down"#)
        .output()
        .map_err(|e| anyhow::anyhow!("Failed to simulate paste: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        log::warn!("Paste simulation warning: {}", stderr);
    }

    // Small delay then restore clipboard
    std::thread::sleep(std::time::Duration::from_millis(200));

    if let Some(original) = original_clipboard {
        let _ = set_clipboard(&original);
    }

    Ok(())
}

fn get_clipboard() -> Option<String> {
    Command::new("pbpaste")
        .env("LANG", "en_US.UTF-8")
        .env("LC_CTYPE", "en_US.UTF-8")
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout).ok()
            } else {
                None
            }
        })
}

/// Copy text to clipboard without pasting (public for smart_inject_text)
pub fn copy_to_clipboard(text: &str) -> Result<(), anyhow::Error> {
    set_clipboard(text)
}

fn set_clipboard(text: &str) -> Result<(), anyhow::Error> {
    use std::io::Write;
    // Force UTF-8: when launched from Finder, the .app inherits no LANG,
    // and pbcopy then falls back to the system legacy encoding (Mac Roman
    // on many setups), mangling accented characters on the way to the
    // pasteboard.
    let mut child = Command::new("pbcopy")
        .env("LANG", "en_US.UTF-8")
        .env("LC_CTYPE", "en_US.UTF-8")
        .stdin(std::process::Stdio::piped())
        .spawn()?;

    if let Some(ref mut stdin) = child.stdin {
        stdin.write_all(text.as_bytes())?;
    }

    child.wait()?;
    Ok(())
}
