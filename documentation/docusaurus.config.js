// @ts-check
// `@type` JSDoc annotations allow editor autocompletion and type checking
// (when paired with `@ts-check`).
// There are various equivalent ways to declare your Docusaurus config.
// See: https://docusaurus.io/docs/api/docusaurus-config

import { themes as prismThemes } from "prism-react-renderer";

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
        },
        items: [
          {
            type: "docSidebar",
            sidebarId: "tutorialSidebar",
            position: "left",
            label: "Docs",
          },
          {
            to: "/tutorials/getting-started",
            label: "Tutorials",
            position: "left",
          },
          {
            to: "/reference/",
            label: "Reference",
            position: "left",
          },
          {
            href: "https://www.hook0.com/blog",
            label: "Blog",
            position: "right",
          },
          {
            href: "https://gitlab.com/hook0/hook0",
            label: "GitLab",
            position: "right",
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
