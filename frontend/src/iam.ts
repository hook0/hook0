import { InjectionKey, Plugin } from 'vue';
import Keycloak, { KeycloakConfig } from 'keycloak-js';

export const keycloakKey = Symbol() as InjectionKey<Keycloak>;

function getParams(): string | KeycloakConfig {
  if (import.meta.env.VITE_KEYCLOAK_URL) {
    return {
      url: import.meta.env.VITE_KEYCLOAK_URL,
      realm: import.meta.env.VITE_KEYCLOAK_REALM ?? '',
      clientId: import.meta.env.VITE_KEYCLOAK_FRONT_CLIENT_ID ?? '',
    };
  }
  return '/keycloak.json';
}

const keycloak = new Keycloak(getParams());

keycloak.onTokenExpired = () => {
  void onTokenExpired();
};

export function onTokenExpired() {
  return keycloak.updateToken(3600).catch(async (_err) => {
    console.error(_err);
    return keycloak.login();
  });
}

const auth$ = keycloak.init({
  onLoad: 'login-required',
  redirectUri: window.location.href,
  enableLogging: import.meta.env.NODE_ENV !== 'production',
  checkLoginIframe: false,
});

export const KeycloakPlugin: Plugin = {
  install: (app, _options) => {
    app.provide(keycloakKey, keycloak);
  },
};

export interface KeycloakTokenParsedAttributes {
  email: string;
}

export function getToken(): Promise<string> {
  return auth$.then((auth) => {
    if (!auth) {
      window.location.reload();
    }

    return keycloak.token as string;
  });
}
