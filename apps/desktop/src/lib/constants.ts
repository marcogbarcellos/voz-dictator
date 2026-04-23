export const LANGUAGES = [
  { code: "pt", label: "Português (BR)", flag: "🇧🇷", priority: true },
  { code: "en", label: "English", flag: "🇺🇸", priority: false },
  { code: "es", label: "Español", flag: "🇪🇸", priority: false },
  { code: "fr", label: "Français", flag: "🇫🇷", priority: false },
  { code: "de", label: "Deutsch", flag: "🇩🇪", priority: false },
  { code: "it", label: "Italiano", flag: "🇮🇹", priority: false },
  { code: "ja", label: "日本語", flag: "🇯🇵", priority: false },
  { code: "ko", label: "한국어", flag: "🇰🇷", priority: false },
  { code: "zh", label: "中文", flag: "🇨🇳", priority: false },
  { code: "auto", label: "Auto-detect", flag: "🌐", priority: false },
] as const;

export type LanguageCode = (typeof LANGUAGES)[number]["code"];

export type SttProvider = "groq" | "deepgram" | "assemblyai" | "local";
export type SttMode = "cloud" | "local";

export interface AppSettings {
  onboardingComplete: boolean;
  hotkey: string;
  language: LanguageCode;
  sttMode: SttMode;
  sttProvider: SttProvider;
  aiCleanup: boolean;
  removeFillers: boolean;
  fixGrammar: boolean;
  adaptTone: boolean;
  groqApiKey: string;
  deepgramApiKey: string;
  assemblyaiApiKey: string;
  anthropicApiKey: string;
  localModelPath: string;
  personalDictionary: string[];
  autoStart: boolean;
  personalLanguages: string[];
}

export const DEFAULT_SETTINGS: AppSettings = {
  onboardingComplete: false,
  hotkey: "Alt+Space",
  language: "pt",
  sttMode: "local",
  sttProvider: "local",
  aiCleanup: true,
  removeFillers: true,
  fixGrammar: true,
  adaptTone: false,
  groqApiKey: "",
  deepgramApiKey: "",
  assemblyaiApiKey: "",
  anthropicApiKey: "",
  localModelPath: "",
  personalDictionary: [],
  autoStart: false,
  personalLanguages: [],
};

export type RecordingStatus =
  | "idle"
  | "listening"
  | "processing"
  | "done"
  | "error";

export interface RecordingState {
  status: RecordingStatus;
  duration: number;
  audioLevel: number;
  transcript: string;
  error: string | null;
  startRecording: () => void;
  stopRecording: () => void;
}
