import { createI18n } from 'vue-i18n';
import en from '@/locales/en.json';
import type { App } from 'vue';

const i18n = createI18n({
  legacy: false, // composition API mode
  locale: 'en',
  fallbackLocale: 'en',
  messages: {
    en,
  },
  datetimeFormats: {
    en: {
      short: {
        year: 'numeric',
        month: 'short',
        day: 'numeric',
      },
      long: {
        year: 'numeric',
        month: 'long',
        day: 'numeric',
        hour: 'numeric',
        minute: 'numeric',
        second: 'numeric',
      },
      time: {
        hour: 'numeric',
        minute: 'numeric',
      },
    },
  },
});

export function setupI18n(app: App): void {
  app.use(i18n);
}

export default i18n;
