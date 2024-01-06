const locals = {};

// https://simpleicons.org/
locals.social = {
  twitter: {
    name: 'Twitter',
    href: 'https://twitter.com/hook0_',
    logo: `<svg role="img" class="h-6 w-6" fill="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><title>Twitter</title><path d="M23.953 4.57a10 10 0 01-2.825.775 4.958 4.958 0 002.163-2.723c-.951.555-2.005.959-3.127 1.184a4.92 4.92 0 00-8.384 4.482C7.69 8.095 4.067 6.13 1.64 3.162a4.822 4.822 0 00-.666 2.475c0 1.71.87 3.213 2.188 4.096a4.904 4.904 0 01-2.228-.616v.06a4.923 4.923 0 003.946 4.827 4.996 4.996 0 01-2.212.085 4.936 4.936 0 004.604 3.417 9.867 9.867 0 01-6.102 2.105c-.39 0-.779-.023-1.17-.067a13.995 13.995 0 007.557 2.209c9.053 0 13.998-7.496 13.998-13.985 0-.21 0-.42-.015-.63A9.935 9.935 0 0024 4.59z"/></svg>`,
  },
  discord: {
    name: 'Discord',
    href: 'https://www.hook0.com/community',
    logo: `<svg role="img" class="h-6 w-6" fill="currentColor"  viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><title>Discord</title><path d="M20.317 4.3698a19.7913 19.7913 0 00-4.8851-1.5152.0741.0741 0 00-.0785.0371c-.211.3753-.4447.8648-.6083 1.2495-1.8447-.2762-3.68-.2762-5.4868 0-.1636-.3933-.4058-.8742-.6177-1.2495a.077.077 0 00-.0785-.037 19.7363 19.7363 0 00-4.8852 1.515.0699.0699 0 00-.0321.0277C.5334 9.0458-.319 13.5799.0992 18.0578a.0824.0824 0 00.0312.0561c2.0528 1.5076 4.0413 2.4228 5.9929 3.0294a.0777.0777 0 00.0842-.0276c.4616-.6304.8731-1.2952 1.226-1.9942a.076.076 0 00-.0416-.1057c-.6528-.2476-1.2743-.5495-1.8722-.8923a.077.077 0 01-.0076-.1277c.1258-.0943.2517-.1923.3718-.2914a.0743.0743 0 01.0776-.0105c3.9278 1.7933 8.18 1.7933 12.0614 0a.0739.0739 0 01.0785.0095c.1202.099.246.1981.3728.2924a.077.077 0 01-.0066.1276 12.2986 12.2986 0 01-1.873.8914.0766.0766 0 00-.0407.1067c.3604.698.7719 1.3628 1.225 1.9932a.076.076 0 00.0842.0286c1.961-.6067 3.9495-1.5219 6.0023-3.0294a.077.077 0 00.0313-.0552c.5004-5.177-.8382-9.6739-3.5485-13.6604a.061.061 0 00-.0312-.0286zM8.02 15.3312c-1.1825 0-2.1569-1.0857-2.1569-2.419 0-1.3332.9555-2.4189 2.157-2.4189 1.2108 0 2.1757 1.0952 2.1568 2.419 0 1.3332-.9555 2.4189-2.1569 2.4189zm7.9748 0c-1.1825 0-2.1569-1.0857-2.1569-2.419 0-1.3332.9554-2.4189 2.1569-2.4189 1.2108 0 2.1757 1.0952 2.1568 2.419 0 1.3332-.946 2.4189-2.1568 2.4189Z"/></svg>`,
  },
  gitlab: {
    name: 'Gitlab',
    href: 'https://gitlab.com/hook0',
    repoHref: 'https://gitlab.com/hook0/hook0',
    logo: `<svg role="img" class="h-6 w-6" fill="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><title>GitLab</title><path d="M4.845.904c-.435 0-.82.28-.955.692C2.639 5.449 1.246 9.728.07 13.335a1.437 1.437 0 00.522 1.607l11.071 8.045c.2.145.472.144.67-.004l11.073-8.04a1.436 1.436 0 00.522-1.61c-1.285-3.942-2.683-8.256-3.817-11.746a1.004 1.004 0 00-.957-.684.987.987 0 00-.949.69l-2.405 7.408H8.203l-2.41-7.408a.987.987 0 00-.942-.69h-.006zm-.006 1.42l2.173 6.678H2.675zm14.326 0l2.168 6.678h-4.341zm-10.593 7.81h6.862c-1.142 3.52-2.288 7.04-3.434 10.559L8.572 10.135zm-5.514.005h4.321l3.086 9.5zm13.567 0h4.325c-2.467 3.17-4.95 6.328-7.411 9.502 1.028-3.167 2.059-6.334 3.086-9.502zM2.1 10.762l6.977 8.947-7.817-5.682a.305.305 0 01-.112-.341zm19.798 0l.952 2.922a.305.305 0 01-.11.341v.002l-7.82 5.68.026-.035z"/></svg>`,
  },
  youtube: {
    name: 'Youtube',
    href: 'https://www.youtube.com/channel/UCFGvNaoV6Ycdb6uh1rIvMcg',
    logo: `<svg role="img" class="h-6 w-6" fill="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><title>YouTube</title><path d="M23.498 6.186a3.016 3.016 0 0 0-2.122-2.136C19.505 3.545 12 3.545 12 3.545s-7.505 0-9.377.505A3.017 3.017 0 0 0 .502 6.186C0 8.07 0 12 0 12s0 3.93.502 5.814a3.016 3.016 0 0 0 2.122 2.136c1.871.505 9.376.505 9.376.505s7.505 0 9.377-.505a3.015 3.015 0 0 0 2.122-2.136C24 15.93 24 12 24 12s0-3.93-.502-5.814zM9.545 15.568V8.432L15.818 12l-6.273 3.568z"/></svg>`,
  },
  github: {
    name: 'Github',
    href: 'https://github.com/hook0',
    repoHref: 'https://github.com/hook0/hook0',
    logo: `<svg role="img" class="h-6 w-6" fill="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><title>GitHub</title><path d="M12 .297c-6.63 0-12 5.373-12 12 0 5.303 3.438 9.8 8.205 11.385.6.113.82-.258.82-.577 0-.285-.01-1.04-.015-2.04-3.338.724-4.042-1.61-4.042-1.61C4.422 18.07 3.633 17.7 3.633 17.7c-1.087-.744.084-.729.084-.729 1.205.084 1.838 1.236 1.838 1.236 1.07 1.835 2.809 1.305 3.495.998.108-.776.417-1.305.76-1.605-2.665-.3-5.466-1.332-5.466-5.93 0-1.31.465-2.38 1.235-3.22-.135-.303-.54-1.523.105-3.176 0 0 1.005-.322 3.3 1.23.96-.267 1.98-.399 3-.405 1.02.006 2.04.138 3 .405 2.28-1.552 3.285-1.23 3.285-1.23.645 1.653.24 2.873.12 3.176.765.84 1.23 1.91 1.23 3.22 0 4.61-2.805 5.625-5.475 5.92.42.36.81 1.096.81 2.22 0 1.606-.015 2.896-.015 3.286 0 .315.21.69.825.57C20.565 22.092 24 17.592 24 12.297c0-6.627-5.373-12-12-12"/></svg>`,
  },
  linkedin: {
    name: 'LinkedIn',
    href: 'https://www.linkedin.com/company/hook0',
    logo: `<svg role="img" class="h-6 w-6" fill="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><title>LinkedIn</title><path d="M20.447 20.452h-3.554v-5.569c0-1.328-.027-3.037-1.852-3.037-1.853 0-2.136 1.445-2.136 2.939v5.667H9.351V9h3.414v1.561h.046c.477-.9 1.637-1.85 3.37-1.85 3.601 0 4.267 2.37 4.267 5.455v6.286zM5.337 7.433c-1.144 0-2.063-.926-2.063-2.065 0-1.138.92-2.063 2.063-2.063 1.14 0 2.064.925 2.064 2.063 0 1.139-.925 2.065-2.064 2.065zm1.782 13.019H3.555V9h3.564v11.452zM22.225 0H1.771C.792 0 0 .774 0 1.729v20.542C0 23.227.792 24 1.771 24h20.451C23.2 24 24 23.227 24 22.271V1.729C24 .774 23.2 0 22.222 0h.003z"/></svg>`,
  },
};

locals.meta = {
  login: 'https://app.hook0.com/',
  doc_api_reference: 'https://documentation.hook0.com/reference/',
  doc_guides: 'https://documentation.hook0.com/docs',
  doc_getstarted: 'https://documentation.hook0.com/docs/getting-started',
  contact: 'mailto:support@hook0.com',
};

locals['guide-sdk-tutorial'] = [
  {
    language: 'Python',
    sdk: {
      repository: '',
      setup: 'pip install hook0',
    },
    send_message: `hook0 = Hook0("AUTH_TOKEN")
hook0.message.create(
    "c0ea6ffa-1972-4435-b434-ec9e93d38f42",
    MessageIn(
        event_type: "invoice.paid",
        event_id: "evt_Wqb1k73rXprtTm7Qdlr38G",
        payload: {
            "id": "invoice_WF7WtCLFFtd8ubcTgboSFNql",
            "status": "paid",
            "attempt": 2
        }
    )
)`,
  },
  {
    language: 'NodeJS',
    sdk: {
      repository: '',
      setup: 'npm install hook0',
      send_message: `const hook0 = Hook0("AUTH_TOKEN")
await hook0.message.create(
    "c0ea6ffa-1972-4435-b434-ec9e93d38f42",
    {
        event_type: "invoice.paid",
        event_id: "evt_Wqb1k73rXprtTm7Qdlr38G",
        payload: {
            "id": "invoice_WF7WtCLFFtd8ubcTgboSFNql",
            "status": "paid",
            "attempt": 2
        }
    }
)`,
    },
  },
];

locals.features = [
  {
    primary: true,
    title: 'Open-Source',
    description: `Unlike alternatives, Hook0 is fully <a href='${locals.social.github.href}' target='_blank'>open-source</a>. No vendor-locking, we are here to stay, no investors, we are fully sustainable since day 1.`,
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

locals.subscriptionLinks = {
  cloud: {
    startup: 'https://buy.stripe.com/eVaaH8agJcMT6RieV0',
    pro: 'https://buy.stripe.com/fZe02ucoR007b7y8ww',
  },
  onprem: {
    pro: 'https://buy.stripe.com/3cs9D4gF75kr5NefZ6',
  },
};

locals.ossFriends = require('./oss-friends.json').data;

module.exports = locals;
