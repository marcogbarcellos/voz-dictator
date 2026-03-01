import { useState, useEffect, useCallback } from "react";

interface HotkeyConfigProps {
  value: string;
  onChange: (hotkey: string) => void;
}

export function HotkeyConfig({ value, onChange }: HotkeyConfigProps) {
  const [recording, setRecording] = useState(false);

  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      if (!recording) return;
      e.preventDefault();

      const parts: string[] = [];
      if (e.metaKey) parts.push("Cmd");
      if (e.ctrlKey) parts.push("Ctrl");
      if (e.altKey) parts.push("Alt");
      if (e.shiftKey) parts.push("Shift");

      const key = e.key;
      if (!["Meta", "Control", "Alt", "Shift"].includes(key)) {
        parts.push(key === " " ? "Space" : key.length === 1 ? key.toUpperCase() : key);
        onChange(parts.join("+"));
        setRecording(false);
      }
    },
    [recording, onChange]
  );

  useEffect(() => {
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [handleKeyDown]);

  const displayHotkey = (hotkey: string) =>
    hotkey
      .replace("Alt", "⌥")
      .replace("Cmd", "⌘")
      .replace("Ctrl", "⌃")
      .replace("Shift", "⇧")
      .replace(/\+/g, " ");

  return (
    <button
      onClick={() => setRecording(true)}
      className={`px-3 py-2 rounded-lg text-sm font-mono border transition-all w-full text-center
        ${
          recording
            ? "border-accent bg-accent-soft text-accent animate-pulse"
            : "border-glass-border bg-bg-secondary text-text-primary hover:bg-bg-elevated"
        }`}
    >
      {recording ? "Press shortcut..." : displayHotkey(value)}
    </button>
  );
}
