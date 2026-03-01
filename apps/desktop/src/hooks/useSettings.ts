import { useState, useEffect, useCallback } from "react";
import { AppSettings, DEFAULT_SETTINGS } from "../lib/constants";
import * as commands from "../lib/tauri-commands";

const STORAGE_KEY = "voz-settings";

/** Maps Rust snake_case settings to frontend camelCase */
function fromRust(rust: Record<string, unknown>): Partial<AppSettings> {
  return {
    onboardingComplete: rust.onboarding_complete as boolean | undefined,
    language: (rust.language as AppSettings["language"]) ?? undefined,
    sttMode: (rust.stt_mode as AppSettings["sttMode"]) ?? undefined,
    sttProvider: (rust.stt_provider as AppSettings["sttProvider"]) ?? undefined,
    aiCleanup: rust.ai_cleanup as boolean | undefined,
    removeFillers: rust.remove_fillers as boolean | undefined,
    fixGrammar: rust.fix_grammar as boolean | undefined,
    adaptTone: rust.adapt_tone as boolean | undefined,
    groqApiKey: (rust.groq_api_key as string) ?? undefined,
    deepgramApiKey: (rust.deepgram_api_key as string) ?? undefined,
    assemblyaiApiKey: (rust.assemblyai_api_key as string) ?? undefined,
    anthropicApiKey: (rust.anthropic_api_key as string) ?? undefined,
    hotkey: (rust.hotkey as string) ?? undefined,
    localModelPath: (rust.local_model_path as string) ?? undefined,
  };
}

/** Maps frontend camelCase updates to Rust snake_case */
function toRust(updates: Partial<AppSettings>): Record<string, unknown> {
  const out: Record<string, unknown> = {};
  if (updates.onboardingComplete !== undefined) out.onboarding_complete = updates.onboardingComplete;
  if (updates.language !== undefined) out.language = updates.language;
  if (updates.sttMode !== undefined) out.stt_mode = updates.sttMode;
  if (updates.sttProvider !== undefined) out.stt_provider = updates.sttProvider;
  if (updates.aiCleanup !== undefined) out.ai_cleanup = updates.aiCleanup;
  if (updates.removeFillers !== undefined) out.remove_fillers = updates.removeFillers;
  if (updates.fixGrammar !== undefined) out.fix_grammar = updates.fixGrammar;
  if (updates.adaptTone !== undefined) out.adapt_tone = updates.adaptTone;
  if (updates.groqApiKey !== undefined) out.groq_api_key = updates.groqApiKey;
  if (updates.deepgramApiKey !== undefined) out.deepgram_api_key = updates.deepgramApiKey;
  if (updates.assemblyaiApiKey !== undefined) out.assemblyai_api_key = updates.assemblyaiApiKey;
  if (updates.anthropicApiKey !== undefined) out.anthropic_api_key = updates.anthropicApiKey;
  if (updates.hotkey !== undefined) out.hotkey = updates.hotkey;
  if (updates.localModelPath !== undefined) out.local_model_path = updates.localModelPath;
  return out;
}

export function useSettings() {
  const [settings, setSettings] = useState<AppSettings>(DEFAULT_SETTINGS);
  const [isLoaded, setIsLoaded] = useState(false);

  // Load from Rust backend (source of truth), fallback to localStorage
  useEffect(() => {
    async function load() {
      try {
        const rust = await commands.getSettings();
        const mapped = fromRust(rust);
        setSettings({ ...DEFAULT_SETTINGS, ...mapped });
      } catch {
        // Tauri not available — fallback to localStorage
        try {
          const stored = localStorage.getItem(STORAGE_KEY);
          if (stored) {
            setSettings({ ...DEFAULT_SETTINGS, ...JSON.parse(stored) });
          }
        } catch {
          // Use defaults
        }
      }
      setIsLoaded(true);
    }
    load();
  }, []);

  const updateSettings = useCallback(
    (updates: Partial<AppSettings>) => {
      setSettings((prev) => {
        const next = { ...prev, ...updates };

        // Save settings to Rust backend
        const rustUpdates = toRust(updates);
        if (Object.keys(rustUpdates).length > 0) {
          commands.updateSettings(rustUpdates).catch((err) => {
            console.error("Failed to save settings to backend:", err);
          });
        }

        return next;
      });
    },
    []
  );

  return { settings, updateSettings, isLoaded };
}
