// @ts-check
// `@type` JSDoc annotations allow editor autocompletion and type checking
// (when paired with `@ts-check`).
// There are various equivalent ways to declare your Docusaurus config.
// See: https://docusaurus.io/docs/api/docusaurus-config

import { themes as prismThemes } from "prism-react-renderer";

/** @type {import('@docusaurus/types').Config} */
const config = {
  title: "Hook0 Documentation",
  tagline:
    "Comprehensive documentation for Hook0 - The open-source webhook server",
  favicon: "favicon.ico",

  // Set the production url of your site here
  url: process.env.SITE_URL || "https://hook0.gitlab.io",
  // Set the /<baseUrl>/ pathname under which your site is served
  baseUrl: process.env.BASE_URL || "/",

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
            url: "/hook0-api.json",
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
      footer: {
        style: "dark",
        links: [
          {
            title: "Learn",
            items: [
              {
                label: "Introduction",
                to: "/",
              },
              {
                label: "Getting Started",
                to: "/tutorials/getting-started",
              },
              {
                label: "First Webhook",
                to: "/tutorials/first-webhook-integration",
              },
              {
                label: "Self-hosting Guide",
                to: "/tutorials/self-hosting-docker",
              },
            ],
          },
          {
            title: "Guides",
            items: [
              {
                label: "Debug Webhooks",
                to: "/how-to-guides/debug-failed-webhooks",
              },
              {
                label: "Security Best Practices",
                to: "/how-to-guides/secure-webhook-endpoints",
              },
              {
                label: "GitLab Webhook Migration",
                to: "/how-to-guides/gitlab-webhook-migration",
              },
            ],
          },
          {
            title: "Reference",
            items: [
              {
                label: "API Documentation",
                to: "/openapi/intro",
              },
              {
                label: "Configuration",
                to: "/reference/configuration",
              },
              {
                label: "JavaScript SDK",
                to: "/reference/sdk/javascript",
              },
              {
                label: "Error Codes",
                to: "/reference/error-codes",
              },
            ],
          },
          {
            title: "Community",
            items: [
              {
                label: "GitLab",
                href: "https://gitlab.com/hook0/hook0",
              },
              {
                label: "Discord",
                href: "https://www.hook0.com/community",
              },
              {
                label: "Twitter",
                href: "https://twitter.com/hook0_",
              },
              {
                label: "Stack Overflow",
                href: "https://stackoverflow.com/questions/tagged/hook0",
              },
            ],
          },
          {
            title: "Resources",
            items: [
              {
                label: "Blog",
                href: "https://www.hook0.com/blog",
              },
              {
                label: "Changelog",
                href: "https://gitlab.com/hook0/hook0/-/releases",
              },
              {
                label: "Roadmap",
                href: "https://gitlab.com/hook0/hook0/-/boards",
              },
              {
                label: "Contributing",
                href: "https://gitlab.com/hook0/hook0/-/blob/main/CONTRIBUTING.md",
              },
            ],
          },
          {
            title: "More",
            items: [
              {
                label: "Docker Hub",
                href: "https://hub.docker.com/r/hook0/hook0",
              },
              {
                label: "Status Page",
                href: "https://status.hook0.com",
              },
              {
                label: "Support",
                href: "mailto:support@hook0.com",
              },
              {
                label: "License",
                href: "https://gitlab.com/hook0/hook0/-/blob/main/LICENSE",
              },
            ],
          },
        ],
        copyright: `Copyright © ${new Date().getFullYear()} Hook0 SAS. Built with Docusaurus.`,
      },
      prism: {
        theme: prismThemes.github,
        darkTheme: prismThemes.nightOwl,
        additionalLanguages: ["rust", "bash", "json", "yaml", "toml"],
      },
      announcementBar: {
        id: "hook0_v2",
        content:
          '🚀 Hook0 v2.0 is now available! Check out the <a href="/tutorials/getting-started">updated getting started guide</a> and <a href="https://gitlab.com/hook0/hook0/-/releases">release notes</a>.',
        backgroundColor: "#1890ff",
        textColor: "#ffffff",
        isCloseable: true,
      },
      colorMode: {
        defaultMode: "light",
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
