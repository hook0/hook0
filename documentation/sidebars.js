/**
 * Creating a sidebar enables you to:
 - create an ordered group of docs
 - render a sidebar for each doc of that group
 - provide next/previous navigation

 The sidebars can be generated from the filesystem, or explicitly defined here.

 Create as many sidebars as you want.
 */

// @ts-check

// Try to load the generated API sidebar
let apiSidebar = [];
try {
  const generatedSidebar = require('./api/sidebar.ts');
  
  // The generated sidebar exports default which is the array directly
  const sidebarData = generatedSidebar.default || generatedSidebar;
  
  if (Array.isArray(sidebarData)) {
    apiSidebar = sidebarData;
  }
} catch (e) {
  // API sidebar not generated yet
  console.log('API sidebar will be generated during build');
}

/** @type {import('@docusaurus/plugin-content-docs').SidebarsConfig} */
const sidebars = {
  tutorialSidebar: [
    {
      type: 'doc',
      id: 'index',
      label: 'Home',
    },
    {
      type: 'category',
      label: 'Getting Started',
      items: [
        'explanation/what-is-hook0',
        'tutorials/getting-started',
      ],
    },
    {
      type: 'category',
      label: 'Tutorials',
      link: {
        type: 'doc',
        id: 'tutorials/index',
      },
      items: [
        'tutorials/getting-started',
        'tutorials/first-webhook-integration',
        'tutorials/event-types-subscriptions',
        'tutorials/webhook-authentication',
        'tutorials/self-hosting-docker',
      ],
    },
    {
      type: 'category',
      label: 'How-to Guides',
      link: {
        type: 'doc',
        id: 'how-to-guides/index',
      },
      items: [
        'how-to-guides/debug-failed-webhooks',
        'how-to-guides/secure-webhook-endpoints',
        'how-to-guides/gitlab-webhook-migration',
      ],
    },
    {
      type: 'category',
      label: 'Reference',
      link: {
        type: 'doc',
        id: 'reference/index',
      },
      items: [
        {
          type: 'category',
          label: 'API Documentation',
          items: apiSidebar.length > 0 ? [
            {
              type: 'doc',
              id: 'openapi/intro',
              label: 'Overview',
            },
            ...apiSidebar
          ] : [
            {
              type: 'doc',
              id: 'openapi/intro',
              label: 'API Reference',
            },
          ],
        },
        'reference/event-schemas',
        'reference/configuration',
        'reference/error-codes',
        {
          type: 'category',
          label: 'SDKs',
          link: {
            type: 'doc',
            id: 'reference/sdk/index',
          },
          items: [
            'reference/sdk/javascript',
            'reference/sdk/rust',
          ],
        },
      ],
    },
    {
      type: 'category',
      label: 'Explanation',
      link: {
        type: 'doc',
        id: 'explanation/index',
      },
      items: [
        'explanation/what-is-hook0',
        'explanation/hook0-architecture',
        'explanation/event-processing',
        'explanation/security-model',
      ],
    },
  ],
};

export default sidebars;