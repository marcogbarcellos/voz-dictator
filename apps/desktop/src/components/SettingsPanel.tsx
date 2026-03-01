import { motion } from "framer-motion";
import { LanguageSelector } from "./LanguageSelector";
import { ModeToggle } from "./ModeToggle";
import { HotkeyConfig } from "./HotkeyConfig";
import type { AppSettings, SttProvider } from "../lib/constants";
import { setAutoStart } from "../lib/tauri-commands";

interface SettingsPanelProps {
  settings: AppSettings;
  onUpdate: (updates: Partial<AppSettings>) => void;
  onClose: () => void;
}

function Toggle({
  label,
  value,
  onChange,
}: {
  label: string;
  value: boolean;
  onChange: (v: boolean) => void;
}) {
  return (
    <label className="flex items-center justify-between py-1.5 cursor-pointer group">
      <span className="text-sm text-text-primary group-hover:text-accent transition-colors">
        {label}
      </span>
      <button
        role="switch"
        aria-checked={value}
        onClick={() => onChange(!value)}
        className={`relative w-9 h-5 rounded-full transition-colors ${
          value ? "bg-accent" : "bg-bg-elevated"
        }`}
      >
        <motion.div
          animate={{ x: value ? 16 : 2 }}
          transition={{ type: "spring", stiffness: 500, damping: 30 }}
          className="absolute top-0.5 w-4 h-4 rounded-full bg-white shadow"
        />
      </button>
    </label>
  );
}

function Section({
  title,
  children,
}: {
  title: string;
  children: React.ReactNode;
}) {
  return (
    <div className="space-y-2">
      <h3 className="text-xs font-semibold uppercase tracking-wider text-text-muted">
        {title}
      </h3>
      {children}
    </div>
  );
}

const STT_PROVIDERS: { id: SttProvider; name: string; description: string }[] = [
  { id: "groq", name: "Groq", description: "Fast, affordable" },
  { id: "deepgram", name: "Deepgram", description: "High accuracy" },
  { id: "assemblyai", name: "AssemblyAI", description: "Universal-2" },
];

function ApiKeyInput({
  label,
  value,
  onChange,
  placeholder,
}: {
  label: string;
  value: string;
  onChange: (v: string) => void;
  placeholder: string;
}) {
  const hasKey = value.length > 0;
  return (
    <div>
      <label className="text-xs text-text-secondary mb-1 flex items-center justify-between">
        <span>{label}</span>
        {hasKey && (
          <span className="flex items-center gap-1 text-success">
            <span className="w-1.5 h-1.5 rounded-full bg-success" />
            Set
          </span>
        )}
      </label>
      <input
        type="password"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder={placeholder}
        className="w-full px-3 py-2 rounded-lg text-sm bg-bg-secondary border border-glass-border
          text-text-primary placeholder:text-text-muted focus:border-accent focus:outline-none transition-colors"
      />
    </div>
  );
}

export function SettingsPanel({ settings, onUpdate, onClose }: SettingsPanelProps) {
  return (
    <div className="h-full flex flex-col bg-bg-primary">
      {/* Header */}
      <div className="flex items-center justify-between px-5 pt-4 pb-3 border-b border-glass-border flex-shrink-0">
        <div className="flex items-center gap-2">
          <button
            onClick={onClose}
            className="w-7 h-7 flex items-center justify-center rounded-lg
              text-text-muted hover:text-text-primary hover:bg-bg-secondary transition-colors"
            aria-label="Back"
          >
            <svg className="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2}>
              <path strokeLinecap="round" strokeLinejoin="round" d="M15 19l-7-7 7-7" />
            </svg>
          </button>
          <h2 className="text-lg font-display text-text-primary">Settings</h2>
        </div>
      </div>

      {/* Scrollable content */}
      <div className="flex-1 overflow-y-auto px-5 py-4 space-y-5">
        {/* Hotkey */}
        <Section title="Hotkey">
          <HotkeyConfig
            value={settings.hotkey}
            onChange={(hotkey) => onUpdate({ hotkey })}
          />
        </Section>

        {/* Language */}
        <Section title="Language">
          <LanguageSelector
            value={settings.language}
            onChange={(language) => onUpdate({ language })}
          />
        </Section>

        {/* Mode */}
        <Section title="Transcription Mode">
          <ModeToggle
            value={settings.sttMode}
            onChange={(sttMode) => onUpdate({ sttMode })}
          />
          {settings.sttMode === "local" && (
            <p className="text-xs text-text-muted mt-1">
              Uses on-device Whisper. Requires model download (~3GB).
            </p>
          )}
        </Section>

        {/* STT Provider */}
        {settings.sttMode === "cloud" && (
          <Section title="STT Provider">
            <div className="space-y-1.5">
              {STT_PROVIDERS.map((provider) => (
                <button
                  key={provider.id}
                  onClick={() => onUpdate({ sttProvider: provider.id })}
                  className={`flex items-center gap-3 w-full px-3 py-2.5 rounded-lg text-sm border transition-all ${
                    settings.sttProvider === provider.id
                      ? "border-accent bg-accent-soft"
                      : "border-glass-border bg-bg-secondary hover:border-text-muted"
                  }`}
                >
                  <div className={`w-2 h-2 rounded-full flex-shrink-0 ${
                    settings.sttProvider === provider.id ? "bg-accent" : "bg-bg-elevated"
                  }`} />
                  <div className="text-left flex-1">
                    <span className={`font-medium ${
                      settings.sttProvider === provider.id ? "text-accent" : "text-text-primary"
                    }`}>
                      {provider.name}
                    </span>
                    <span className="text-text-muted ml-1.5 text-xs">
                      {provider.description}
                    </span>
                  </div>
                  {settings.sttProvider === provider.id && (
                    <motion.svg
                      initial={{ scale: 0 }}
                      animate={{ scale: 1 }}
                      className="w-4 h-4 text-accent flex-shrink-0"
                      viewBox="0 0 24 24"
                      fill="none"
                      stroke="currentColor"
                      strokeWidth={2.5}
                    >
                      <path strokeLinecap="round" strokeLinejoin="round" d="M5 13l4 4L19 7" />
                    </motion.svg>
                  )}
                </button>
              ))}
            </div>
          </Section>
        )}

        {/* AI Cleanup */}
        <Section title="AI Cleanup">
          <div className="rounded-lg border border-glass-border bg-bg-secondary p-3 space-y-1">
            <Toggle
              label="AI Cleanup"
              value={settings.aiCleanup}
              onChange={(v) => onUpdate({ aiCleanup: v })}
            />
            <Toggle
              label="Remove fillers"
              value={settings.removeFillers}
              onChange={(v) => onUpdate({ removeFillers: v })}
            />
            <Toggle
              label="Fix grammar"
              value={settings.fixGrammar}
              onChange={(v) => onUpdate({ fixGrammar: v })}
            />
            <Toggle
              label="Adapt tone"
              value={settings.adaptTone}
              onChange={(v) => onUpdate({ adaptTone: v })}
            />
          </div>
        </Section>

        {/* System */}
        <Section title="System">
          <div className="rounded-lg border border-glass-border bg-bg-secondary p-3">
            <Toggle
              label="Start at login"
              value={settings.autoStart}
              onChange={(v) => {
                onUpdate({ autoStart: v });
                setAutoStart(v).catch((err) =>
                  console.error("Failed to toggle auto-start:", err)
                );
              }}
            />
          </div>
        </Section>

        {/* API Keys */}
        <Section title="API Keys">
          <div className="space-y-2.5">
            <ApiKeyInput
              label="Groq"
              value={settings.groqApiKey}
              onChange={(v) => onUpdate({ groqApiKey: v })}
              placeholder="gsk_..."
            />
            <ApiKeyInput
              label="Deepgram"
              value={settings.deepgramApiKey}
              onChange={(v) => onUpdate({ deepgramApiKey: v })}
              placeholder="Token..."
            />
            <ApiKeyInput
              label="AssemblyAI"
              value={settings.assemblyaiApiKey}
              onChange={(v) => onUpdate({ assemblyaiApiKey: v })}
              placeholder="API key..."
            />
            <ApiKeyInput
              label="Anthropic (AI cleanup)"
              value={settings.anthropicApiKey}
              onChange={(v) => onUpdate({ anthropicApiKey: v })}
              placeholder="sk-ant-..."
            />
          </div>
        </Section>

        {/* Footer */}
        <div className="flex items-center justify-center gap-4 pt-1 pb-2 text-xs text-text-muted">
          <span>Voz v0.1.0</span>
        </div>
      </div>
    </div>
  );
}
