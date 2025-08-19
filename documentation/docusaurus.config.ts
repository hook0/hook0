import type { Config } from '@docusaurus/types';
import type * as Preset from '@docusaurus/preset-classic';
import type * as SearchLocalPlugin from '@easyops-cn/docusaurus-search-local';
import type * as OpenApiPlugin from 'docusaurus-plugin-openapi-docs';

import { themes as prismThemes } from 'prism-react-renderer';

const config: Config = {
  title: 'Hook0 Documentation',
  tagline: 'Comprehensive documentation for Hook0 - The open-source webhook server',
  favicon: 'favicon.ico',

  // Set the production url of your site here
  url: 'https://hook0.gitlab.io',
  // Set the /<baseUrl>/ pathname under which your site is served
  baseUrl: '/hook0/',

  ///////////////////////////////////////////////////////////////
  noIndex: true, // Do not remove until this goes into production
  ///////////////////////////////////////////////////////////////

  // GitLab pages deployment config.
  organizationName: 'hook0',
  projectName: 'hook0',

  onBrokenLinks: 'throw',
  onBrokenMarkdownLinks: 'warn',

  staticDirectories: ['static'],

  future: {
    experimental_faster: {
      rspackBundler: true,
      rspackPersistentCache: true,
      ssgWorkerThreads: true,
    },
    v4: {
      removeLegacyPostBuildHeadAttribute: true,
    },
  },

  // Even if you don't use internationalization, you can use this field to set
  // useful metadata like html lang. For example, if your site is Chinese, you
  // may want to replace "en" with "zh-Hans".
  i18n: {
    defaultLocale: 'en',
    locales: ['en'],
  },

  presets: [
    [
      'classic',
      {
        docs: {
          path: '.',
          sidebarPath: './sidebars.ts',
          routeBasePath: '/',
          editUrl: 'https://gitlab.com/hook0/hook0/-/tree/master/documentation/',
          exclude: ['**/node_modules/**', '**/build/**', '**/.docusaurus/**'],
          docItemComponent: '@theme/ApiItem',
        },
        blog: false,
        theme: {
          // customCss: './src/css/custom.css',
        },
      } satisfies Preset.Options,
    ],
  ],

  plugins: [
    [
      'docusaurus-plugin-openapi-docs',
      {
        id: 'openapi',
        docsPluginId: 'classic',
        config: {
          petstore: {
            specPath: 'https://app.hook0.com/api/v1/swagger.json',
            outputDir: './api',
            sidebarOptions: {
              groupPathsBy: 'tag',
            },
          } satisfies OpenApiPlugin.Options,
        },
      },
    ],
  ],

  themes: [
    '@docusaurus/theme-mermaid',
    [
      '@easyops-cn/docusaurus-search-local',
      {
        hashed: true,
        language: ['en', 'fr'],
        highlightSearchTermsOnTargetPage: true,
        explicitSearchResultPath: true,
        forceIgnoreNoIndex: true,
      } satisfies SearchLocalPlugin.PluginOptions,
    ],
    'docusaurus-theme-openapi-docs',
  ],

  markdown: {
    mermaid: true,
  },

  themeConfig: {
    // Replace with your project's social card
    image: 'img/hook0-social-card.jpg',
    navbar: {
      title: 'Hook0',
      logo: {
        alt: 'Hook0 Logo',
        src: 'logo.svg',
      },
      items: [
        {
          type: 'docSidebar',
          sidebarId: 'tutorialSidebar',
          position: 'left',
          label: 'Docs',
        },
        {
          to: '/tutorials/getting-started',
          label: 'Tutorials',
          position: 'left',
        },
        {
          to: '/reference/api-reference',
          label: 'API',
          position: 'left',
        },
        {
          to: '/reference/sdk/',
          label: 'SDKs',
          position: 'left',
        },
        {
          href: 'https://www.hook0.com/blog',
          label: 'Blog',
          position: 'right',
        },
        {
          href: 'https://gitlab.com/hook0/hook0',
          label: 'GitLab',
          position: 'right',
        },
      ],
    },
    footer: {
      style: 'dark',
      links: [
        {
          title: 'Learn',
          items: [
            {
              label: 'Introduction',
              to: '/',
            },
            {
              label: 'Getting Started',
              to: '/tutorials/getting-started',
            },
            {
              label: 'First Webhook',
              to: '/tutorials/first-webhook-integration',
            },
            {
              label: 'Self-hosting Guide',
              to: '/tutorials/self-hosting-docker',
            },
          ],
        },
        {
          title: 'Guides',
          items: [
            {
              label: 'Debug Webhooks',
              to: '/how-to-guides/debug-failed-webhooks',
            },
            {
              label: 'Security Best Practices',
              to: '/how-to-guides/secure-webhook-endpoints',
            },
            {
              label: 'GitLab Webhook Migration',
              to: '/how-to-guides/gitlab-webhook-migration',
            },
          ],
        },
        {
          title: 'Reference',
          items: [
            {
              label: 'API Documentation',
              to: '/reference/api-reference',
            },
            {
              label: 'Configuration',
              to: '/reference/configuration',
            },
            {
              label: 'JavaScript SDK',
              to: '/reference/sdk/javascript',
            },
            {
              label: 'Error Codes',
              to: '/reference/error-codes',
            },
          ],
        },
        {
          title: 'Community',
          items: [
            {
              label: 'GitLab',
              href: 'https://gitlab.com/hook0/hook0',
            },
            {
              label: 'Discord',
              href: 'https://www.hook0.com/community',
            },
            {
              label: 'Twitter',
              href: 'https://twitter.com/hook0_',
            },
            {
              label: 'Stack Overflow',
              href: 'https://stackoverflow.com/questions/tagged/hook0',
            },
          ],
        },
        {
          title: 'Resources',
          items: [
            {
              label: 'Blog',
              href: 'https://www.hook0.com/blog',
            },
            {
              label: 'Changelog',
              href: 'https://gitlab.com/hook0/hook0/-/releases',
            },
            {
              label: 'Roadmap',
              href: 'https://gitlab.com/hook0/hook0/-/boards',
            },
            {
              label: 'Contributing',
              href: 'https://gitlab.com/hook0/hook0/-/blob/master/CONTRIBUTING.md',
            },
          ],
        },
        {
          title: 'More',
          items: [
            {
              label: 'Docker Hub',
              href: 'https://hub.docker.com/r/hook0/hook0',
            },
            {
              label: 'Status Page',
              href: 'https://status.hook0.com',
            },
            {
              label: 'Support',
              href: 'mailto:support@hook0.com',
            },
            {
              label: 'License',
              href: 'https://gitlab.com/hook0/hook0/-/blob/master/LICENSE',
            },
          ],
        },
      ],
      copyright: `Copyright © ${new Date().getFullYear()} Hook0 SAS. Built with Docusaurus.`,
    },
    prism: {
      theme: prismThemes.github,
      darkTheme: prismThemes.dracula,
      additionalLanguages: ['rust', 'bash', 'json', 'yaml', 'toml'],
    },
    announcementBar: {
      id: 'hook0_v2',
      content:
        '🚀 Hook0 v2.0 is now available! Check out the <a href="/tutorials/getting-started">updated getting started guide</a> and <a href="https://gitlab.com/hook0/hook0/-/releases">release notes</a>.',
      backgroundColor: '#1890ff',
      textColor: '#ffffff',
      isCloseable: true,
    },
    colorMode: {
      defaultMode: 'light',
      disableSwitch: false,
      respectPrefersColorScheme: true,
    },
    docs: {
      sidebar: {
        hideable: true,
        autoCollapseCategories: true,
      },
    },
  } satisfies Preset.ThemeConfig,
};

export default config;
