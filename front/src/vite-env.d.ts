/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_API_BASE?: string;
  readonly VITE_API_KEY?: string;
  readonly VITE_AUTH_API_BASE?: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
