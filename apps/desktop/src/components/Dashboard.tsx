import { useState, useEffect } from "react";
import { motion, AnimatePresence } from "framer-motion";
import type { RecordingState, AppSettings } from "../lib/constants";
import { Waveform } from "./Waveform";

interface DashboardProps {
  recording: RecordingState;
  settings: AppSettings;
  onOpenSettings: () => void;
}

function formatTime(seconds: number) {
  const m = Math.floor(seconds / 60);
  const s = seconds % 60;
  return `${m}:${s.toString().padStart(2, "0")}`;
}

function formatRelativeTime(date: Date) {
  const seconds = Math.floor((Date.now() - date.getTime()) / 1000);
  if (seconds < 5) return "just now";
  if (seconds < 60) return `${seconds}s ago`;
  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) return `${minutes}m ago`;
  return `${Math.floor(minutes / 60)}h ago`;
}

interface TranscriptEntry {
  text: string;
  timestamp: Date;
}

const displayHotkey = (hotkey: string) =>
  hotkey
    .replace("Alt", "⌥")
    .replace("Cmd", "⌘")
    .replace("Ctrl", "⌃")
    .replace("Shift", "⇧")
    .replace(/\+/g, " ");

export function Dashboard({ recording, settings, onOpenSettings }: DashboardProps) {
  const { status, duration, audioLevel, transcript } = recording;
  const [history, setHistory] = useState<TranscriptEntry[]>([]);

  // Track transcription history
  useEffect(() => {
    if (status === "done" && transcript) {
      setHistory((prev) => [
        { text: transcript, timestamp: new Date() },
        ...prev.slice(0, 4), // keep last 5
      ]);
    }
  }, [status, transcript]);

  // Refresh relative timestamps
  const [, setTick] = useState(0);
  useEffect(() => {
    const interval = setInterval(() => setTick((t) => t + 1), 10000);
    return () => clearInterval(interval);
  }, []);

  const isActive = status !== "idle";

  const providerLabel =
    settings.sttProvider === "groq" ? "Groq" :
    settings.sttProvider === "deepgram" ? "Deepgram" :
    settings.sttProvider === "assemblyai" ? "AssemblyAI" : "Local";

  return (
    <div className="h-full flex flex-col bg-bg-primary">
      {/* Drag region + settings */}
      <div className="flex items-center justify-end px-3 pt-2 pb-0">
        <button
          onClick={onOpenSettings}
          className="w-7 h-7 flex items-center justify-center rounded-md
            text-text-muted hover:text-text-secondary hover:bg-bg-secondary/60 transition-all"
          aria-label="Settings"
        >
          <svg className="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={1.5}>
            <path d="M12.22 2h-.44a2 2 0 00-2 2v.18a2 2 0 01-1 1.73l-.43.25a2 2 0 01-2 0l-.15-.08a2 2 0 00-2.73.73l-.22.38a2 2 0 00.73 2.73l.15.1a2 2 0 011 1.72v.51a2 2 0 01-1 1.74l-.15.09a2 2 0 00-.73 2.73l.22.38a2 2 0 002.73.73l.15-.08a2 2 0 012 0l.43.25a2 2 0 011 1.73V20a2 2 0 002 2h.44a2 2 0 002-2v-.18a2 2 0 011-1.73l.43-.25a2 2 0 012 0l.15.08a2 2 0 002.73-.73l.22-.39a2 2 0 00-.73-2.73l-.15-.08a2 2 0 01-1-1.74v-.5a2 2 0 011-1.74l.15-.09a2 2 0 00.73-2.73l-.22-.38a2 2 0 00-2.73-.73l-.15.08a2 2 0 01-2 0l-.43-.25a2 2 0 01-1-1.73V4a2 2 0 00-2-2z" />
            <circle cx="12" cy="12" r="3" />
          </svg>
        </button>
      </div>

      {/* Main mic area */}
      <div className="flex-shrink-0 flex flex-col items-center pt-6 pb-5 px-6">
        {/* Mic button container */}
        <div className="relative mb-4">
          {/* Outer glow ring — only when active */}
          <AnimatePresence>
            {isActive && (
              <motion.div
                initial={{ opacity: 0, scale: 0.8 }}
                animate={{ opacity: 1, scale: 1 }}
                exit={{ opacity: 0, scale: 0.8 }}
                className="absolute -inset-4 rounded-full"
                style={{
                  background: status === "listening"
                    ? "radial-gradient(circle, rgba(239,68,68,0.12) 0%, transparent 70%)"
                    : status === "processing"
                    ? "radial-gradient(circle, rgba(245,158,11,0.12) 0%, transparent 70%)"
                    : "radial-gradient(circle, rgba(52,211,153,0.12) 0%, transparent 70%)",
                }}
              />
            )}
          </AnimatePresence>

          {/* Pulsing ring when listening */}
          <AnimatePresence>
            {status === "listening" && (
              <motion.div
                initial={{ opacity: 0.6, scale: 1 }}
                animate={{ opacity: 0, scale: 1.6 }}
                transition={{ duration: 1.5, repeat: Infinity, ease: "easeOut" }}
                className="absolute inset-0 rounded-full border-2 border-recording/40"
              />
            )}
          </AnimatePresence>

          {/* Main circle — clickable */}
          <motion.button
            onClick={() => {
              if (status === "idle") recording.startRecording();
              else if (status === "listening") recording.stopRecording();
            }}
            whileHover={status === "idle" ? { scale: 1.06 } : {}}
            whileTap={status === "idle" ? { scale: 0.97 } : {}}
            animate={{
              scale: status === "listening" ? 1.05 : 1,
              borderColor: status === "listening"
                ? "rgba(239,68,68,0.5)"
                : status === "processing"
                ? "rgba(245,158,11,0.4)"
                : status === "done"
                ? "rgba(52,211,153,0.4)"
                : "rgba(255,255,255,0.06)",
            }}
            transition={{ type: "spring", stiffness: 300, damping: 25 }}
            className="relative w-[88px] h-[88px] rounded-full border flex items-center justify-center cursor-pointer"
            style={{
              background: isActive
                ? status === "listening"
                  ? "linear-gradient(145deg, rgba(239,68,68,0.08), rgba(239,68,68,0.03))"
                  : status === "processing"
                  ? "linear-gradient(145deg, rgba(245,158,11,0.08), rgba(245,158,11,0.03))"
                  : "linear-gradient(145deg, rgba(52,211,153,0.08), rgba(52,211,153,0.03))"
                : "linear-gradient(145deg, #292524, #1C1917)",
              boxShadow: isActive
                ? status === "listening"
                  ? "0 0 30px rgba(239,68,68,0.1), inset 0 1px 0 rgba(255,255,255,0.04)"
                  : "0 0 30px rgba(245,158,11,0.1), inset 0 1px 0 rgba(255,255,255,0.04)"
                : "0 2px 8px rgba(0,0,0,0.3), inset 0 1px 0 rgba(255,255,255,0.04)",
            }}
          >
            <AnimatePresence mode="wait">
              {status === "listening" && (
                <motion.div
                  key="waveform"
                  initial={{ opacity: 0 }}
                  animate={{ opacity: 1 }}
                  exit={{ opacity: 0 }}
                >
                  <Waveform level={audioLevel} isActive={true} color="#EF4444" barCount={5} />
                </motion.div>
              )}
              {status === "processing" && (
                <motion.div
                  key="spinner"
                  initial={{ opacity: 0 }}
                  animate={{ opacity: 1, rotate: 360 }}
                  exit={{ opacity: 0 }}
                  transition={{ rotate: { duration: 1, repeat: Infinity, ease: "linear" } }}
                  className="w-6 h-6 rounded-full border-2 border-accent border-t-transparent"
                />
              )}
              {status === "done" && (
                <motion.svg
                  key="check"
                  initial={{ opacity: 0, scale: 0 }}
                  animate={{ opacity: 1, scale: 1 }}
                  exit={{ opacity: 0 }}
                  transition={{ type: "spring", stiffness: 500, damping: 20 }}
                  className="w-7 h-7 text-success"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  strokeWidth={2.5}
                >
                  <path strokeLinecap="round" strokeLinejoin="round" d="M5 13l4 4L19 7" />
                </motion.svg>
              )}
              {status === "error" && (
                <motion.span
                  key="error"
                  initial={{ opacity: 0 }}
                  animate={{ opacity: 1 }}
                  exit={{ opacity: 0 }}
                  className="text-recording text-lg font-semibold"
                >
                  !
                </motion.span>
              )}
              {status === "idle" && (
                <motion.svg
                  key="mic"
                  initial={{ opacity: 0 }}
                  animate={{ opacity: 1 }}
                  exit={{ opacity: 0 }}
                  className="w-7 h-7 text-text-muted"
                  viewBox="0 0 24 24"
                  fill="currentColor"
                >
                  <path d="M12 14c1.66 0 3-1.34 3-3V5c0-1.66-1.34-3-3-3S9 3.34 9 5v6c0 1.66 1.34 3 3 3z" />
                  <path d="M17 11c0 2.76-2.24 5-5 5s-5-2.24-5-5H5c0 3.53 2.61 6.43 6 6.92V21h2v-3.08c3.39-.49 6-3.39 6-6.92h-2z" />
                </motion.svg>
              )}
            </AnimatePresence>
          </motion.button>
        </div>

        {/* Status label */}
        <AnimatePresence mode="wait">
          <motion.div
            key={status}
            initial={{ opacity: 0, y: 4 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -4 }}
            transition={{ duration: 0.15 }}
            className="text-center"
          >
            {status === "idle" && (
              <div className="flex items-center gap-2 text-text-muted text-[13px]">
                <span>Press</span>
                <kbd className="px-1.5 py-0.5 rounded bg-bg-secondary border border-glass-border text-text-secondary text-[11px] font-mono">
                  {displayHotkey(settings.hotkey)}
                </kbd>
                <span>to dictate</span>
              </div>
            )}
            {status === "listening" && (
              <div className="flex items-center gap-2">
                <span className="relative flex h-2 w-2">
                  <span className="absolute inset-0 rounded-full bg-recording animate-ping opacity-75" />
                  <span className="relative rounded-full h-2 w-2 bg-recording" />
                </span>
                <span className="text-[13px] font-medium text-recording">Listening</span>
                <span className="text-[13px] tabular-nums text-text-muted">{formatTime(duration)}</span>
              </div>
            )}
            {status === "processing" && (
              <span className="text-[13px] font-medium text-accent">Transcribing...</span>
            )}
            {status === "done" && (
              <span className="text-[13px] font-medium text-success">Done</span>
            )}
            {status === "error" && (
              <span className="text-[13px] font-medium text-recording">
                {recording.error || "Something went wrong"}
              </span>
            )}
          </motion.div>
        </AnimatePresence>
      </div>

      {/* Divider */}
      <div className="mx-5 border-t border-glass-border" />

      {/* Recent transcriptions */}
      <div className="flex-1 overflow-y-auto px-5 pt-3 pb-2">
        <div className="flex items-center justify-between mb-2">
          <span className="text-[11px] font-medium uppercase tracking-wider text-text-muted">
            Recent
          </span>
          {history.length > 0 && (
            <button
              onClick={() => setHistory([])}
              className="text-[11px] text-text-muted hover:text-text-secondary transition-colors"
            >
              Clear
            </button>
          )}
        </div>

        <AnimatePresence>
          {history.length === 0 ? (
            <motion.div
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              className="flex flex-col items-center justify-center py-8 text-center"
            >
              <p className="text-text-muted text-[13px] leading-relaxed">
                Your transcriptions will appear here.
              </p>
              <p className="text-text-muted/60 text-[12px] mt-1">
                Press {displayHotkey(settings.hotkey)} anywhere to start.
              </p>
            </motion.div>
          ) : (
            <div className="space-y-2">
              {history.map((entry, i) => (
                <motion.div
                  key={entry.timestamp.getTime()}
                  initial={{ opacity: 0, y: 8 }}
                  animate={{ opacity: 1, y: 0 }}
                  exit={{ opacity: 0, height: 0 }}
                  transition={{ delay: i * 0.05 }}
                  className="group px-3 py-2.5 rounded-lg bg-bg-secondary/50 border border-glass-border
                    hover:bg-bg-secondary transition-colors cursor-default"
                >
                  <p className="text-[13px] text-text-primary leading-snug line-clamp-2">
                    {entry.text}
                  </p>
                  <p className="text-[11px] text-text-muted mt-1.5">
                    {formatRelativeTime(entry.timestamp)}
                  </p>
                </motion.div>
              ))}
            </div>
          )}
        </AnimatePresence>
      </div>

      {/* Bottom status bar */}
      <div className="flex-shrink-0 px-5 py-3 border-t border-glass-border">
        <div className="flex items-center gap-2">
          <StatusChip
            icon={
              <svg className="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2}>
                <path d="M18 10h-1.26A8 8 0 109 20h9a5 5 0 000-10z" />
              </svg>
            }
            label={settings.sttMode === "cloud" ? providerLabel : "Local"}
          />
          <StatusChip
            icon={
              settings.aiCleanup ? (
                <svg className="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2}>
                  <path d="M12 2L9 12l-7 1 5.5 4.5L5 22l7-4 7 4-2.5-4.5L22 13l-7-1z" />
                </svg>
              ) : null
            }
            label={settings.aiCleanup ? "AI Cleanup" : "Raw"}
          />
          <StatusChip
            label={settings.language === "auto" ? "Auto" : settings.language.toUpperCase()}
          />
          <div className="flex-1" />
          <button
            onClick={onOpenSettings}
            className="text-[11px] text-text-muted hover:text-accent transition-colors"
          >
            Settings
          </button>
        </div>
      </div>
    </div>
  );
}

function StatusChip({ icon, label }: { icon?: React.ReactNode; label: string }) {
  return (
    <div className="flex items-center gap-1 px-2 py-1 rounded-md bg-bg-secondary/50 border border-glass-border text-text-muted text-[11px]">
      {icon}
      <span>{label}</span>
    </div>
  );
}
