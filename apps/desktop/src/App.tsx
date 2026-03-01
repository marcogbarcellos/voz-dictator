import { useState, useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { AnimatePresence, motion } from "framer-motion";
import { Dashboard } from "./components/Dashboard";
import { SettingsPanel } from "./components/SettingsPanel";
import { Onboarding } from "./components/Onboarding";
import { useRecording } from "./hooks/useRecording";
import { useSettings } from "./hooks/useSettings";

type View = "home" | "settings" | "onboarding";

export default function App() {
  const { settings, updateSettings, isLoaded } = useSettings();
  const recording = useRecording(settings);
  const [view, setView] = useState<View>("home");

  // Show onboarding on first run — window starts hidden, so show it
  useEffect(() => {
    if (isLoaded && !settings.onboardingComplete) {
      setView("onboarding");
      const win = getCurrentWindow();
      win.show();
      win.setFocus();
    }
  }, [isLoaded, settings.onboardingComplete]);

  // Listen for tray "open-settings" event — show + focus window
  useEffect(() => {
    const unlisten = listen("open-settings", () => {
      setView("settings");
      const win = getCurrentWindow();
      win.show();
      win.setFocus();
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  if (!isLoaded) {
    return (
      <div className="h-full flex items-center justify-center bg-bg-primary">
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          className="text-accent text-xl font-display italic"
        >
          Voz
        </motion.div>
      </div>
    );
  }

  return (
    <AnimatePresence mode="wait">
      {view === "onboarding" && (
        <motion.div
          key="onboarding"
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          transition={{ duration: 0.2 }}
          className="h-full"
        >
          <Onboarding
            onComplete={() => {
              updateSettings({ onboardingComplete: true });
              setView("home");
            }}
            onUpdate={updateSettings}
          />
        </motion.div>
      )}

      {view === "settings" && (
        <motion.div
          key="settings"
          initial={{ opacity: 0, x: 20 }}
          animate={{ opacity: 1, x: 0 }}
          exit={{ opacity: 0, x: 20 }}
          transition={{ duration: 0.2, ease: "easeOut" }}
          className="h-full"
        >
          <SettingsPanel
            settings={settings}
            onUpdate={updateSettings}
            onClose={() => setView("home")}
          />
        </motion.div>
      )}

      {view === "home" && (
        <motion.div
          key="home"
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          transition={{ duration: 0.2 }}
          className="h-full"
        >
          <Dashboard
            recording={recording}
            settings={settings}
            onOpenSettings={() => setView("settings")}
          />
        </motion.div>
      )}
    </AnimatePresence>
  );
}
