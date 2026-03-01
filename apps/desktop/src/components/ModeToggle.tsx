import { motion } from "framer-motion";
import type { SttMode } from "../lib/constants";

interface ModeToggleProps {
  value: SttMode;
  onChange: (mode: SttMode) => void;
}

export function ModeToggle({ value, onChange }: ModeToggleProps) {
  return (
    <div className="flex items-center rounded-lg bg-bg-secondary border border-glass-border p-0.5">
      {(["cloud", "local"] as const).map((mode) => (
        <button
          key={mode}
          onClick={() => onChange(mode)}
          className="relative flex-1 px-4 py-1.5 text-sm font-medium rounded-md transition-colors z-10"
          style={{
            color: value === mode ? "#FAFAF9" : "#78716C",
          }}
        >
          {value === mode && (
            <motion.div
              layoutId="mode-toggle"
              className="absolute inset-0 rounded-md bg-bg-elevated"
              transition={{ type: "spring", stiffness: 500, damping: 35 }}
              style={{ zIndex: -1 }}
            />
          )}
          <span className="flex items-center justify-center gap-1.5">
            {mode === "cloud" ? (
              <svg className="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2}>
                <path d="M18 10h-1.26A8 8 0 109 20h9a5 5 0 000-10z" />
              </svg>
            ) : (
              <svg className="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2}>
                <rect x="4" y="4" width="16" height="16" rx="2" />
                <path d="M9 9h6v6H9z" />
              </svg>
            )}
            {mode === "cloud" ? "Cloud" : "Local"}
          </span>
        </button>
      ))}
    </div>
  );
}
