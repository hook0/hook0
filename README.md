<!-- PROJECT LOGO -->
<p align="center">
  <a href="https://github.com/hook0/hook0">
   <img src="./mediakit/logo/1920x1920-banner.png" alt="Hook0 Logo banner">
  </a>

<h3 align="center">Hook0</h3>

  <p align="center">
    The open-source implementation of a webhooks server and UI.
    <br />
    <a href="https://www.hook0.com"><strong>Learn more »</strong></a>
    <br />
    <br />
    <a href="https://www.hook0.com/community">Discord</a>
    ·
    <a href="https://www.hook0.com">Website</a>
    ·
    <a href="https://github.com/hook0/hook0/issues">Issues</a>
    ·
    <a href="https://gitlab.com/hook0/hook0/-/boards">Roadmap</a>
  </p>
</p>

<p align="center">
   <a href="https://www.hook0.com/community"><img src="https://img.shields.io/badge/Discord-hook0.com%2Fcommunity-%234A154B" alt="Join Hook0 Discord"></a>
   <!--<a href="https://producthunt.com/posts/hook0"><img src="https://img.shields.io/badge/Product%20Hunt-%231%20Product%20of%20the%20Month-%23DA552E" alt="Product Hunt"></a>-->
   <a href="https://status.hook0.com/"><img height="20px" src="https://uptime.betterstack.com/status-badges/v1/monitor/es5i.svg" alt="Uptime"></a>
   <a href="https://github.com/hook0/hook0/stargazers"><img src="https://img.shields.io/github/stars/hook0/hook0" alt="Github Stars"></a>
   <!--<a href="https://news.ycombinator.com/item?id="><img src="https://img.shields.io/badge/Hacker%20News-%231-%23FF6600" alt="Hacker News"></a>-->
   <a href="https://github.com/hook0/hook0/blob/main/LICENSE.txt"><img src="https://img.shields.io/badge/license-SSPL-purple" alt="License"></a>
   <a href="https://github.com/hook0/hook0/pulse"><img src="https://img.shields.io/github/commit-activity/m/hook0/hook0" alt="Commits-per-month"></a>
   <a href="https://www.hook0.com/?pricing.destination=cloud#pricing"><img src="https://img.shields.io/badge/Pricing-Free-brightgreen" alt="Pricing"></a>
   <!--<a href="https://hub.docker.com/r/hook0/hook0"><img src="https://img.shields.io/docker/pulls/hook0/hook0"></a>-->
   <a href="https://gitlab.com/hook0/hook0/-/issues/?sort=milestone_due_desc&state=opened&label_name%5B%5D=Community%3A%3Aaccepting%20merge%20requests&first_page_size=20"><img src="https://img.shields.io/badge/Help%20Wanted-Contribute-blue"></a>
   <a href="https://www.hook0.com/community"><img src="https://img.shields.io/badge/translations-contribute-brightgreen" /></a>
   <a href="https://contributor-covenant.org/version/1/4/code-of-conduct/"><img src="https://img.shields.io/badge/Contributor%20Covenant-1.4-purple" /></a>
</p>

Welcome to Hook0! Sign up to [hook0.com](https://www.hook0.com/) and start opening your SaaS to the web!

You should check our documentation website to know what Hook0 is and what is our vision: https://documentation.hook0.com/docs/what-is-hook0

# 🐰 Introduction
Hook0 is a real-time API and App dashboard for managing webhooks.

- **Free & open-source**. No artificial limitations, vendor lock-in, or hidden paywalls.
- **REST API**. JSON REST API and integrations makes it easy to trigger webhook events from your Application and connect to every available SaaS
- **Fine-grained subscriptions**. Enable your users to subscribe to your events by setting up a webhook. They can choose which event types they want to receive.
- **Auto-Retry**. If Hook0 can't reach a webhook, or if it does not respond with a success code, Hook0 will try again automatically.
- **Event scoping**. Scope events to one or several levels of your application.
- **Events & responses persistence**. Hook0 keep tracks of every event your application sent it and of every webhook call. Best for debugging !
- **On-Prem or Cloud**. Run locally, install on-premises, or use our self-service Cloud service (free tier available).
- **A modern dashboard**. Our dashboard app is safe and intuitive for non-technical users, no training required.
- **Project fully sustainable** since day 1. Fork it, twist it and help us build the best open source webhook server for applications

[Learn more about Hook0](https://www.hook0.com/)

# 🚀 Hook0 Cloud
Hook.com (Cloud) allows you to create free Hook0 projects in 90 seconds.

- Free Community Cloud tier available (no credit card required)
- No product limitations or service usage quotas (unlimited users and applications, API requests, etc)
- A modern self-service dashboard to create and monitor all your projects in one place
- End-to-end solution: Hook0, database, auto-scaling, storage, and a global CDN
- Event-based pricing for our Standard Cloud allows you to pay-as-you-go
- Select your desired region and provision a new project in ~90 seconds

[Create your Free Project](https://www.hook0.com/) - [Contact a human](mailto:sales@hook0.com)

# 🤔 Community Help
The [Hook0 Documentation](https://documentation.hook0.com/) is a great place to start, or explore these other channels:

- [Discord](https://www.hook0.com/community) (Live Discussions)
- [Gitlab Issues](https://gitlab.com/hook0/hook0/-/issues) (Report Bugs, Questions, Feature Requests)
- [Twitter](https://twitter.com/hook0_) (Latest News)
- [Website](https://www.hook0.com/) (Login, sign up)

# 📌 Requirements
Hook0 only requires Rust and supports most operating systems and SQL database vendors.

- Rust active LTS

**Supported Databases**
- PostgreSQL 10+

**Supported OS**
- Ubuntu 18.04
- CentOS / RHEL 8
- macOS Catalina or newer
- Windows 10/11
- Docker (DockerHub + Dockerfile)

_Other operating systems may also work, but are not officially supported._

# Related

[Hook0 Cloud Status Page](https://status.hook0.com/)

# ❤️ Contributing & Sponsoring
Please read our [Contributing Guide](./contributing.md) before submitting Pull Requests.

All security vulnerabilities should be reported in accordance with our Security Policy.

# License

Hook0 is a premium open-source [Server Side Public License (SSPL) v1](./LICENSE.txt) project made possible with support from our passionate core team, talented contributors, and amazing Sponsors. Thank you all!

The license allows the free right to use, modify, create derivative works, and redistribute, with three simple limitations:

- You may not provide the products to others as a managed service
- You may not circumvent the license key functionality or remove/obscure features protect

© Hook0 SAS
