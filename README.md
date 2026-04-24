# Voz

**System-wide voice dictation for Mac** — speak naturally in Brazilian Portuguese, English, or 100+ languages and get clean, polished text injected into any application.

Voz is a commercial competitor to [Wispr Flow](https://wisprflow.ai), built with a hybrid architecture (local Whisper + cloud APIs) and AI-powered text cleanup. The key differentiator is world-class **Brazilian Portuguese** transcription at an affordable price point.

---

## Architecture

```
┌──────────────────────────────────────────────┐
│                 Tauri 2 App                   │
│                                               │
│  ┌─────────────┐    ┌──────────────────────┐ │
│  │  React UI    │    │  Rust Backend         │ │
│  │  (Webview)   │◄──►│                       │ │
│  │              │    │  ├─ Audio Capture      │ │
│  │  - Float pill│    │  ├─ VAD               │ │
│  │  - Settings  │    │  ├─ Local STT (Whisper)│ │
│  │  - Tray menu │    │  ├─ Cloud STT (Groq/DG)│ │
│  │  - Onboarding│    │  ├─ LLM Cleanup        │ │
│  └─────────────┘    │  ├─ Text Injection     │ │
│                      │  └─ Global Hotkey      │ │
│                      └──────────────────────┘ │
└──────────────────────────────────────────────┘
```

**How it works:**

1. Press **Option+Space** anywhere on your Mac
2. Speak naturally — Voz captures audio via `cpal` and shows a floating pill indicator
3. Release the hotkey — audio is resampled to 16kHz mono WAV
4. The WAV is transcribed either **locally** (bundled Whisper large-v3-turbo, default — no API key required) or via **cloud** (Groq Whisper or Deepgram Nova-3)
5. If an Anthropic key is configured, raw transcription goes through **Claude Haiku 4.5** for cleanup (grammar, fillers, tone) — otherwise raw transcript is used
6. Final text is injected into the active app via clipboard paste simulation

---

## Tech Stack

| Layer | Technology | Purpose |
|-------|-----------|---------|
| **App shell** | [Tauri 2](https://tauri.app) | Native macOS app with Rust backend + webview frontend |
| **Frontend** | React 19 + TypeScript + Tailwind CSS v4 | Settings UI, floating pill, onboarding |
| **Animations** | Framer Motion 11 | Spring animations, transitions |
| **Audio** | `cpal` 0.15 | Microphone capture |
| **Resampling** | `rubato` 0.15 | Convert to 16kHz mono for Whisper |
| **WAV encoding** | `hound` 3.5 | Encode audio as WAV |
| **HTTP** | `reqwest` 0.12 | API calls to Groq, Deepgram, Anthropic |
| **Key simulation** | `enigo` 0.2 | Cmd+V paste simulation |
| **STT (primary)** | Groq Whisper large-v3 | Cloud speech-to-text (~$0.11/hr) |
| **STT (premium)** | Deepgram Nova-3 | Premium pt-BR accuracy (~$0.26/hr) |
| **AI cleanup** | Claude Haiku 4.5 | Grammar, filler removal, tone adaptation |
| **Landing page** | Astro 5 + Tailwind CSS v4 | Static marketing site |
| **Monorepo** | Turborepo + pnpm workspaces | Build orchestration |

---

## Project Structure

```
voz/
├── apps/
│   ├── desktop/                    # Tauri 2 desktop app
│   │   ├── src/                    # React frontend
│   │   │   ├── components/
│   │   │   │   ├── FloatingPill.tsx       # Glass pill recording indicator
│   │   │   │   ├── SettingsPanel.tsx      # Full settings UI
│   │   │   │   ├── LanguageSelector.tsx   # Language dropdown (pt-BR first)
│   │   │   │   ├── ModeToggle.tsx         # Cloud vs Local toggle
│   │   │   │   ├── Waveform.tsx           # Canvas audio visualizer
│   │   │   │   ├── HotkeyConfig.tsx       # Shortcut capture widget
│   │   │   │   └── Onboarding.tsx         # 5-step first-run flow
│   │   │   ├── hooks/
│   │   │   │   ├── useRecording.ts        # Recording state machine
│   │   │   │   ├── useTranscription.ts    # STT integration
│   │   │   │   └── useSettings.ts         # Persisted settings
│   │   │   ├── lib/
│   │   │   │   ├── tauri-commands.ts      # Typed Tauri IPC wrappers
│   │   │   │   └── constants.ts           # Types, defaults, languages
│   │   │   ├── App.tsx
│   │   │   └── main.tsx
│   │   ├── src-tauri/              # Rust backend
│   │   │   ├── src/
│   │   │   │   ├── lib.rs                 # Tauri commands & app setup
│   │   │   │   ├── main.rs                # Entry point
│   │   │   │   ├── audio/
│   │   │   │   │   ├── capture.rs         # cpal mic capture + WAV encoding
│   │   │   │   │   ├── vad.rs             # Voice Activity Detection
│   │   │   │   │   └── resample.rs        # 16kHz resampling via rubato
│   │   │   │   ├── stt/
│   │   │   │   │   ├── local.rs           # Bundled Whisper (whisper-rs, Metal)
│   │   │   │   │   ├── groq.rs            # Groq Whisper API client
│   │   │   │   │   ├── deepgram.rs        # Deepgram Nova-3 client
│   │   │   │   │   └── assemblyai.rs      # AssemblyAI client
│   │   │   │   ├── cleanup/
│   │   │   │   │   ├── llm.rs             # Claude Haiku API client
│   │   │   │   │   └── prompts.rs         # Context-aware system prompts
│   │   │   │   ├── injection/
│   │   │   │   │   ├── paste.rs           # Clipboard + Cmd+V simulation
│   │   │   │   │   └── accessibility.rs   # macOS active app detection
│   │   │   │   ├── hotkey.rs              # Global Alt+Space shortcut
│   │   │   │   ├── tray.rs                # Menu bar tray icon
│   │   │   │   ├── settings.rs            # JSON config persistence
│   │   │   │   └── dictionary.rs          # Personal dictionary
│   │   │   ├── Cargo.toml
│   │   │   ├── tauri.conf.json
│   │   │   ├── Info.plist                 # macOS permissions descriptions
│   │   │   └── Entitlements.plist         # macOS entitlements
│   │   ├── package.json
│   │   ├── tsconfig.json
│   │   └── vite.config.ts
│   └── web/                        # Astro landing page
│       ├── src/
│       │   ├── pages/index.astro          # Full marketing page
│       │   ├── layouts/Layout.astro
│       │   └── styles/global.css
│       └── package.json
├── packages/
│   └── shared/                     # Shared types & constants
├── turbo.json
├── pnpm-workspace.yaml
└── package.json
```

---

## Install (end users)

> Release binaries are published on the [Releases page](https://github.com/marcogbarcellos/voz-dictator/releases/latest). Builds are currently **unsigned** — expect a one-time OS warning on first launch.

### macOS

1. Download the `.dmg` matching your chip (`Voz_<version>_aarch64.dmg` for Apple Silicon, `Voz_<version>_x64.dmg` for Intel)
2. Open the DMG, drag **Voz** to Applications
3. First launch: right-click the app → **Open** → **Open** again on the "unidentified developer" dialog. One time only. (Double-clicking shows a "cannot be opened" message because the app isn't notarized — the right-click path bypasses this.)
4. Grant **Microphone** and **Accessibility** permissions when prompted

### Windows

1. Download `Voz_<version>_x64-setup.msi`
2. Run the installer — on the "Windows protected your PC" screen, click **More info** → **Run anyway** (SmartScreen warning, same unsigned-binary reason as above)
3. Launch Voz from the Start menu

> **Note on Windows builds:** macOS-specific paths (global hotkey, text injection via Cmd+V, tray behavior) currently have only minimal Windows stubs. The Windows build is experimental.

---

## Prerequisites (for building from source)

- **macOS** 10.15+ (Catalina or later) or Windows 10+
- **Node.js** >= 20
- **pnpm** >= 9
- **Rust** (stable, latest) — install via [rustup.rs](https://rustup.rs)
- **CMake** — required by `whisper-rs-sys` to compile the embedded whisper.cpp
  - macOS: `brew install cmake`
  - Windows: bundled with recent Visual Studio Build Tools, or install from [cmake.org](https://cmake.org)
- **Xcode Command Line Tools** (macOS only): `xcode-select --install`

> **First build caveat:** the build script downloads `ggml-large-v3-turbo-q8_0.bin` (~874 MB, SHA256-verified) from Hugging Face into `apps/desktop/src-tauri/resources/`, then compiles whisper.cpp from source (~5 minutes on Apple Silicon). Subsequent builds reuse the cached model and compiled artifacts.

---

## Local Setup

### 1. Clone and install dependencies

```bash
git clone <repo-url> voz
cd voz
pnpm install
```

### 2. Install the Tauri CLI

```bash
cargo install tauri-cli --version "^2"
```

### 3. Build Rust dependencies (first time: ~5–10 min, ~874 MB download)

```bash
cd apps/desktop/src-tauri
cargo build
cd ../../..
```

First build triggers the Whisper model download (verified against a pinned SHA256) and compiles whisper.cpp. Repeat builds are fast.

### 4. Configure API keys (optional)

By default Voz runs fully offline with the bundled Whisper model — **no API keys required**. Keys only matter if you want:

- Faster cloud transcription (Groq or Deepgram), or
- AI cleanup of raw transcripts (Claude)

Keys are entered through the **Settings UI** inside the app (tray icon → Settings) and persisted to a local JSON file. Alternatively you can hand-edit the settings file before first launch:

```bash
mkdir -p ~/Library/Application\ Support/voz
cat > ~/Library/Application\ Support/voz/settings.json << 'EOF'
{
  "language": "pt",
  "stt_mode": "local",
  "stt_provider": "local",
  "ai_cleanup": true,
  "remove_fillers": true,
  "fix_grammar": true,
  "adapt_tone": false,
  "groq_api_key": "",
  "deepgram_api_key": "",
  "anthropic_api_key": "",
  "local_model_path": "",
  "hotkey": "Alt+Space",
  "personal_dictionary": []
}
EOF
```

### 5. Run the desktop app in dev mode

```bash
cd apps/desktop
cargo tauri dev
```

This starts the Vite dev server (hot reload for the frontend) and compiles/launches the Tauri app.

### 6. Run the landing page (optional)

```bash
cd apps/web
pnpm dev
```

Opens at `http://localhost:4321`.

### 7. Build for production

```bash
# Desktop app (produces .dmg / .app)
cd apps/desktop
cargo tauri build

# Landing page (static HTML)
cd apps/web
pnpm build
```

---

## API Keys (all optional)

Voz works offline out of the box. API keys only enable faster cloud transcription and AI cleanup:

| Key | Where to get it | Enables | Pricing |
|-----|----------------|---------|---------|
| **Groq API Key** | [console.groq.com](https://console.groq.com) | Cloud transcription via Whisper large-v3 (faster than local on old hardware) | Free tier available, ~$0.11/hr after |
| **Deepgram API Key** | [console.deepgram.com](https://console.deepgram.com) | Premium pt-BR accuracy via Nova-3 | $200 free credit, ~$0.26/hr after |
| **Anthropic API Key** | [console.anthropic.com](https://console.anthropic.com) | AI text cleanup via Claude Haiku 4.5 (grammar, fillers, tone) | $0.80/1M input tokens, $4/1M output |

### Where to enter them

**Option A — App UI (recommended):**

1. Launch the app
2. Click the tray icon > **Settings**
3. Scroll to the **API Keys** section
4. Paste your keys and they're saved automatically

**Option B — Settings file:**

Edit `~/Library/Application Support/voz/settings.json` directly (see step 4 above).

### Notes

- **Default (no keys)**: local Whisper transcription, no cleanup — raw transcript as written
- **Adding an Anthropic key**: enables AI cleanup on top of whichever STT is configured
- **Adding a Groq/Deepgram key**: offers a cloud alternative; switch in Settings if you want to use it
- Keys are stored in plaintext in `~/Library/Application Support/voz/settings.json`. Treat this file as sensitive

---

## macOS Permissions

On first launch, macOS will prompt you for:

| Permission | Why | How to grant |
|-----------|-----|-------------|
| **Microphone** | Audio capture for transcription | System prompt on first recording attempt |
| **Accessibility** | Detecting the active app + simulating Cmd+V paste | System Settings > Privacy & Security > Accessibility > enable Voz |

If text injection isn't working, check that Voz is listed and enabled in **System Settings > Privacy & Security > Accessibility**.

---

## Design System — "Warm Obsidian"

The UI uses a warm dark-mode aesthetic with amber/gold accents:

| Token | Value | Usage |
|-------|-------|-------|
| `--bg-primary` | `#1C1917` | Main background |
| `--bg-secondary` | `#292524` | Cards, panels |
| `--accent` | `#F59E0B` | Buttons, highlights, waveform |
| `--text-primary` | `#FAFAF9` | Headings, body text |
| `--text-secondary` | `#A8A29E` | Secondary text |
| `--recording` | `#EF4444` | Recording indicator |
| `--success` | `#34D399` | Done state |

**Fonts:** [Instrument Serif](https://fonts.google.com/specimen/Instrument+Serif) (headings) + [DM Sans](https://fonts.google.com/specimen/DM+Sans) (body)

---

## Supported Languages

Portuguese (BR) and English are the primary languages with optimized filler removal. Auto-detect is available for mixed usage.

| Language | Code | Filler removal |
|----------|------|---------------|
| Portugues (BR) | `pt` | ne, tipo, assim, entao, ta, ai, ahn, sabe, entendeu |
| English | `en` | uh, um, like, you know, basically, I mean, sort of |
| Espanol | `es` | este, pues, o sea, bueno, digamos |
| + 97 more | `auto` | Generic (via LLM) |

Set your language in **Settings > Language** or use `auto` to let Whisper detect it per recording.

---

## How the Pipeline Works

```
Hotkey pressed (Alt+Space)
        │
        ▼
┌─────────────────┐
│  cpal capture    │  Microphone → f32 samples → buffer
│  (48kHz stereo)  │
└────────┬────────┘
         │  Hotkey released
         ▼
┌─────────────────┐
│  rubato resample │  48kHz stereo → 16kHz mono
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  hound WAV       │  f32 samples → 16-bit PCM WAV bytes
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Groq Whisper    │  WAV → raw transcription text
│  (or Deepgram)   │  POST /audio/transcriptions
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Claude Haiku    │  raw text → cleaned text
│  (AI cleanup)    │  Removes fillers, fixes grammar,
│                   │  adapts tone to active app
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Text injection  │  pbcopy → Cmd+V → restore clipboard
│  (enigo + paste) │
└─────────────────┘
```

---

## Development Tips

- **Frontend hot reload**: The Vite dev server runs at `localhost:1420`. Tauri's webview connects to it in dev mode, so React changes are reflected instantly.
- **Rust changes**: Require recompilation. `cargo tauri dev` will rebuild automatically but it takes a few seconds.
- **Logs**: Run with `RUST_LOG=debug cargo tauri dev` to see detailed backend logs.
- **Settings location**: `~/Library/Application Support/voz/settings.json` — delete this file to reset to defaults.
- **Dictionary location**: `~/Library/Application Support/voz/dictionary.json`

---

## License

[MIT](LICENSE)
