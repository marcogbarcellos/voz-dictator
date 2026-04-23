import { useState, useEffect, useCallback, useRef } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { LANGUAGES, type AppSettings } from "../lib/constants";
import {
  checkPermissions,
  requestMicrophonePermission,
  setAutoStart,
} from "../lib/tauri-commands";

interface OnboardingProps {
  onComplete: () => void;
  onUpdate: (updates: Partial<AppSettings>) => void;
}

type Step = "welcome" | "language" | "permissions" | "apikeys" | "ready";

const STEPS: Step[] = ["welcome", "language", "permissions", "apikeys", "ready"];

const TOP_LANGUAGES = LANGUAGES.filter((l) => l.code !== "auto");

interface PermissionStatus {
  microphone: boolean;
  accessibility: boolean;
}

export function Onboarding({ onComplete, onUpdate }: OnboardingProps) {
  const [step, setStep] = useState<Step>("welcome");
  const [selectedLangs, setSelectedLangs] = useState<Set<string>>(
    new Set(["pt", "en"])
  );
  const [permissions, setPermissions] = useState<PermissionStatus>({
    microphone: false,
    accessibility: false,
  });
  const [checkingPermissions, setCheckingPermissions] = useState(false);
  const [groqKey, setGroqKey] = useState("");
  const [anthropicKey, setAnthropicKey] = useState("");
  const [autoStartEnabled, setAutoStartEnabled] = useState(true);
  const idx = STEPS.indexOf(step);

  const toggleLang = (code: string) => {
    setSelectedLangs((prev) => {
      const next = new Set(prev);
      if (next.has(code)) {
        if (next.size > 1) next.delete(code);
      } else {
        next.add(code);
      }
      return next;
    });
  };

  const pollPermissions = useCallback(async () => {
    try {
      const result = await checkPermissions();
      setPermissions(result);
      return result;
    } catch {
      return { microphone: false, accessibility: false };
    }
  }, []); // Stable ref — no deps needed

  // Request permissions ONCE when entering the step
  const hasRequestedRef = useRef(false);
  useEffect(() => {
    if (step !== "permissions") {
      hasRequestedRef.current = false;
      return;
    }
    if (hasRequestedRef.current) return;
    hasRequestedRef.current = true;

    setCheckingPermissions(true);

    async function requestMic() {
      // Only auto-request microphone (shows a one-time native dialog)
      // Accessibility must be user-initiated — its dialog doesn't auto-dismiss
      try {
        await requestMicrophonePermission();
      } catch {
        // ignore
      }
      await pollPermissions();
      setCheckingPermissions(false);
    }
    requestMic();
  }, [step, pollPermissions]);

  // Poll separately to detect when user grants permissions
  useEffect(() => {
    if (step !== "permissions") return;

    const interval = setInterval(() => {
      pollPermissions();
    }, 1500);

    return () => clearInterval(interval);
  }, [step, pollPermissions]);

  const next = () => {
    // Save language when leaving the language step
    if (step === "language") {
      const langs = [...selectedLangs];
      const primary = selectedLangs.has("pt") ? "pt" : langs[0] || "auto";
      onUpdate({
        language: primary as AppSettings["language"],
        personalLanguages: langs,
      });
    }
    // Save API keys when leaving the apikeys step
    if (step === "apikeys") {
      const updates: Partial<AppSettings> = {};
      if (groqKey.trim()) updates.groqApiKey = groqKey.trim();
      if (anthropicKey.trim()) updates.anthropicApiKey = anthropicKey.trim();
      onUpdate(updates);
    }
    if (idx < STEPS.length - 1) {
      setStep(STEPS[idx + 1]);
    }
  };

  return (
    <div className="h-full flex flex-col items-center justify-center bg-bg-primary p-8">
      <AnimatePresence mode="wait">
        <motion.div
          key={step}
          initial={{ opacity: 0, x: 20 }}
          animate={{ opacity: 1, x: 0 }}
          exit={{ opacity: 0, x: -20 }}
          transition={{ duration: 0.2 }}
          className="w-full max-w-sm text-center space-y-6"
        >
          {step === "welcome" && (
            <>
              <motion.div
                initial={{ opacity: 0, scale: 0.8 }}
                animate={{ opacity: 1, scale: 1 }}
                transition={{ delay: 0.1, type: "spring", stiffness: 300, damping: 20 }}
              >
                <h1 className="text-3xl font-display text-text-primary">
                  Welcome to <span className="text-accent italic">Voz</span>
                </h1>
              </motion.div>
              <p className="text-text-secondary text-sm leading-relaxed">
                Voice dictation that understands you. Speak naturally in
                Portuguese, English, or 100+ languages — get clean, polished
                text instantly.
              </p>
              <button onClick={next} className="btn-primary">
                Get Started
              </button>
            </>
          )}

          {step === "language" && (
            <>
              <h2 className="text-2xl font-display text-text-primary">
                What languages do you speak?
              </h2>
              <p className="text-text-secondary text-sm">
                Select all that apply. Voz will auto-detect which language
                you're using each time you dictate.
              </p>
              <div className="grid grid-cols-2 gap-2 text-left">
                {TOP_LANGUAGES.map((lang) => (
                  <button
                    key={lang.code}
                    onClick={() => toggleLang(lang.code)}
                    className={`flex items-center gap-2 px-3 py-2.5 rounded-lg text-sm border transition-all ${
                      selectedLangs.has(lang.code)
                        ? "border-accent bg-accent-soft text-accent"
                        : "border-glass-border bg-bg-secondary text-text-secondary hover:border-text-muted"
                    }`}
                  >
                    <span>{lang.flag}</span>
                    <span className="truncate">{lang.label}</span>
                    {selectedLangs.has(lang.code) && (
                      <motion.svg
                        initial={{ scale: 0 }}
                        animate={{ scale: 1 }}
                        className="w-3.5 h-3.5 ml-auto flex-shrink-0"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        strokeWidth={3}
                      >
                        <path strokeLinecap="round" strokeLinejoin="round" d="M5 13l4 4L19 7" />
                      </motion.svg>
                    )}
                  </button>
                ))}
              </div>
              <button onClick={next} className="btn-primary">
                Continue
              </button>
            </>
          )}

          {step === "permissions" && (
            <>
              <h2 className="text-2xl font-display text-text-primary">
                Permissions
              </h2>
              <p className="text-text-secondary text-sm">
                Voz needs microphone access to hear you and accessibility access
                to type text into your apps.
              </p>
              <div className="space-y-3 text-left">
                <button
                  onClick={() => requestMicrophonePermission().then(() => pollPermissions())}
                  className={`flex items-center gap-3 p-3 rounded-lg border transition-colors w-full text-left ${
                    permissions.microphone
                      ? "bg-accent-soft border-accent/20"
                      : "bg-bg-secondary border-glass-border hover:border-text-muted"
                  }`}
                >
                  <span className="text-lg">🎤</span>
                  <div className="flex-1">
                    <div className="text-sm font-medium text-text-primary">
                      Microphone
                    </div>
                    <div className="text-xs text-text-muted">
                      {permissions.microphone ? "Granted" : "Tap to grant access"}
                    </div>
                  </div>
                  {permissions.microphone ? (
                    <motion.svg
                      initial={{ scale: 0 }}
                      animate={{ scale: 1 }}
                      transition={{ type: "spring", stiffness: 500, damping: 25 }}
                      className="w-5 h-5 text-success"
                      viewBox="0 0 24 24"
                      fill="none"
                      stroke="currentColor"
                      strokeWidth={2.5}
                    >
                      <path strokeLinecap="round" strokeLinejoin="round" d="M5 13l4 4L19 7" />
                    </motion.svg>
                  ) : (
                    <div className="w-5 h-5 rounded-full border-2 border-text-muted" />
                  )}
                </button>

                <div
                  className={`flex items-center gap-3 p-3 rounded-lg border transition-colors w-full text-left ${
                    permissions.accessibility
                      ? "bg-accent-soft border-accent/20"
                      : "bg-bg-secondary border-glass-border"
                  }`}
                >
                  <span className="text-lg">⌨️</span>
                  <div className="flex-1">
                    <div className="text-sm font-medium text-text-primary">
                      Accessibility
                    </div>
                    <div className="text-xs text-text-muted">
                      {permissions.accessibility
                        ? "Granted"
                        : "Will be requested on first use"}
                    </div>
                  </div>
                  {permissions.accessibility ? (
                    <motion.svg
                      initial={{ scale: 0 }}
                      animate={{ scale: 1 }}
                      transition={{ type: "spring", stiffness: 500, damping: 25 }}
                      className="w-5 h-5 text-success"
                      viewBox="0 0 24 24"
                      fill="none"
                      stroke="currentColor"
                      strokeWidth={2.5}
                    >
                      <path strokeLinecap="round" strokeLinejoin="round" d="M5 13l4 4L19 7" />
                    </motion.svg>
                  ) : (
                    <svg className="w-5 h-5 text-text-muted" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={1.5}>
                      <path strokeLinecap="round" strokeLinejoin="round" d="M13 16h-1v-4h-1m1-4h.01M12 2a10 10 0 100 20 10 10 0 000-20z" />
                    </svg>
                  )}
                </div>
              </div>

              <p className="text-xs text-text-muted leading-relaxed">
                {permissions.microphone
                  ? "Accessibility permission will be requested when you first dictate text. macOS will prompt you to allow it."
                  : "macOS will prompt you to grant permissions. If nothing happens, open System Settings → Privacy & Security."}
              </p>

              <button
                onClick={next}
                className="btn-primary"
              >
                Continue
              </button>
            </>
          )}

          {step === "apikeys" && (
            <>
              <h2 className="text-2xl font-display text-text-primary">
                API Keys
              </h2>
              <p className="text-text-secondary text-sm leading-relaxed">
                Voz works fully offline by default using a local Whisper model.
                Keys are optional — add them only if you want faster cloud
                transcription (Groq) or AI text cleanup (Claude).
              </p>
              <div className="space-y-4 text-left">
                <div>
                  <label className="block text-xs font-medium text-text-secondary mb-1.5">
                    Groq API Key
                    <span className="text-text-muted ml-1">(optional, for cloud STT)</span>
                  </label>
                  <input
                    type="password"
                    value={groqKey}
                    onChange={(e) => setGroqKey(e.target.value)}
                    placeholder="gsk_..."
                    className="w-full px-3 py-2 rounded-lg bg-bg-secondary border border-glass-border
                      text-text-primary text-sm placeholder:text-text-muted/50
                      focus:border-accent focus:outline-none transition-colors"
                  />
                  <p className="text-[11px] text-text-muted mt-1">
                    Get a free key at{" "}
                    <span className="text-text-secondary">console.groq.com</span>
                  </p>
                </div>
                <div>
                  <label className="block text-xs font-medium text-text-secondary mb-1.5">
                    Anthropic API Key
                    <span className="text-text-muted ml-1">(optional, for AI cleanup)</span>
                  </label>
                  <input
                    type="password"
                    value={anthropicKey}
                    onChange={(e) => setAnthropicKey(e.target.value)}
                    placeholder="sk-ant-..."
                    className="w-full px-3 py-2 rounded-lg bg-bg-secondary border border-glass-border
                      text-text-primary text-sm placeholder:text-text-muted/50
                      focus:border-accent focus:outline-none transition-colors"
                  />
                  <p className="text-[11px] text-text-muted mt-1">
                    For grammar & filler cleanup via Claude Haiku.
                  </p>
                </div>
              </div>
              <button
                onClick={next}
                className="btn-primary"
              >
                Continue
              </button>
            </>
          )}

          {step === "ready" && (
            <>
              <motion.div
                initial={{ scale: 0, rotate: -20 }}
                animate={{ scale: 1, rotate: 0 }}
                transition={{ type: "spring", stiffness: 400, damping: 15 }}
                className="text-5xl"
              >
                🎉
              </motion.div>
              <h2 className="text-2xl font-display text-text-primary">
                You're all set!
              </h2>
              <p className="text-text-secondary text-sm">
                Press{" "}
                <kbd className="px-1.5 py-0.5 rounded bg-bg-elevated text-text-primary text-xs font-mono">
                  ⌥ Space
                </kbd>{" "}
                anywhere to start dictating.
              </p>

              {/* Auto-start toggle */}
              <label className="flex items-center justify-between px-4 py-3 rounded-lg bg-bg-secondary border border-glass-border cursor-pointer group">
                <div className="text-left">
                  <span className="text-sm text-text-primary group-hover:text-accent transition-colors">
                    Open at Mac startup
                  </span>
                  <p className="text-[11px] text-text-muted mt-0.5">
                    Voz will launch automatically when your Mac starts
                  </p>
                </div>
                <button
                  role="switch"
                  aria-checked={autoStartEnabled}
                  onClick={() => setAutoStartEnabled((v) => !v)}
                  className={`relative w-9 h-5 rounded-full transition-colors flex-shrink-0 ml-3 ${
                    autoStartEnabled ? "bg-accent" : "bg-bg-elevated"
                  }`}
                >
                  <motion.div
                    animate={{ x: autoStartEnabled ? 16 : 2 }}
                    transition={{ type: "spring", stiffness: 500, damping: 30 }}
                    className="absolute top-0.5 w-4 h-4 rounded-full bg-white shadow"
                  />
                </button>
              </label>

              <button
                onClick={() => {
                  if (autoStartEnabled) {
                    // Fire-and-forget — the autostart plugin can stall on
                    // first-run on macOS; never block the onboarding exit.
                    setAutoStart(true).catch(() => {});
                  }
                  onComplete();
                }}
                className="btn-primary"
              >
                Start Using Voz
              </button>
            </>
          )}
        </motion.div>
      </AnimatePresence>

      {/* Progress dots */}
      <div className="flex gap-2 mt-8">
        {STEPS.map((s, i) => (
          <motion.div
            key={s}
            animate={{
              scale: i === idx ? 1.3 : 1,
              backgroundColor: i <= idx ? "#F59E0B" : "#44403C",
            }}
            transition={{ type: "spring", stiffness: 500, damping: 30 }}
            className="w-1.5 h-1.5 rounded-full"
          />
        ))}
      </div>

    </div>
  );
}
