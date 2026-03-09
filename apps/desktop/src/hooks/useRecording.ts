import { useState, useCallback, useRef, useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import {
  AppSettings,
  RecordingState,
  RecordingStatus,
} from "../lib/constants";
import * as commands from "../lib/tauri-commands";

export function useRecording(settings: AppSettings): RecordingState {
  const [status, setStatus] = useState<RecordingStatus>("idle");
  const [duration, setDuration] = useState(0);
  const [audioLevel, setAudioLevel] = useState(0);
  const [transcript, setTranscript] = useState("");
  const [error, setError] = useState<string | null>(null);
  const timerRef = useRef<ReturnType<typeof setInterval>>(undefined);
  const levelRef = useRef<ReturnType<typeof setInterval>>(undefined);

  const startRecording = useCallback(async () => {
    try {
      setStatus("listening");
      setDuration(0);
      setTranscript("");
      setError(null);

      await commands.startRecording(settings.language);

      timerRef.current = setInterval(() => {
        setDuration((d) => d + 1);
      }, 1000);

      // Audio level polling disabled — IPC calls during recording
      // interfere with Bluetooth mic (AirPods) CoreAudio stream,
      // causing silence after ~4 seconds. Tray icon provides
      // recording feedback instead.
    } catch (err) {
      setStatus("error");
      setError(err instanceof Error ? err.message : "Failed to start recording");
    }
  }, [settings.language]);

  const stopRecording = useCallback(async () => {
    clearInterval(timerRef.current);
    clearInterval(levelRef.current);

    try {
      setStatus("processing");
      const result = await commands.stopRecording();

      let finalText = result;
      if (settings.aiCleanup) {
        try {
          const activeApp = await commands.getActiveApp().catch(() => "unknown");
          finalText = await commands.cleanupText(
            result,
            settings.language,
            activeApp
          );
        } catch (cleanupErr) {
          console.warn("AI cleanup failed, using raw transcript:", cleanupErr);
        }
      }
      setTranscript(finalText);
      const injectResult = await commands.smartInjectText(finalText);
      if (injectResult === "copied") {
        setTranscript((prev) => prev + " (copied to clipboard)");
      }

      setStatus("done");
      setTimeout(() => setStatus("idle"), 1500);
    } catch (err) {
      setStatus("error");
      setError(err instanceof Error ? err.message : "Transcription failed");
      setTimeout(() => setStatus("idle"), 3000);
    }
  }, [settings.aiCleanup, settings.language]);

  useEffect(() => {
    return () => {
      clearInterval(timerRef.current);
      clearInterval(levelRef.current);
    };
  }, []);

  // Listen for hotkey events from Tauri
  useEffect(() => {
    const unlisten = listen("hotkey-toggle", () => {
      if (status === "idle") {
        startRecording();
      } else if (status === "listening") {
        stopRecording();
      }
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, [status, startRecording, stopRecording]);

  return {
    status,
    duration,
    audioLevel,
    transcript,
    error,
    startRecording,
    stopRecording,
  };
}
