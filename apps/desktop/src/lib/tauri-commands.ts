import { invoke } from "@tauri-apps/api/core";

export async function startRecording(language: string): Promise<void> {
  return invoke("start_recording", { language });
}

export async function stopRecording(): Promise<string> {
  return invoke("stop_recording");
}

export async function getAudioLevel(): Promise<number> {
  return invoke("get_audio_level");
}

export async function transcribeAudio(
  audioPath: string,
  language: string,
  provider: string
): Promise<string> {
  return invoke("transcribe_audio", { audioPath, language, provider });
}

export async function cleanupText(
  text: string,
  language: string,
  appContext: string
): Promise<string> {
  return invoke("cleanup_text", { text, language, appContext });
}

export async function injectText(text: string): Promise<void> {
  return invoke("inject_text", { text });
}

export async function getActiveApp(): Promise<string> {
  return invoke("get_active_app");
}

export async function setGlobalHotkey(hotkey: string): Promise<void> {
  return invoke("set_global_hotkey", { hotkey });
}

export async function checkPermissions(): Promise<{
  microphone: boolean;
  accessibility: boolean;
}> {
  return invoke("check_permissions");
}

export async function requestMicrophonePermission(): Promise<boolean> {
  return invoke("request_microphone_permission");
}

export async function requestAccessibilityPermission(): Promise<boolean> {
  return invoke("request_accessibility_permission");
}

export async function setRecordingState(state: string): Promise<void> {
  return invoke("set_recording_state", { recordingState: state });
}

export async function smartInjectText(text: string): Promise<string> {
  return invoke("smart_inject_text", { text });
}

export async function getSettings(): Promise<Record<string, unknown>> {
  return invoke("get_settings");
}

export async function updateSettings(
  updates: Record<string, unknown>
): Promise<void> {
  return invoke("update_settings", { updates });
}
