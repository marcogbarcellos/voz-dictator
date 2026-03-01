import { motion, AnimatePresence } from "framer-motion";
import { Waveform } from "./Waveform";
import type { RecordingState } from "../lib/constants";

interface FloatingPillProps {
  state: RecordingState;
}

/**
 * Compact pill overlay for recording status.
 * Used as an optional overlay on top of the Dashboard.
 * The Dashboard's AmbientOrb is the primary visual indicator.
 */
export function FloatingPill({ state }: FloatingPillProps) {
  const { status, duration, audioLevel } = state;

  // Only show during active recording states
  if (status === "idle") return null;

  const formatTime = (seconds: number) => {
    const m = Math.floor(seconds / 60);
    const s = seconds % 60;
    return `${m}:${s.toString().padStart(2, "0")}`;
  };

  return (
    <AnimatePresence>
      <motion.div
        initial={{ opacity: 0, y: 20, scale: 0.95 }}
        animate={{ opacity: 1, y: 0, scale: 1 }}
        exit={{ opacity: 0, y: 10, scale: 0.95 }}
        transition={{ type: "spring", stiffness: 400, damping: 30 }}
        className="fixed bottom-8 left-1/2 -translate-x-1/2 z-50"
      >
        <div
          className="flex items-center gap-3 px-5 py-2.5 rounded-full border"
          style={{
            background: "rgba(41, 37, 36, 0.82)",
            backdropFilter: "blur(20px)",
            WebkitBackdropFilter: "blur(20px)",
            borderColor: "rgba(255, 255, 255, 0.08)",
            boxShadow: "0 8px 32px rgba(0, 0, 0, 0.4)",
          }}
        >
          {status === "listening" && (
            <>
              <span className="relative flex h-2.5 w-2.5">
                <span
                  className="absolute inline-flex h-full w-full rounded-full opacity-75 animate-ping"
                  style={{ backgroundColor: "#EF4444" }}
                />
                <span
                  className="relative inline-flex rounded-full h-2.5 w-2.5"
                  style={{ backgroundColor: "#EF4444" }}
                />
              </span>
              <span className="text-sm font-medium text-text-primary">
                Listening...
              </span>
              <Waveform level={audioLevel} isActive={true} />
              <span className="text-sm tabular-nums text-text-secondary">
                {formatTime(duration)}
              </span>
            </>
          )}

          {status === "processing" && (
            <>
              <motion.div
                animate={{ rotate: 360 }}
                transition={{
                  duration: 1,
                  repeat: Infinity,
                  ease: "linear",
                }}
                className="w-4 h-4 rounded-full border-2 border-accent border-t-transparent"
              />
              <span className="text-sm font-medium text-text-primary">
                Transcribing...
              </span>
            </>
          )}

          {status === "done" && (
            <>
              <motion.svg
                initial={{ scale: 0 }}
                animate={{ scale: 1 }}
                transition={{ type: "spring", stiffness: 500, damping: 25 }}
                className="w-4 h-4 text-success"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth={3}
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  d="M5 13l4 4L19 7"
                />
              </motion.svg>
              <span className="text-sm font-medium text-success">Done</span>
            </>
          )}

          {status === "error" && (
            <>
              <span className="w-4 h-4 text-recording text-center text-xs font-bold">
                !
              </span>
              <span className="text-sm font-medium text-recording">Error</span>
            </>
          )}
        </div>
      </motion.div>
    </AnimatePresence>
  );
}
