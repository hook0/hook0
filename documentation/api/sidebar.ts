import type { SidebarsConfig } from "@docusaurus/plugin-content-docs";

const sidebar: SidebarsConfig = {
  apisidebar: [
    {
      type: "doc",
      id: "api/hook-0-api",
    },
    {
      type: "category",
      label: "Applications Management",
      items: [
        {
          type: "doc",
          id: "api/application-secrets-read",
          label: "List application secrets",
          className: "api-method get",
        },
        {
          type: "doc",
          id: "api/application-secrets-create",
          label: "Create a new application secret",
          className: "api-method post",
        },
        {
          type: "doc",
          id: "api/application-secrets-update",
          label: "Update an application secret",
          className: "api-method put",
        },
        {
          type: "doc",
          id: "api/application-secrets-delete",
          label: "Delete an application secret",
          className: "api-method delete",
        },
        {
          type: "doc",
          id: "api/applications-list",
          label: "List applications",
          className: "api-method get",
        },
        {
          type: "doc",
          id: "api/applications-create",
          label: "Create a new application",
          className: "api-method post",
        },
        {
          type: "doc",
          id: "api/applications-get",
          label: "Get an application by its ID",
          className: "api-method get",
        },
        {
          type: "doc",
          id: "api/applications-update",
          label: "Edit an application",
          className: "api-method put",
        },
        {
          type: "doc",
          id: "api/applications-delete",
          label: "Delete an application",
          className: "api-method delete",
        },
      ],
    },
    {
      type: "category",
      label: "User Authentication",
      items: [
        {
          type: "doc",
          id: "api/auth-begin-reset-password",
          label: "Begin reset password",
          className: "api-method post",
        },
        {
          type: "doc",
          id: "api/auth-login",
          label: "Login",
          className: "api-method post",
        },
        {
          type: "doc",
          id: "api/auth-logout",
          label: "Logout",
          className: "api-method post",
        },
        {
          type: "doc",
          id: "api/auth-change-password",
          label: "Change password",
          className: "api-method post",
        },
        {
          type: "doc",
          id: "api/auth-refresh",
          label: "Refresh access token",
          className: "api-method post",
        },
        {
          type: "doc",
          id: "api/auth-reset-password",
          label: "Reset password",
          className: "api-method post",
        },
        {
          type: "doc",
          id: "api/auth-verify-email",
          label: "Email verification",
          className: "api-method post",
        },
      ],
    },
    {
      type: "category",
      label: "Hook0",
      items: [
        {
          type: "doc",
          id: "api/errors-list",
          label: "List errors",
          className: "api-method get",
        },
        {
          type: "doc",
          id: "api/instance-health",
          label: "Check instance health",
          className: "api-method get",
        },
        {
          type: "doc",
          id: "api/instance-get",
          label: "Get instance configuration",
          className: "api-method get",
        },
        {
          type: "doc",
          id: "api/quotas-get",
          label: "Get quotas",
          className: "api-method get",
        },
      ],
    },
    {
      type: "category",
      label: "Events Management",
      items: [
        {
          type: "doc",
          id: "api/events-ingest",
          label: "Ingest an event",
          className: "api-method post",
        },
        {
          type: "doc",
          id: "api/event-types-list",
          label: "List event types",
          className: "api-method get",
        },
        {
          type: "doc",
          id: "api/event-types-create",
          label: "Create a new event type",
          className: "api-method post",
        },
        {
          type: "doc",
          id: "api/event-types-get",
          label: "Get an event type by its name",
          className: "api-method get",
        },
        {
          type: "doc",
          id: "api/event-types-delete",
          label: "Delete an event type",
          className: "api-method delete",
        },
        {
          type: "doc",
          id: "api/events-list",
          label: "List latest events",
          className: "api-method get",
        },
        {
          type: "doc",
          id: "api/events-get",
          label: "Get an event",
          className: "api-method get",
        },
        {
          type: "doc",
          id: "api/events-replay",
          label: "Replay an event",
          className: "api-method post",
        },
        {
          type: "doc",
          id: "api/payload-content-types-list",
          label: "List supported event payload content types",
          className: "api-method get",
        },
      ],
    },
    {
      type: "category",
      label: "Organizations Management",
      items: [
        {
          type: "doc",
          id: "api/organizations-list",
          label: "List organizations",
          className: "api-method get",
        },
        {
          type: "doc",
          id: "api/organizations-create",
          label: "Create an organization",
          className: "api-method post",
        },
        {
          type: "doc",
          id: "api/organizations-get",
          label: "Get organization's info by its ID",
          className: "api-method get",
        },
        {
          type: "doc",
          id: "api/organizations-edit",
          label: "Edit an organization",
          className: "api-method put",
        },
        {
          type: "doc",
          id: "api/organizations-delete",
          label: "Delete an organization",
          className: "api-method delete",
        },
        {
          type: "doc",
          id: "api/organizations-edit-role",
          label: "Edit a user's role in an organization",
          className: "api-method put",
        },
        {
          type: "doc",
          id: "api/organizations-invite",
          label: "Invite a user to an organization",
          className: "api-method post",
        },
        {
          type: "doc",
          id: "api/organizations-revoke",
          label: "Revoke a user's access to an organization",
          className: "api-method delete",
        },
        {
          type: "doc",
          id: "api/register",
          label: "Create a new user account and its own personal organization",
          className: "api-method post",
        },
      ],
    },
    {
      type: "category",
      label: "Subscriptions Management",
      items: [
        {
          type: "doc",
          id: "api/request-attempts-read",
          label: "List request attempts",
          className: "api-method get",
        },
        {
          type: "doc",
          id: "api/response-get",
          label: "Get a response by its ID",
          className: "api-method get",
        },
        {
          type: "doc",
          id: "api/subscriptions-list",
          label: "List subscriptions",
          className: "api-method get",
        },
        {
          type: "doc",
          id: "api/subscriptions-create",
          label: "Create a new subscription",
          className: "api-method post",
        },
        {
          type: "doc",
          id: "api/subscriptions-get",
          label: "Get a subscription by its id",
          className: "api-method get",
        },
        {
          type: "doc",
          id: "api/subscriptions-update",
          label: "Update a subscription",
          className: "api-method put",
        },
        {
          type: "doc",
          id: "api/subscriptions-delete",
          label: "Delete a subscription",
          className: "api-method delete",
        },
      ],
    },
    {
      type: "category",
      label: "Service Tokens Management",
      items: [
        {
          type: "doc",
          id: "api/service-token-list",
          label: "List service tokens",
          className: "api-method get",
        },
        {
          type: "doc",
          id: "api/service-token-create",
          label: "Create a new service token",
          className: "api-method post",
        },
        {
          type: "doc",
          id: "api/service-token-get",
          label: "Get a service token",
          className: "api-method get",
        },
        {
          type: "doc",
          id: "api/service-token-edit",
          label: "Edit a service token",
          className: "api-method put",
        },
        {
          type: "doc",
          id: "api/service-token-delete",
          label: "Delete a service token",
          className: "api-method delete",
        },
      ],
    },
  ],
};

export default sidebar.apisidebar;
