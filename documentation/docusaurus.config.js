// @ts-check
// `@type` JSDoc annotations allow editor autocompletion and type checking
// (when paired with `@ts-check`).
// There are various equivalent ways to declare your Docusaurus config.
// See: https://docusaurus.io/docs/api/docusaurus-config

import { themes as prismThemes } from "prism-react-renderer";
import poimandresTheme from "./src/prism/poimandres.js";
import path from "path";
import fs from "fs/promises";
import { fileURLToPath } from "url";
import sidebars from "./sidebars.js";
import { buildLlmsTxt } from "./scripts/generate-llms-txt.js";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// Opening line of /llms.txt. An assistant reads it before it reads anything
// else, so it says what Hook0 does and under what licence, in one breath.
const LLMS_TXT_SUMMARY =
  "Hook0 sends webhooks on your behalf: one API call per event, HMAC signatures, " +
  "retries with a dead letter queue, delivery logs, and a subscriber portal your " +
  "customers manage themselves. Open-source (SSPL-1.0) — self-host it, or use the " +
  "cloud hosted in the EU (France). Every page published here is listed below.";

const url = process.env.DOCUMENTATION_URL || "http://localhost:3000/";
const baseUrl = process.env.DOCUMENTATION_BASE_URL || "/";

/** @type {import('@docusaurus/types').Config} */
const config = {
  title: "Hook0 Documentation",
  tagline:
    "Comprehensive documentation for Hook0 - The open-source webhook server",
  favicon: "favicon.ico",

  future: {
    v4: {
      removeLegacyPostBuildHeadAttribute: true,
      fasterByDefault: true,
    },
  },

  // Set the production url of your site here
  url,
  // Set the /<baseUrl>/ pathname under which your site is served
  baseUrl,

  // GitLab pages deployment config.
  organizationName: "hook0",
  projectName: "hook0",

  trailingSlash: false,
  onBrokenLinks: "throw",

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
          editUrl: "https://gitlab.com/hook0/hook0/-/tree/master/documentation/",
          exclude: [
            "**/node_modules/**",
            "**/build/**",
            "**/.docusaurus/**",
            "**/scripts/**",
            "**/CLAUDE.md",
            // The harnesses the SDK examples are assembled into, and the note explaining them.
            // They live beside the pages so that whoever edits a snippet can see what it stands
            // on, and they are addressed to that person rather than to a reader of the product.
            "**/reference/sdk/examples/**",
          ],
        },
        sitemap: {
          ignorePatterns: ['/search', '/search/**', '/self-hosting/_dev-only-warning', '/CLAUDE'],
        },
        blog: false,
        theme: {
          customCss: "./src/css/custom.css",
        },
      }),
    ],
  ],

  plugins: [
    // Inline build-time env vars pulled in via @shared/website-data (website/data.js)
    // so they don't leak `process.env.*` into the browser bundle. Without this, the
    // literal `process.env.LOCAL_PREVIEW_URL` reaches the client and throws
    // "ReferenceError: process is not defined" (most visibly on the client-rendered /api page).
    // `currentBundler.instance` resolves to the active bundler (Rspack under `future.faster`).
    () => ({
      name: "define-shared-env",
      configureWebpack: (_config, _isServer, utils) => ({
        plugins: [
          new utils.currentBundler.instance.DefinePlugin({
            "process.env.LOCAL_PREVIEW_URL": JSON.stringify(
              process.env.LOCAL_PREVIEW_URL || ""
            ),
          }),
        ],
      }),
    }),
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
    // Publishes /llms.txt: the index an assistant reads before answering a
    // question about Hook0. www.hook0.com already serves one and points here,
    // but the documentation — where the answers actually live — served none.
    // Built from the pages Docusaurus has just emitted and ordered by the same
    // sidebar as the navigation, so a page added tomorrow appears on its own.
    () => ({
      name: "generate-llms-txt",
      async postBuild({ siteConfig, outDir, plugins }) {
        const docsPlugin = plugins.find(
          (plugin) => plugin.name === "docusaurus-plugin-content-docs"
        );
        if (!docsPlugin?.content?.loadedVersions?.length) {
          throw new Error(
            "generate-llms-txt: no loaded docs version to index. The docs " +
              "plugin moved or changed shape; llms.txt would be published empty."
          );
        }

        const docs = docsPlugin.content.loadedVersions
          .flatMap((version) => version.docs)
          .map(({ id, title, description, permalink }) => ({
            id,
            title,
            description,
            permalink,
          }));

        await fs.writeFile(
          path.join(outDir, "llms.txt"),
          buildLlmsTxt({
            siteUrl: siteConfig.url + siteConfig.baseUrl,
            title: siteConfig.title,
            summary: LLMS_TXT_SUMMARY,
            sidebarItems: sidebars.tutorialSidebar,
            docs,
          }),
          "utf8"
        );
      },
    }),
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
    hooks: {
      onBrokenMarkdownLinks: "throw",
    },
  },

  // Client modules for Matomo advanced tracking
  clientModules: [
    require.resolve("./src/mermaid/theme-switcher.js"),
    ...(process.env.DOCUMENTATION_MATOMO_URL
      ? [require.resolve("./src/matomo/tracking.js")]
      : []),
  ],

  // Matomo analytics script (injected in <head>)
  // Uses same configuration as website (piwik.php/piwik.js)
  // Site ID 3 is for documentation
  headTags:
    process.env.DOCUMENTATION_MATOMO_URL && process.env.DOCUMENTATION_MATOMO_SITE_ID
      ? [
          {
            tagName: "script",
            attributes: {},
            innerHTML: `
              var _paq = window._paq = window._paq || [];
              /* Custom dimension 1 = Content Type (Diataxis) - set by tracking.js */
              _paq.push(['enableLinkTracking']);
              _paq.push(['setLinkTrackingTimer', 500]);
              (function() {
                var u="${process.env.DOCUMENTATION_MATOMO_URL}";
                _paq.push(['setTrackerUrl', u+'piwik.php']);
                _paq.push(['setSiteId', '${process.env.DOCUMENTATION_MATOMO_SITE_ID}']);
                var d=document, g=d.createElement('script'), s=d.getElementsByTagName('script')[0];
                g.async=true; g.src=u+'piwik.js'; s.parentNode.insertBefore(g,s);
              })();
            `,
          },
        ]
      : [],

  themeConfig:
    /** @type {import('@docusaurus/preset-classic').ThemeConfig} */
    ({
      // Social card for og:image / twitter:image. An absolute URL hosted on the
      // main site rather than a local file: img/hook0-social-card.jpg never
      // existed in the repo (every docs page emitted a broken preview), and a
      // raster committed under this static/ dir would ship as an LFS pointer,
      // since the docs build does no LFS smudge. website serves it reliably.
      // This is the doc-specific 1200x630 card, next to the site's own one.
      image: "https://www.hook0.com/img/hook0-doc-social-card.png",
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
            href: "https://play.hook0.com",
            label: "Play",
            position: "right",
            className: "navbar__item--no-external-icon",
          },
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
            href: "https://app.hook0.com/register",
            label: "Get Started →",
            position: "right",
            className: "navbar__item--primary navbar__item--no-external-icon",
          },
        ],
      },
      // Footer is handled by custom component in src/theme/Footer
      prism: {
        theme: prismThemes.github,
        darkTheme: poimandresTheme,
        // One entry per language an SDK page shows code in; a language absent here
        // renders as plain text.
        additionalLanguages: [
          "rust",
          "bash",
          "json",
          "yaml",
          "toml",
          "python",
          "go",
          "ruby",
          "php",
          "csharp",
          "java",
          "kotlin",
          "lua",
          "zig",
        ],
      },
      // announcementBar: {
      //   id: "hook0_v2",
      //   content:
      //     '🚀 Hook0 v2.0 is now available! Check out the <a href="/tutorials/getting-started">updated getting started guide</a> and <a href="https://github.com/hook0/hook0/releases">release notes</a>.',
      //   backgroundColor: "#4ade80",
      //   textColor: "#ffffff",
      //   isCloseable: true,
      // },
      colorMode: {
        defaultMode: "dark",
        disableSwitch: false,
        respectPrefersColorScheme: true,
      },
      mermaid: {
        theme: {
          light: 'base',
          dark: 'dark',
        },
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
