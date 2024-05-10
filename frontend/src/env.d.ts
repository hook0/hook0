interface ImportMeta {
  env: {
    readonly NODE_ENV: 'development' | 'production' | 'test';
    readonly BASE_URL: string;
    readonly VITE_API_TIMEOUT: string;
    readonly VITE_API_ENDPOINT: string;
    readonly VITE_KEYCLOAK_URL: string;
    readonly VITE_KEYCLOAK_REALM: string;
    readonly VITE_KEYCLOAK_FRONT_CLIENT_ID: string;
    readonly VITE_FEATURES_KEYCLOAK?: string;
    readonly VITE_ENABLE_QUOTA_ENFORCEMENT?: string;
    readonly VITE_CRISP_WEBSITE_ID?: string;
    readonly VITE_BISCUIT_PUBLIC_KEY?: string;
  };
}
