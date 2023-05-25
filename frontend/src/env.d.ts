declare namespace __WebpackModuleApi {
  interface NodeProcess {
    env: {
      readonly NODE_ENV: 'development' | 'production' | 'test';
      readonly BASE_URL: string;
      readonly VUE_APP_API_ENDPOINT: string;
      readonly VUE_APP_KEYCLOAK_URL: string;
      readonly VUE_APP_KEYCLOAK_REALM: string;
      readonly VUE_APP_KEYCLOAK_FRONT_CLIENT_ID: string;
      readonly VUE_APP_FEATURES_KEYCLOAK?: string;
      readonly VUE_APP_ENABLE_QUOTA_ENFORCEMENT?: boolean;
    };
  }
}
