module.exports = (locals) => {
  return [
    {
      primary: true,
      title: 'Open-Source',
      description: `Unlike alternatives, Hook0 is fully <a href='${locals.social.gitlab.href}' target='_blank'>open-source</a>. No vendor-locking, we are here to stay, no investors, we are fully sustainable since day 1.`,
    },
    {
      primary: true,
      title: 'Easy Integration',
      description: 'Our JSON REST API and integrations makes it easy to trigger webhook events from your Application and connect to every available SaaS',
    },
    {
      primary: true,
      title: 'Enterprise Level Security',
      description: 'All webhooks are SSL secured and contain Signing Secrets to prevent Replay, Forgery and Man-in-the-middle attacks',
    },
    {
      primary: true,
      title: 'Smart Retries',
      description: 'Managing webhook retries is a pain. Our exponential back offs, endpoint monitoring and notifications handle it for you',
      wip: false,
    },
    {
      primary: true,
      title: 'Make Your Subscribers Happy',
      description: 'Give your users a primo experience with our mock payloads, webhook logs and subscriber portal',
      wip: false,
    },
    {
      primary: true,
      title: 'Transparent Webhooks',
      description: 'All webhook attempts are logged so you and your subscribers can easily search, debug and replay old events',
      wip: false,
    },
    {
      primary: true,
      title: 'Embeddable Portal',
      description: 'Give your subscribers a branded experience with a custom subdomain and your logo uploaded on the subscriber portal',
      wip: false,
    },
    {
      primary: true,
      title: 'Real-time Monitoring',
      description: 'We monitor your subscriber endpoints for SSL and uptime and send notifications for non-responsive endpoints',
      wip: true,
    },
    {
      primary: true,
      title: 'Data & Sovereignty',
      description: 'Hook0 does not lock your data nor your software. If you subscribe to Hook0 SaaS version, all your data will stay in Europe. No GAFAM there.',
    },
    {
      title: 'Fine-grained subscriptions',
      description: 'Enable your users to subscribe to your events by setting up a webhook. They can choose which event types they want to receive.',
      wip: false,
    },
    {
      title: 'Multi subscriptions',
      description: 'Your users can register several webhooks, we will send events to all of them!',
      wip: false,
    },
    {
      title: 'Event scoping',
      description: 'Scope events to one or several levels of your application. Users, organizations, administrators, [insert your own], they can all handle subscriptions to their events.',
      wip: false,
    },
    {
      title: 'Dashboards',
      description: 'Either use Hook0 out-of-the-box dashboards to let your users see events that went through their subscriptions, or build your own with the API.',
      wip: true,
    },

    {
      title: 'Failure notification',
      description: "If after several retries we still can't successfuly reach a webhook, your subscriber is notified by email.",
      wip: true,
    },
    {
      title: 'Events & responses persistence',
      description: 'Hook0 can keep track of every event your application sent it and of every webhook call. This can helps you debug things or act as an audit log !',
      wip: false,
    },
    {
      title: 'High availability',
      description: "Hook0 won't miss the events you send it.",
      wip: true,
    },
    {
      title: 'GDPR Compliant',
      description: 'Hook0 is fully GDPR compliant and can easily execute a data processor agreement with your company if needed.',
      wip: false,
    },
    {
      title: 'Data Security',
      description: 'Hook0 utilizes best practices for data storage and encryption. We also offer single-tenant and on-premise deployment options.',
      wip: false,
    },
    {
      title: 'Designed for Enteprise Scale',
      description: 'Hook0 robust architecture automatically scales to handle thousands of requests per minute.',
      wip: false,
    },
  ];
};
