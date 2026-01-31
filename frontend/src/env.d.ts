interface ImportMeta {
  env: {
    readonly NODE_ENV: 'development' | 'production' | 'test';
    readonly VITE_API_TIMEOUT: string;
    readonly VITE_API_ENDPOINT: string;
    readonly VITE_CRISP_WEBSITE_ID?: string;
  };
}
