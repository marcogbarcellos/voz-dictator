/// <reference types="vite/client" />

interface Window {
  __TAURI__?: {
    event?: {
      listen: (
        event: string,
        handler: (event: { payload: unknown }) => void
      ) => Promise<() => void>;
      emit: (event: string, payload?: unknown) => Promise<void>;
    };
  };
}
