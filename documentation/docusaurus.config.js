// @ts-check
// `@type` JSDoc annotations allow editor autocompletion and type checking
// (when paired with `@ts-check`).
// There are various equivalent ways to declare your Docusaurus config.
// See: https://docusaurus.io/docs/api/docusaurus-config

import { themes as prismThemes } from "prism-react-renderer";
import path from "path";
import { fileURLToPath } from "url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const baseUrl = "/hook0/";

/** @type {import('@docusaurus/types').Config} */
const config = {
  title: "Hook0 Documentation",
  tagline:
    "Comprehensive documentation for Hook0 - The open-source webhook server",
  favicon: "favicon.ico",

  // Set the production url of your site here
  url: "https://hook0.gitlab.io",
  // Set the /<baseUrl>/ pathname under which your site is served
  baseUrl,

  // GitLab pages deployment config.
  organizationName: "hook0",
  projectName: "hook0",

  onBrokenLinks: "throw",
  onBrokenMarkdownLinks: "warn",

  staticDirectories: ["static"],

  // Even if you don't use internationalization, you can use this field to set
  // useful metadata like html lang. For example, if your site is Chinese, you
  // may want to replace "en" with "zh-Hans".
  i18n: {
    defaultLocale: "en",
    locales: ["en"],
  },

  presets: [
    [
      "classic",
      /** @type {import('@docusaurus/preset-classic').Options} */
      ({
        docs: {
          path: ".",
          sidebarPath: "./sidebars.js",
          routeBasePath: "/",
          editUrl: "https://gitlab.com/hook0/hook0/-/tree/main/documentation/",
          exclude: [
            "**/node_modules/**",
            "**/build/**",
            "**/.docusaurus/**",
            "**/scripts/**",
          ],
        },
        blog: false,
        theme: {
          customCss: "./src/css/custom.css",
        },
      }),
    ],
  ],

  plugins: [
    // Module alias plugin to import shared data from website/
    // Without this, importing via relative path (../../../../website/...) causes Webpack
    // to watch the entire parent directory, leading to EMFILE "too many open files" errors
    [
      "docusaurus-plugin-module-alias",
      {
        alias: {
          "@shared/website-data": path.resolve(__dirname, "../website/data.js"),
        },
      },
    ],
    [
      "@scalar/docusaurus",
      {
        label: "API Reference",
        route: "/api",
        showNavLink: false,
        configuration: {
          spec: {
            url: `${baseUrl}hook0-api.json`,
          },
          theme: "none",
          authentication: {
            preferredSecurityScheme: "apiToken",
          },
          customCss: `@import url('https://unpkg.com/highlight.js@11.11.1/styles/night-owl.min.css');`,
        },
      },
    ],
  ],

  themes: [
    "@docusaurus/theme-mermaid",
    [
      require.resolve("@easyops-cn/docusaurus-search-local"),
      {
        hashed: true,
        language: ["en", "fr"],
        highlightSearchTermsOnTargetPage: true,
        explicitSearchResultPath: true,
        docsDir: ".",
        docsRouteBasePath: "/",
      },
    ],
  ],

  markdown: {
    mermaid: true,
  },

  // Matomo analytics script (injected in <head>)
  headTags:
    process.env.DOCUMENTATION_MATOMO_URL && process.env.DOCUMENTATION_MATOMO_SITE_ID
      ? [
          {
            tagName: "script",
            attributes: {},
            innerHTML: `
              var _paq = window._paq = window._paq || [];
              _paq.push(['trackPageView']);
              _paq.push(['enableLinkTracking']);
              (function() {
                var u="${process.env.DOCUMENTATION_MATOMO_URL}";
                _paq.push(['setTrackerUrl', u+'matomo.php']);
                _paq.push(['setSiteId', '${process.env.DOCUMENTATION_MATOMO_SITE_ID}']);
                var d=document, g=d.createElement('script'), s=d.getElementsByTagName('script')[0];
                g.async=true; g.src=u+'matomo.js'; s.parentNode.insertBefore(g,s);
              })();
            `,
          },
        ]
      : [],

  themeConfig:
    /** @type {import('@docusaurus/preset-classic').ThemeConfig} */
    ({
      // Replace with your project's social card
      image: "img/hook0-social-card.jpg",
      navbar: {
        title: "Hook0",
        logo: {
          alt: "Hook0 Logo",
          src: "img/logo.svg",
          href: "https://www.hook0.com",
        },
        items: [
          // Left side
          {
            to: "/",
            label: "Documentation",
            position: "left",
            activeBaseRegex: "^/hook0/(?!tutorials|api).*$|^/hook0/$",
          },
          {
            to: "/tutorials",
            label: "Tutorials",
            position: "left",
            activeBaseRegex: "^/hook0/tutorials(/.*)?$",
          },
          {
            to: "/api",
            label: "Reference",
            position: "left",
            activeBaseRegex: "^/hook0/api(/.*)?$",
          },
          // Right side
          {
            href: "mailto:support@hook0.com",
            label: "Contact",
            position: "right",
            className: "navbar__item--no-external-icon",
          },
          {
            href: "https://app.hook0.com/",
            label: "Login",
            position: "right",
            className: "navbar__item--no-external-icon",
          },
          {
            href: "https://www.hook0.com/sign_up",
            label: "Get Started →",
            position: "right",
            className: "navbar__item--primary navbar__item--no-external-icon",
          },
        ],
      },
      // Footer is handled by custom component in src/theme/Footer
      prism: {
        theme: prismThemes.github,
        darkTheme: prismThemes.nightOwl,
        additionalLanguages: ["rust", "bash", "json", "yaml", "toml"],
      },
      announcementBar: {
        id: "hook0_v2",
        content:
          '🚀 Hook0 v2.0 is now available! Check out the <a href="/tutorials/getting-started">updated getting started guide</a> and <a href="https://github.com/hook0/hook0/releases">release notes</a>.',
        backgroundColor: "#4ade80",
        textColor: "#ffffff",
        isCloseable: true,
      },
      colorMode: {
        defaultMode: "dark",
        disableSwitch: false,
        respectPrefersColorScheme: true,
      },
      docs: {
        sidebar: {
          hideable: true,
          autoCollapseCategories: true,
        },
      },
    }),
};

export default config;
