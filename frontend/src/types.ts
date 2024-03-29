/**
 * This file was auto-generated by openapi-typescript.
 * Do not make direct changes to the file.
 */

export interface paths {
  '/api/v1/application_secrets/': {
    /** List application secrets */
    get: operations['applicationSecrets.read'];
    /** Create a new application secret */
    post: operations['applicationSecrets.create'];
  };
  '/api/v1/application_secrets/{application_secret_token}': {
    /** Update an application secret */
    put: operations['applicationSecrets.update'];
    /** Delete an application secret */
    delete: operations['applicationSecrets.delete'];
  };
  '/api/v1/applications/': {
    /** List applications */
    get: operations['applications.list'];
    /**
     * Create a new application
     * @description An application emit events that are consumed by customers through webhooks
     */
    post: operations['applications.create'];
  };
  '/api/v1/applications/{application_id}': {
    /**
     * Get an application by its ID
     * @description An application emit events that are consumed by customers through webhooks
     */
    get: operations['applications.get'];
    /**
     * Edit an application
     * @description Change the name of an application
     */
    put: operations['applications.update'];
    /**
     * Delete an application
     * @description Delete an application, further events won't be sent, active webhook subscriptions will also be deleted.
     */
    delete: operations['applications.delete'];
  };
  '/api/v1/errors/': {
    /**
     * List errors
     * @description List of every possible errors that Hook0 can return. Each error is in RFC7807 problem format.
     */
    get: operations['errors.list'];
  };
  '/api/v1/event/': {
    /** Ingest an event */
    post: operations['events.ingest'];
  };
  '/api/v1/event_types/': {
    /** List event types */
    get: operations['eventTypes.list'];
    /** Create a new event type */
    post: operations['eventTypes.create'];
  };
  '/api/v1/event_types/{event_type_name}': {
    /** Get an event type by its name */
    get: operations['eventTypes.get'];
    /** Delete an event type */
    delete: operations['eventTypes.delete'];
  };
  '/api/v1/events/': {
    /** List latest events */
    get: operations['events.list'];
  };
  '/api/v1/events/{event_id}': {
    /** Get an event */
    get: operations['events.get'];
  };
  '/api/v1/instance/': {
    /**
     * Get instance configuration
     * @description Get an object that shows how this instance is configured.
     */
    get: operations['instance.get'];
  };
  '/api/v1/organizations/': {
    /** List organizations */
    get: operations['organizations.list'];
    /**
     * Create an organization
     * @description Note that you will need to regenerate a JWT to be able to see/use the newly created organization.
     */
    post: operations['organizations.create'];
  };
  '/api/v1/organizations/{organization_id}/': {
    /** Get organization's info by its ID */
    get: operations['organizations.get'];
    /**
     * Edit an organization
     * @description Note that you will need to regenerate a JWT to be able to see the updated name of the organization.
     */
    put: operations['organizations.edit'];
    /**
     * Delete an organization
     * @description Note that you will need to regenerate a JWT to be able to make the deleted organization go away.
     */
    delete: operations['organizations.delete'];
  };
  '/api/v1/organizations/{organization_id}/invite': {
    /** Invite a user to an organization */
    put: operations['organizations.invite'];
    /** Revoke a user's access to an organization */
    delete: operations['organizations.revoke'];
  };
  '/api/v1/payload_content_types/': {
    /**
     * List supported event payload content types
     * @description List of every possible content types that can be used in event payloads.
     */
    get: operations['payload_content_types.list'];
  };
  '/api/v1/register/': {
    /** Create a new user account and a new organization */
    post: operations['register'];
  };
  '/api/v1/request_attempts/': {
    /** List request attempts */
    get: operations['requestAttempts.read'];
  };
  '/api/v1/responses/{response_id}': {
    /**
     * Get a response by its ID
     * @description A response is produced when a request attempt is processed
     */
    get: operations['response.get'];
  };
  '/api/v1/subscriptions/': {
    /**
     * List subscriptions
     * @description List all subscriptions created by customers against the application events
     */
    get: operations['subscriptions.list'];
    /**
     * Create a new subscription
     * @description A subscription let your customers subscribe to events. Events will be sent through the defined medium inside the subscription (e.g. HTTP POST request) as a webhook.
     */
    post: operations['subscriptions.create'];
  };
  '/api/v1/subscriptions/{subscription_id}': {
    /** Get a subscription by its id */
    get: operations['subscriptions.get'];
    /** Update a subscription */
    put: operations['subscriptions.update'];
    /** Delete a subscription */
    delete: operations['subscriptions.delete'];
  };
}

export type webhooks = Record<string, never>;

export interface components {
  schemas: {
    Application: {
      /** Format: uuid */
      application_id: string;
      name: string;
      /** Format: uuid */
      organization_id: string;
    };
    ApplicationInfo: {
      /** Format: uuid */
      application_id: string;
      name: string;
      /** Format: uuid */
      organization_id: string;
      quotas: {
        /** Format: int32 */
        days_of_events_retention_limit: number;
        /** Format: int32 */
        events_per_day_limit: number;
      };
    };
    ApplicationPost: {
      name: string;
      /** Format: uuid */
      organization_id: string;
    };
    ApplicationSecret: {
      /** Format: date-time */
      created_at: string;
      /** Format: date-time */
      deleted_at?: string;
      name?: string;
      /** Format: uuid */
      token: string;
    };
    ApplicationSecretPost: {
      /** Format: uuid */
      application_id: string;
      name?: string;
    };
    Event: {
      /** Format: uuid */
      application_secret_token: string;
      /** Format: uuid */
      event_id: string;
      event_type_name: string;
      ip: string;
      labels: Record<string, never>;
      metadata?: Record<string, never>;
      /** Format: date-time */
      occurred_at: string;
      payload_content_type: string;
      /** Format: date-time */
      received_at: string;
    };
    EventPost: {
      /** Format: uuid */
      application_id: string;
      /** Format: uuid */
      event_id: string;
      event_type: string;
      labels: Record<string, never>;
      metadata?: Record<string, never>;
      /** Format: date-time */
      occurred_at: string;
      payload: string;
      payload_content_type: string;
    };
    EventType: {
      event_type_name: string;
      resource_type_name: string;
      service_name: string;
      verb_name: string;
    };
    EventTypePost: {
      /** Format: uuid */
      application_id: string;
      resource_type: string;
      service: string;
      verb: string;
    };
    EventWithPayload: {
      /** Format: uuid */
      application_secret_token: string;
      /** Format: uuid */
      event_id: string;
      event_type_name: string;
      ip: string;
      labels: Record<string, never>;
      metadata?: Record<string, never>;
      /** Format: date-time */
      occurred_at: string;
      payload: string;
      payload_content_type: string;
      /** Format: date-time */
      received_at: string;
    };
    IngestedEvent: {
      /** Format: uuid */
      application_id: string;
      /** Format: uuid */
      event_id: string;
      /** Format: date-time */
      received_at: string;
    };
    InstanceConfig: {
      auto_db_migration: boolean;
      disable_registration: boolean;
      keycloak_front_client_id: string;
      keycloak_realm: string;
      keycloak_url: string;
    };
    Organization: {
      name: string;
      /** Format: uuid */
      organization_id: string;
      plan?: {
        label: string;
        name: string;
      };
      role: string;
    };
    OrganizationInfo: {
      name: string;
      /** Format: uuid */
      organization_id: string;
      plan?: {
        label: string;
        name: string;
      };
      quotas: {
        /** Format: int32 */
        applications_per_organization_limit: number;
        /** Format: int32 */
        days_of_events_retention_limit: number;
        /** Format: int32 */
        events_per_day_limit: number;
        /** Format: int32 */
        members_per_organization_limit: number;
      };
      users: {
        email: string;
        first_name: string;
        last_name: string;
        role: string;
        /** Format: uuid */
        user_id: string;
      }[];
    };
    OrganizationPost: {
      name: string;
    };
    Problem: {
      detail: string;
      id: string;
      /** Format: int32 */
      status: number;
      title: string;
    };
    Registration: {
      /** Format: uuid */
      organization_id: string;
      /** Format: uuid */
      user_id: string;
    };
    RegistrationPost: {
      email: string;
      first_name: string;
      last_name: string;
      organization_name: string;
      password: string;
    };
    RequestAttempt: {
      /** Format: date-time */
      created_at: string;
      /** Format: date-time */
      delay_until?: string;
      /** Format: uuid */
      event_id: string;
      /** Format: date-time */
      failed_at?: string;
      /** Format: date-time */
      picked_at?: string;
      /** Format: uuid */
      request_attempt_id: string;
      /** Format: uuid */
      response_id?: string;
      /** Format: int32 */
      retry_count: number;
      status: string;
      subscription: {
        description?: string;
        /** Format: uuid */
        subscription_id: string;
      };
      /** Format: date-time */
      succeeded_at?: string;
    };
    Response: {
      body?: string;
      /** Format: int32 */
      elapsed_time_ms?: number;
      headers?: Record<string, never>;
      /** Format: int32 */
      http_code?: number;
      response_error_name?: string;
      /** Format: uuid */
      response_id: string;
    };
    Revoke: {
      /** Format: uuid */
      user_id: string;
    };
    Subscription: {
      /** Format: uuid */
      application_id: string;
      /** Format: date-time */
      created_at: string;
      dedicated_workers: string[];
      description?: string;
      event_types: string[];
      is_enabled: boolean;
      label_key: string;
      label_value: string;
      metadata: Record<string, never>;
      /** Format: uuid */
      secret: string;
      /** Format: uuid */
      subscription_id: string;
      target: string;
    };
    SubscriptionPost: {
      /** Format: uuid */
      application_id: string;
      dedicated_workers?: string[];
      description?: string;
      event_types: string[];
      is_enabled: boolean;
      label_key: string;
      label_value: string;
      metadata?: Record<string, never>;
      target: string;
    };
    UserInvitation: {
      email: string;
      role: string;
    };
  };
  responses: never;
  parameters: never;
  requestBodies: never;
  headers: never;
  pathItems: never;
}

export type external = Record<string, never>;

export interface operations {
  /** List application secrets */
  'applicationSecrets.read': {
    parameters: {
      query: {
        application_id: string;
      };
    };
    responses: {
      /** @description OK */
      200: {
        content: {
          'application/json': components['schemas']['ApplicationSecret'][];
        };
      };
      /** @description Bad Request */
      400: never;
      /** @description Forbidden */
      403: never;
      /** @description Not Found */
      404: never;
      /** @description Conflict */
      409: never;
      /** @description Internal Server Error */
      500: never;
    };
  };
  /** Create a new application secret */
  'applicationSecrets.create': {
    requestBody: {
      content: {
        'application/json': components['schemas']['ApplicationSecretPost'];
      };
    };
    responses: {
      /** @description Created */
      201: {
        content: {
          'application/json': components['schemas']['ApplicationSecret'];
        };
      };
      /** @description Bad Request */
      400: never;
      /** @description Forbidden */
      403: never;
      /** @description Not Found */
      404: never;
      /** @description Conflict */
      409: never;
      /** @description Internal Server Error */
      500: never;
    };
  };
  /** Update an application secret */
  'applicationSecrets.update': {
    parameters: {
      path: {
        application_secret_token: string;
      };
    };
    requestBody: {
      content: {
        'application/json': components['schemas']['ApplicationSecretPost'];
      };
    };
    responses: {
      /** @description OK */
      200: {
        content: {
          'application/json': components['schemas']['ApplicationSecret'];
        };
      };
      /** @description Bad Request */
      400: never;
      /** @description Forbidden */
      403: never;
      /** @description Not Found */
      404: never;
      /** @description Conflict */
      409: never;
      /** @description Internal Server Error */
      500: never;
    };
  };
  /** Delete an application secret */
  'applicationSecrets.delete': {
    parameters: {
      query: {
        application_id: string;
      };
      path: {
        application_secret_token: string;
      };
    };
    responses: {
      /** @description No Content */
      204: never;
      /** @description Bad Request */
      400: never;
      /** @description Forbidden */
      403: never;
      /** @description Not Found */
      404: never;
      /** @description Conflict */
      409: never;
      /** @description Internal Server Error */
      500: never;
    };
  };
  /** List applications */
  'applications.list': {
    parameters: {
      query: {
        organization_id: string;
      };
    };
    responses: {
      /** @description OK */
      200: {
        content: {
          'application/json': components['schemas']['Application'][];
        };
      };
      /** @description Bad Request */
      400: never;
      /** @description Forbidden */
      403: never;
      /** @description Not Found */
      404: never;
      /** @description Conflict */
      409: never;
      /** @description Internal Server Error */
      500: never;
    };
  };
  /**
   * Create a new application
   * @description An application emit events that are consumed by customers through webhooks
   */
  'applications.create': {
    requestBody: {
      content: {
        'application/json': components['schemas']['ApplicationPost'];
      };
    };
    responses: {
      /** @description Created */
      201: {
        content: {
          'application/json': components['schemas']['Application'];
        };
      };
      /** @description Bad Request */
      400: never;
      /** @description Forbidden */
      403: never;
      /** @description Not Found */
      404: never;
      /** @description Conflict */
      409: never;
      /** @description Internal Server Error */
      500: never;
    };
  };
  /**
   * Get an application by its ID
   * @description An application emit events that are consumed by customers through webhooks
   */
  'applications.get': {
    parameters: {
      path: {
        application_id: string;
      };
    };
    responses: {
      /** @description OK */
      200: {
        content: {
          'application/json': components['schemas']['ApplicationInfo'];
        };
      };
      /** @description Bad Request */
      400: never;
      /** @description Forbidden */
      403: never;
      /** @description Not Found */
      404: never;
      /** @description Conflict */
      409: never;
      /** @description Internal Server Error */
      500: never;
    };
  };
  /**
   * Edit an application
   * @description Change the name of an application
   */
  'applications.update': {
    parameters: {
      path: {
        application_id: string;
      };
    };
    requestBody: {
      content: {
        'application/json': components['schemas']['ApplicationPost'];
      };
    };
    responses: {
      /** @description OK */
      200: {
        content: {
          'application/json': components['schemas']['Application'];
        };
      };
      /** @description Bad Request */
      400: never;
      /** @description Forbidden */
      403: never;
      /** @description Not Found */
      404: never;
      /** @description Conflict */
      409: never;
      /** @description Internal Server Error */
      500: never;
    };
  };
  /**
   * Delete an application
   * @description Delete an application, further events won't be sent, active webhook subscriptions will also be deleted.
   */
  'applications.delete': {
    parameters: {
      path: {
        application_id: string;
      };
    };
    responses: {
      /** @description No Content */
      204: never;
      /** @description Bad Request */
      400: never;
      /** @description Forbidden */
      403: never;
      /** @description Not Found */
      404: never;
      /** @description Conflict */
      409: never;
      /** @description Internal Server Error */
      500: never;
    };
  };
  /**
   * List errors
   * @description List of every possible errors that Hook0 can return. Each error is in RFC7807 problem format.
   */
  'errors.list': {
    responses: {
      /** @description OK */
      200: {
        content: {
          'application/json': components['schemas']['Problem'][];
        };
      };
      /** @description Bad Request */
      400: never;
      /** @description Forbidden */
      403: never;
      /** @description Not Found */
      404: never;
      /** @description Conflict */
      409: never;
      /** @description Internal Server Error */
      500: never;
    };
  };
  /** Ingest an event */
  'events.ingest': {
    requestBody: {
      content: {
        'application/json': components['schemas']['EventPost'];
      };
    };
    responses: {
      /** @description Created */
      201: {
        content: {
          'application/json': components['schemas']['IngestedEvent'];
        };
      };
      /** @description Bad Request */
      400: never;
      /** @description Forbidden */
      403: never;
      /** @description Not Found */
      404: never;
      /** @description Conflict */
      409: never;
      /** @description Internal Server Error */
      500: never;
    };
  };
  /** List event types */
  'eventTypes.list': {
    parameters: {
      query: {
        application_id: string;
      };
    };
    responses: {
      /** @description OK */
      200: {
        content: {
          'application/json': components['schemas']['EventType'][];
        };
      };
      /** @description Bad Request */
      400: never;
      /** @description Forbidden */
      403: never;
      /** @description Not Found */
      404: never;
      /** @description Conflict */
      409: never;
      /** @description Internal Server Error */
      500: never;
    };
  };
  /** Create a new event type */
  'eventTypes.create': {
    requestBody: {
      content: {
        'application/json': components['schemas']['EventTypePost'];
      };
    };
    responses: {
      /** @description Created */
      201: {
        content: {
          'application/json': components['schemas']['EventType'];
        };
      };
      /** @description Bad Request */
      400: never;
      /** @description Forbidden */
      403: never;
      /** @description Not Found */
      404: never;
      /** @description Conflict */
      409: never;
      /** @description Internal Server Error */
      500: never;
    };
  };
  /** Get an event type by its name */
  'eventTypes.get': {
    parameters: {
      query: {
        application_id: string;
      };
      path: {
        event_type_name: string;
      };
    };
    responses: {
      /** @description OK */
      200: {
        content: {
          'application/json': components['schemas']['EventType'];
        };
      };
      /** @description Bad Request */
      400: never;
      /** @description Forbidden */
      403: never;
      /** @description Not Found */
      404: never;
      /** @description Conflict */
      409: never;
      /** @description Internal Server Error */
      500: never;
    };
  };
  /** Delete an event type */
  'eventTypes.delete': {
    parameters: {
      query: {
        application_id: string;
      };
      path: {
        event_type_name: string;
      };
    };
    responses: {
      /** @description No Content */
      204: never;
      /** @description Bad Request */
      400: never;
      /** @description Forbidden */
      403: never;
      /** @description Not Found */
      404: never;
      /** @description Conflict */
      409: never;
      /** @description Internal Server Error */
      500: never;
    };
  };
  /** List latest events */
  'events.list': {
    parameters: {
      query: {
        application_id: string;
      };
    };
    responses: {
      /** @description OK */
      200: {
        content: {
          'application/json': components['schemas']['Event'][];
        };
      };
      /** @description Bad Request */
      400: never;
      /** @description Forbidden */
      403: never;
      /** @description Not Found */
      404: never;
      /** @description Conflict */
      409: never;
      /** @description Internal Server Error */
      500: never;
    };
  };
  /** Get an event */
  'events.get': {
    parameters: {
      query: {
        application_id: string;
      };
      path: {
        event_id: string;
      };
    };
    responses: {
      /** @description OK */
      200: {
        content: {
          'application/json': components['schemas']['EventWithPayload'];
        };
      };
      /** @description Bad Request */
      400: never;
      /** @description Forbidden */
      403: never;
      /** @description Not Found */
      404: never;
      /** @description Conflict */
      409: never;
      /** @description Internal Server Error */
      500: never;
    };
  };
  /**
   * Get instance configuration
   * @description Get an object that shows how this instance is configured.
   */
  'instance.get': {
    responses: {
      /** @description OK */
      200: {
        content: {
          'application/json': components['schemas']['InstanceConfig'];
        };
      };
      /** @description Bad Request */
      400: never;
      /** @description Forbidden */
      403: never;
      /** @description Not Found */
      404: never;
      /** @description Conflict */
      409: never;
      /** @description Internal Server Error */
      500: never;
    };
  };
  /** List organizations */
  'organizations.list': {
    responses: {
      /** @description OK */
      200: {
        content: {
          'application/json': components['schemas']['Organization'][];
        };
      };
      /** @description Bad Request */
      400: never;
      /** @description Forbidden */
      403: never;
      /** @description Not Found */
      404: never;
      /** @description Conflict */
      409: never;
      /** @description Internal Server Error */
      500: never;
    };
  };
  /**
   * Create an organization
   * @description Note that you will need to regenerate a JWT to be able to see/use the newly created organization.
   */
  'organizations.create': {
    requestBody: {
      content: {
        'application/json': components['schemas']['OrganizationPost'];
      };
    };
    responses: {
      /** @description OK */
      200: {
        content: {
          'application/json': components['schemas']['OrganizationInfo'];
        };
      };
      /** @description Bad Request */
      400: never;
      /** @description Forbidden */
      403: never;
      /** @description Not Found */
      404: never;
      /** @description Conflict */
      409: never;
      /** @description Internal Server Error */
      500: never;
    };
  };
  /** Get organization's info by its ID */
  'organizations.get': {
    parameters: {
      path: {
        organization_id: string;
      };
    };
    responses: {
      /** @description OK */
      200: {
        content: {
          'application/json': components['schemas']['OrganizationInfo'];
        };
      };
      /** @description Bad Request */
      400: never;
      /** @description Forbidden */
      403: never;
      /** @description Not Found */
      404: never;
      /** @description Conflict */
      409: never;
      /** @description Internal Server Error */
      500: never;
    };
  };
  /**
   * Edit an organization
   * @description Note that you will need to regenerate a JWT to be able to see the updated name of the organization.
   */
  'organizations.edit': {
    parameters: {
      path: {
        organization_id: string;
      };
    };
    requestBody: {
      content: {
        'application/json': components['schemas']['OrganizationPost'];
      };
    };
    responses: {
      /** @description OK */
      200: {
        content: {
          'application/json': components['schemas']['OrganizationInfo'];
        };
      };
      /** @description Bad Request */
      400: never;
      /** @description Forbidden */
      403: never;
      /** @description Not Found */
      404: never;
      /** @description Conflict */
      409: never;
      /** @description Internal Server Error */
      500: never;
    };
  };
  /**
   * Delete an organization
   * @description Note that you will need to regenerate a JWT to be able to make the deleted organization go away.
   */
  'organizations.delete': {
    parameters: {
      path: {
        organization_id: string;
      };
    };
    responses: {
      /** @description No Content */
      204: never;
      /** @description Bad Request */
      400: never;
      /** @description Forbidden */
      403: never;
      /** @description Not Found */
      404: never;
      /** @description Conflict */
      409: never;
      /** @description Internal Server Error */
      500: never;
    };
  };
  /** Invite a user to an organization */
  'organizations.invite': {
    parameters: {
      path: {
        organization_id: string;
      };
    };
    requestBody: {
      content: {
        'application/json': components['schemas']['UserInvitation'];
      };
    };
    responses: {
      /** @description OK */
      200: {
        content: {
          'application/json': components['schemas']['UserInvitation'];
        };
      };
      /** @description Bad Request */
      400: never;
      /** @description Forbidden */
      403: never;
      /** @description Not Found */
      404: never;
      /** @description Conflict */
      409: never;
      /** @description Internal Server Error */
      500: never;
    };
  };
  /** Revoke a user's access to an organization */
  'organizations.revoke': {
    parameters: {
      path: {
        organization_id: string;
      };
    };
    requestBody: {
      content: {
        'application/json': components['schemas']['Revoke'];
      };
    };
    responses: {
      /** @description OK */
      200: {
        content: {
          'application/json': components['schemas']['Revoke'];
        };
      };
      /** @description Bad Request */
      400: never;
      /** @description Forbidden */
      403: never;
      /** @description Not Found */
      404: never;
      /** @description Conflict */
      409: never;
      /** @description Internal Server Error */
      500: never;
    };
  };
  /**
   * List supported event payload content types
   * @description List of every possible content types that can be used in event payloads.
   */
  'payload_content_types.list': {
    responses: {
      /** @description OK */
      200: {
        content: {
          'application/json': string[];
        };
      };
      /** @description Bad Request */
      400: never;
      /** @description Forbidden */
      403: never;
      /** @description Not Found */
      404: never;
      /** @description Conflict */
      409: never;
      /** @description Internal Server Error */
      500: never;
    };
  };
  /** Create a new user account and a new organization */
  register: {
    requestBody: {
      content: {
        'application/json': components['schemas']['RegistrationPost'];
      };
    };
    responses: {
      /** @description Created */
      201: {
        content: {
          'application/json': components['schemas']['Registration'];
        };
      };
      /** @description Bad Request */
      400: never;
      /** @description Forbidden */
      403: never;
      /** @description Not Found */
      404: never;
      /** @description Conflict */
      409: never;
      /** @description Internal Server Error */
      500: never;
    };
  };
  /** List request attempts */
  'requestAttempts.read': {
    parameters: {
      query: {
        application_id: string;
        event_id?: string;
        subscription_id?: string;
      };
    };
    responses: {
      /** @description OK */
      200: {
        content: {
          'application/json': components['schemas']['RequestAttempt'][];
        };
      };
      /** @description Bad Request */
      400: never;
      /** @description Forbidden */
      403: never;
      /** @description Not Found */
      404: never;
      /** @description Conflict */
      409: never;
      /** @description Internal Server Error */
      500: never;
    };
  };
  /**
   * Get a response by its ID
   * @description A response is produced when a request attempt is processed
   */
  'response.get': {
    parameters: {
      query: {
        application_id: string;
      };
      path: {
        response_id: string;
      };
    };
    responses: {
      /** @description OK */
      200: {
        content: {
          'application/json': components['schemas']['Response'];
        };
      };
      /** @description Bad Request */
      400: never;
      /** @description Forbidden */
      403: never;
      /** @description Not Found */
      404: never;
      /** @description Conflict */
      409: never;
      /** @description Internal Server Error */
      500: never;
    };
  };
  /**
   * List subscriptions
   * @description List all subscriptions created by customers against the application events
   */
  'subscriptions.list': {
    parameters: {
      query: {
        application_id: string;
      };
    };
    responses: {
      /** @description OK */
      200: {
        content: {
          'application/json': components['schemas']['Subscription'][];
        };
      };
      /** @description Bad Request */
      400: never;
      /** @description Forbidden */
      403: never;
      /** @description Not Found */
      404: never;
      /** @description Conflict */
      409: never;
      /** @description Internal Server Error */
      500: never;
    };
  };
  /**
   * Create a new subscription
   * @description A subscription let your customers subscribe to events. Events will be sent through the defined medium inside the subscription (e.g. HTTP POST request) as a webhook.
   */
  'subscriptions.create': {
    requestBody: {
      content: {
        'application/json': components['schemas']['SubscriptionPost'];
      };
    };
    responses: {
      /** @description Created */
      201: {
        content: {
          'application/json': components['schemas']['Subscription'];
        };
      };
      /** @description Bad Request */
      400: never;
      /** @description Forbidden */
      403: never;
      /** @description Not Found */
      404: never;
      /** @description Conflict */
      409: never;
      /** @description Internal Server Error */
      500: never;
    };
  };
  /** Get a subscription by its id */
  'subscriptions.get': {
    parameters: {
      path: {
        subscription_id: string;
      };
    };
    responses: {
      /** @description OK */
      200: {
        content: {
          'application/json': components['schemas']['Subscription'];
        };
      };
      /** @description Bad Request */
      400: never;
      /** @description Forbidden */
      403: never;
      /** @description Not Found */
      404: never;
      /** @description Conflict */
      409: never;
      /** @description Internal Server Error */
      500: never;
    };
  };
  /** Update a subscription */
  'subscriptions.update': {
    parameters: {
      path: {
        subscription_id: string;
      };
    };
    requestBody: {
      content: {
        'application/json': components['schemas']['SubscriptionPost'];
      };
    };
    responses: {
      /** @description OK */
      200: {
        content: {
          'application/json': components['schemas']['Subscription'];
        };
      };
      /** @description Bad Request */
      400: never;
      /** @description Forbidden */
      403: never;
      /** @description Not Found */
      404: never;
      /** @description Conflict */
      409: never;
      /** @description Internal Server Error */
      500: never;
    };
  };
  /** Delete a subscription */
  'subscriptions.delete': {
    parameters: {
      query: {
        application_id: string;
      };
      path: {
        subscription_id: string;
      };
    };
    responses: {
      /** @description No Content */
      204: never;
      /** @description Bad Request */
      400: never;
      /** @description Forbidden */
      403: never;
      /** @description Not Found */
      404: never;
      /** @description Conflict */
      409: never;
      /** @description Internal Server Error */
      500: never;
    };
  };
}
