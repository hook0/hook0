import {Plugin} from 'vue';
import Keycloak from 'keycloak-js';

const keycloak = Keycloak({
  url: process.env.VUE_APP_KEYCLOAK_URL,
  realm: process.env.VUE_APP_KEYCLOAK_REALM,
  clientId: process.env.VUE_APP_KEYCLOAK_FRONT_CLIENT_ID,
});

keycloak.onTokenExpired = () => {
  keycloak.updateToken(3600).catch(async (_err) => {
    await keycloak.login();
  });
};

const auth$ = keycloak.init({
  onLoad: 'login-required',
  redirectUri: window.location.href,
  enableLogging: process.env.NODE_ENV !== 'production',
  checkLoginIframe: false
});


export const KeycloakPlugin: Plugin = {
  install: (app, _options) => {
    app.config.globalProperties.$keycloak = keycloak;
  }
};

declare module '@vue/runtime-core' {
  export interface ComponentCustomProperties {
    $keycloak: Keycloak.KeycloakInstance;
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
