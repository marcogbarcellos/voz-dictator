import { useCallback } from "react";
import { AppSettings } from "../lib/constants";
import * as commands from "../lib/tauri-commands";

export function useTranscription(settings: AppSettings) {
  const transcribe = useCallback(
    async (audioPath: string): Promise<string> => {
      return commands.transcribeAudio(
        audioPath,
        settings.language,
        settings.sttProvider
      );
    },
    [settings.language, settings.sttProvider]
  );

  const cleanup = useCallback(
    async (text: string, appContext: string): Promise<string> => {
      if (!settings.aiCleanup) return text;
      return commands.cleanupText(text, settings.language, appContext);
    },
    [settings.aiCleanup, settings.language]
  );

  return { transcribe, cleanup };
}
