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
│  │  - Settings  │    │  ├─ Cloud STT (Groq)  │ │
│  │  - Tray menu │    │  ├─ Cloud STT (DG)    │ │
│  │  - Onboarding│    │  ├─ LLM Cleanup       │ │
│  └─────────────┘    │  ├─ Text Injection     │ │
│                      │  └─ Global Hotkey      │ │
│                      └──────────────────────┘ │
└──────────────────────────────────────────────┘
```

**How it works:**

1. Press **Option+Space** anywhere on your Mac
2. Speak naturally — Voz captures audio via `cpal` and shows a floating pill indicator
3. Release the hotkey — audio is resampled to 16kHz mono WAV
4. The WAV is sent to **Groq Whisper large-v3** (or Deepgram Nova-3) for transcription
5. Raw transcription goes through **Claude Haiku 4.5** for cleanup (grammar, fillers, tone)
6. Cleaned text is injected into the active app via clipboard paste simulation

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
│   │   │   │   │   ├── groq.rs            # Groq Whisper API client
│   │   │   │   │   └── deepgram.rs        # Deepgram Nova-3 client
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

## Prerequisites

- **macOS** 10.15+ (Catalina or later)
- **Node.js** >= 20
- **pnpm** >= 9
- **Rust** (stable, latest) — install via [rustup.rs](https://rustup.rs)
- **Tauri CLI** v2

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

### 3. Build Rust dependencies (first time takes a few minutes)

```bash
cd apps/desktop/src-tauri
cargo build
cd ../../..
```

### 4. Configure API keys

API keys are entered through the **Settings UI** inside the app (accessible from the tray icon or during onboarding). They are persisted to a local JSON file.

Alternatively, you can create the settings file manually before first launch:

```bash
mkdir -p ~/.config/voz
cat > ~/.config/voz/settings.json << 'EOF'
{
  "language": "pt",
  "stt_mode": "cloud",
  "stt_provider": "groq",
  "ai_cleanup": true,
  "remove_fillers": true,
  "fix_grammar": true,
  "adapt_tone": false,
  "groq_api_key": "YOUR_GROQ_API_KEY",
  "deepgram_api_key": "",
  "anthropic_api_key": "YOUR_ANTHROPIC_API_KEY",
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

## API Keys

You need **2 required** API keys and **1 optional** key:

### Required

| Key | Where to get it | Used for | Pricing |
|-----|----------------|----------|---------|
| **Groq API Key** | [console.groq.com](https://console.groq.com) | Speech-to-text via Whisper large-v3 | Free tier available, ~$0.11/hr after |
| **Anthropic API Key** | [console.anthropic.com](https://console.anthropic.com) | AI text cleanup via Claude Haiku 4.5 | $0.80/1M input tokens, $4/1M output |

### Optional

| Key | Where to get it | Used for | Pricing |
|-----|----------------|----------|---------|
| **Deepgram API Key** | [console.deepgram.com](https://console.deepgram.com) | Premium pt-BR accuracy via Nova-3 | $200 free credit, ~$0.26/hr after |

### Where to enter them

**Option A — App UI (recommended):**

1. Launch the app
2. Click the tray icon > **Settings**
3. Scroll to the **API Keys** section
4. Paste your keys and they're saved automatically

**Option B — Settings file:**

Edit `~/.config/voz/settings.json` directly (see step 4 above).

### Notes

- If no **Anthropic key** is set, AI cleanup is skipped — you get raw transcription output
- If no **Groq key** is set, cloud transcription won't work (you'll see an error in the pill)
- **Deepgram** is only used if you switch the STT provider to `deepgram` in settings — it's not the default
- Keys are stored in plaintext in `~/.config/voz/settings.json`. Treat this file as sensitive

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
- **Settings location**: `~/.config/voz/settings.json` — delete this file to reset to defaults.
- **Dictionary location**: `~/.config/voz/dictionary.json`

---

## License

[MIT](LICENSE)
