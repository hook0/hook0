import { Plugin } from 'vue';
import Keycloak from 'keycloak-js';

function getParams() {
  if (process.env.VUE_APP_KEYCLOAK_URL) {
    return {
      url: process.env.VUE_APP_KEYCLOAK_URL,
      realm: process.env.VUE_APP_KEYCLOAK_REALM,
      clientId: process.env.VUE_APP_KEYCLOAK_FRONT_CLIENT_ID,
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
  enableLogging: process.env.NODE_ENV !== 'production',
  checkLoginIframe: false,
});

export const KeycloakPlugin: Plugin = {
  install: (app, _options) => {
    app.config.globalProperties.$keycloak = keycloak;
  },
};

declare module '@vue/runtime-core' {
  export interface ComponentCustomProperties {
    $keycloak: Keycloak;
  }
}

export interface KeycloakTokenParsedAttributes {
  email: string;
}

export default {
  getToken(): Promise<string> {
    return auth$.then((auth) => {
      if (!auth) {
        window.location.reload();
      }

      return keycloak.token as string;
    });
  },
};
