interface ImportMetaEnv {
  readonly NODE_ENV: 'development' | 'production' | 'test';
  readonly DEV: boolean;
  readonly PROD: boolean;
  readonly VITE_API_TIMEOUT: string;
  readonly VITE_API_ENDPOINT: string;
  readonly VITE_CRISP_WEBSITE_ID?: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
