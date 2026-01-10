/**
 * Creating a sidebar enables you to:
 - create an ordered group of docs
 - render a sidebar for each doc of that group
 - provide next/previous navigation

 The sidebars can be generated from the filesystem, or explicitly defined here.

 Create as many sidebars as you want.
 */

// @ts-check

/** @type {import('@docusaurus/plugin-content-docs').SidebarsConfig} */
const sidebars = {
  tutorialSidebar: [
    {
      type: "doc",
      id: "index",
      label: "Home",
    },
    {
      type: "category",
      label: "Getting Started",
      items: ["explanation/what-is-hook0", "tutorials/getting-started"],
    },
    {
      type: "category",
      label: "Concepts",
      link: {
        type: "doc",
        id: "concepts/index",
      },
      items: [
        "concepts/events",
        "concepts/subscriptions",
        "concepts/event-types",
        "concepts/metadata",
      ],
    },
    {
      type: "category",
      label: "Tutorials",
      link: {
        type: "doc",
        id: "tutorials/index",
      },
      items: [
        "tutorials/first-webhook-integration",
        "tutorials/event-types-subscriptions",
        "tutorials/webhook-authentication",
      ],
    },
    {
      type: "category",
      label: "How-to Guides",
      link: {
        type: "doc",
        id: "how-to-guides/index",
      },
      items: [
        "how-to-guides/debug-failed-webhooks",
        "how-to-guides/secure-webhook-endpoints",
        "how-to-guides/monitor-webhook-performance",
        "how-to-guides/client-error-handling",
        "how-to-guides/multi-tenant-architecture",
      ],
    },
    {
      type: "category",
      label: "Reference",
      link: {
        type: "doc",
        id: "reference/index",
      },
      items: [
        "openapi/intro",
        {
          type: "link",
          label: "API Reference",
          href: "/api",
        },
        "reference/event-schemas",
        "reference/configuration",
        "reference/error-codes",
        "reference/mcp",
        {
          type: "category",
          label: "SDKs",
          link: {
            type: "doc",
            id: "reference/sdk/index",
          },
          items: ["reference/sdk/javascript", "reference/sdk/rust"],
        },
      ],
    },
    {
      type: "category",
      label: "Explanation",
      link: {
        type: "doc",
        id: "explanation/index",
      },
      items: [
        "explanation/hook0-architecture",
        "explanation/event-processing",
        "explanation/security-model",
      ],
    },
    {
      type: "category",
      label: "Self-Hosting",
      link: {
        type: "doc",
        id: "self-hosting/index",
      },
      items: [
        "self-hosting/bare-metal",
        "self-hosting/docker-compose",
        "self-hosting/kubernetes",
        "self-hosting/aws",
        "self-hosting/master-api-key",
      ],
    },
    {
      type: "category",
      label: "Hook0 Cloud",
      link: {
        type: "doc",
        id: "hook0-cloud/index",
      },
      items: [
        {
          type: "category",
          label: "Policies",
          items: [
            "hook0-cloud/access-control-policy",
            "hook0-cloud/backup-policy",
            "hook0-cloud/change-management-process",
            "hook0-cloud/code-of-conduct",
            "hook0-cloud/cryptography-policy",
            "hook0-cloud/information-classification-policy",
            "hook0-cloud/information-retention-policy",
            "hook0-cloud/logging-policy",
            "hook0-cloud/mobile-device-policy",
            "hook0-cloud/password-policy",
            "hook0-cloud/penetration-testing-policy",
            "hook0-cloud/physical-security-policy",
            "hook0-cloud/privacy-policy",
            "hook0-cloud/responsible-disclosure-policy",
            "hook0-cloud/secure-development-policy",
            "hook0-cloud/secure-engineering-policy",
            "hook0-cloud/supplier-policy",
            "hook0-cloud/testing-policy",
          ],
        },
        {
          type: "category",
          label: "Supporting Documents",
          items: [
            "hook0-cloud/business-continuity-disaster-recovery",
            "hook0-cloud/statement-of-applicability",
          ],
        },
      ],
    },
    {
      type: "category",
      label: "Resources",
      link: {
        type: "doc",
        id: "resources/index",
      },
      items: [
        "resources/changelog",
        {
          type: "category",
          label: "Comparisons",
          link: {
            type: "doc",
            id: "comparisons/index",
          },
          items: [
            "comparisons/svix-vs-hook0",
            "comparisons/hookdeck-vs-hook0",
            "comparisons/hostedhooks-vs-hook0",
          ],
        },
        "resources/security",
        "resources/pricing-billing",
        "resources/support",
      ],
    },
  ],
};

export default sidebars;
