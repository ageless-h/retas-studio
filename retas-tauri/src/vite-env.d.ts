/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly MODE: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}

declare module "@tauri-apps/plugin-dialog" {
  export function open(options?: Record<string, unknown>): Promise<string | string[] | null>;
  export function save(options?: Record<string, unknown>): Promise<string | null>;
}
