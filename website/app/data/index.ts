import { BsFillLightningChargeFill } from "react-icons/bs";
import { HiOutlineBookOpen } from "react-icons/hi";
import { PiBank } from "react-icons/pi";
import { FaCode } from "react-icons/fa6";
import { IoLogoGitlab, IoEarthOutline } from "react-icons/io5";
import { ImConnection } from "react-icons/im";
import {
  HiOutlineServerStack,
  HiOutlineClipboardDocumentList,
} from "react-icons/hi2";
import { AiOutlineDiscord, AiOutlineYoutube } from "react-icons/ai";
import { FiGithub, FiAlertTriangle } from "react-icons/fi";
import { CiTwitter, CiLinkedin, CiCircleQuestion } from "react-icons/ci";
import { IoCheckmarkDone } from "react-icons/io5";
import { PiClockCountdownThin } from "react-icons/pi";

export const socialMedia = [
  {
    link: "https://twitter.com/hook0_",
    img: "/twitter.svg",
  },
  {
    link: "https://www.hook0.com/community",
    img: "/discord.svg",
  },
  {
    link: "https://gitlab.com/hook0",
    img: "/gitlab.svg",
  },
  {
    link: "https://www.youtube.com/channel/UCFGvNaoV6Ycdb6uh1rIvMcg",
    img: "/youtube.svg",
  },
  {
    link: "https://github.com/hook0",
    img: "/github.svg",
  },
  {
    link: "https://www.linkedin.com/company/hook0",
    img: "/linkedin.svg",
  },
];

export const navItems = [
  { name: "Use cases", link: "#use-cases" },
  { name: "Pricing", link: "#pricing" },
  { name: "Documentation", link: "https://documentation.hook0.com" },
  { name: "Contact", link: "mailto:support@hook0.com" },
];

export const footerItems = [
  {
    category: "About",
    items: [
      { name: "Contact", link: "mailto:support@hook/com" },
      { name: "Pricing", link: "#pricing" },
      { name: "Resources", link: "https://documentation.hook0.com" },
      { name: "Security & Compliance", link: "/security" },
      { name: "Privacy", link: "/privacy-policy" },
      { name: "Legal", link: "/terms" },
    ],
  },
  {
    category: "Developers",
    items: [
      {
        name: "Start in 5 minutes",
        link: "https://documentation.hook0.com/docs/getting-started",
        icon: BsFillLightningChargeFill,
      },
      {
        name: "Documentation",
        link: "https://documentation.hook0.com/docs",
        icon: HiOutlineBookOpen,
      },
      {
        name: "API reference",
        link: "https://documentation.hook0.com/reference/",
        icon: PiBank,
      },
      {
        name: "SDK & Libraries",
        link: "https://github.com/hook0",
        icon: FaCode,
      },
      {
        name: "Source Code",
        link: "https://github.com/hook0/hook0",
        icon: IoLogoGitlab,
      },
      {
        name: "Status Page",
        link: "https://status.hook0.com/",
        icon: ImConnection,
      },
      {
        name: "Self-hosting Hook0",
        link: "https://documentation.hook0.com/docs/docker-compose",
        icon: HiOutlineServerStack,
      },
    ],
  },
  {
    category: "Community",
    items: [
      {
        name: "Code of Conduct",
        link: "https://gitlab.com/hook0/hook0/-/blob/master/CODE_OF_CONDUCT.md",
        icon: HiOutlineClipboardDocumentList,
      },
      {
        name: "OSS Friends",
        link: "https://www.hook0.com/oss-friends",
        icon: IoEarthOutline,
      },
      {
        name: "Discord",
        link: "https://www.hook0.com/community",
        icon: AiOutlineDiscord,
      },
      {
        name: "Github",
        link: "https://github.com/hook0/hook0",
        icon: FiGithub,
      },
      {
        name: "Linkedin",
        link: "https://www.linkedin.com/company/hook0",
        icon: CiLinkedin,
      },
      {
        name: "YouTube",
        link: "https://www.youtube.com/channel/UCFGvNaoV6Ycdb6uh1rIvMcg",
        icon: AiOutlineYoutube,
      },
      { name: "Twitter", link: "https://twitter.com/hook0_", icon: CiTwitter },
    ],
  },
];

export const siteConfig = {
  name: "Hook0",
  description: "Hook0 description",
};

export const outoftheBox = [
  {
    title: "Open-Source",
    description:
      "Unlike alternatives, Hook0 is fully open-source. No vendor-locking, we are here to stay, no investors, we are fully sustainable since day 1",
    img: "/out-of-the-boxs/open-source.svg",
  },
  {
    title: "Easy Integration",
    description:
      "Our JSON REST API and integrations makes it easy to trigger webhook events from your Application and connect to every available SaaS",
    img: "/out-of-the-boxs/easy-integration.svg",
  },
  {
    title: "Enterprise Level Security",
    description:
      "All webhooks are SSL secured and contain Signing Secrets to prevent Replay, Forgery and Man-in-the-middle attacks",
    img: "/out-of-the-boxs/security.svg",
  },
  {
    title: "Smart Retries",
    description:
      "Managing webhook retries is a pain. Our exponential back offs, endpoint monitoring and notifications handle it for you",
    img: "/out-of-the-boxs/retries.svg",
  },
  {
    title: "Make Your Subscribers Happy",
    description:
      "Give your users a primo experience with our mock payloads, webhook logs and subscriber portal",
    img: "/out-of-the-boxs/happy-customers.svg",
  },
  {
    title: "Transparent Webhooks",
    description:
      "All webhook attempts are logged so you and your subscribers can easily search, debug and replay old events",
    img: "/out-of-the-boxs/webhooks.svg",
  },
  {
    title: "Embeddable Portal",
    description:
      "Give your subscribers a branded experience with a custom subdomain and your logo uploaded on the subscriber portal",
    img: "/out-of-the-boxs/embeddable-portal.svg",
  },
  {
    title: "Real-time Monitoring",
    description:
      "We monitor your subscriber endpoints for SSL and uptime and send notifications for non-responsive endpoints",
    img: "/out-of-the-boxs/real-time-monitoring.svg",
  },
  {
    title: "Data & Sovereignty",
    description:
      "Hook0 does not lock your data nor your software. If you subscribe to Hook0 SaaS version, all your data will stay in Europe. No GAFAM there.",
    img: "/out-of-the-boxs/data-sovereignty.svg",
  },
];

export const everythingYouNeed = [
  {
    title: "Fine-grained subscriptions",
    description:
      "Enable your users to subscribe to your events by setting up a webhook. They can choose which event types they want to receive.",
    icon: IoCheckmarkDone,
    color: "text-green-400",
  },
  {
    title: "Event scoping",
    description:
      "Scope events to one or several levels of your application. Users, organizations, administrators, [insert your own], they can all handle subscriptions to their events.",
    icon: IoCheckmarkDone,
    color: "text-green-400",
  },
  {
    title: "Failure notification",
    description:
      "If after several retries we still can't successfuly reach a webhook, your subscriber is notified by email.",
    icon: PiClockCountdownThin,
    color: "text-orange-400",
  },
  {
    title: "Data Security",
    description:
      "Hook0 utilizes best practices for data storage and encryption. We also offer single-tenant and on-premise deployment options.",
    icon: IoCheckmarkDone,
    color: "text-green-400",
  },
  {
    title: "Multi subscriptions",
    description:
      "Your users can register several webhooks, we will send events to all of them!",
    icon: IoCheckmarkDone,
    color: "text-green-400",
  },
  {
    title: "Dashboards",
    description:
      "Either use Hook0 out-of-the-box dashboards to let your users see events that went through their subscriptions, or build your own with the API.",
    icon: PiClockCountdownThin,
    color: "text-orange-400",
  },
  {
    title: "Events & responses persistence",
    description:
      "Hook0 can keep track of every event your application sent it and of every webhook call. This can helps you debug things or act as an audit log !",
    icon: IoCheckmarkDone,
    color: "text-green-400",
  },
  {
    title: "GDPR Compliant",
    description:
      "Hook0 is fully GDPR compliant and can easily execute a data processor agreement with your company if needed.",
    icon: IoCheckmarkDone,
    color: "text-green-400",
  },
  {
    title: "Designed for Enteprise Scale",
    description:
      "Hook0 robust architecture automatically scales to handle thousands of requests per minute.",
    icon: IoCheckmarkDone,
    color: "text-green-400",
  },
];

export const useCases = [
  {
    title: "No Code Solutions",
    description:
      "Integrate with platforms like Zapier and Integromat and offer your users webhooks without any code.",
    img: "/use-cases/code.svg",
  },
  {
    title: "Stop Polling",
    description:
      "We all know polling for data is inefficient. Use Webhooks instead to push data to your users.",
    img: "/use-cases/stop-polling.svg",
  },
  {
    title: "No More Custom Reporting",
    description:
      "Have your users subscribe to your webhooks and give them a self serve integration.",
    img: "/use-cases/reporting.svg",
  },
  {
    title: "Asynchronous Notifications",
    description:
      "For time consuming tasks, use webhooks to asynchronously notify your users when they are done.",
    img: "/use-cases/asynchronous-notifications.svg",
  },
  {
    title: "Internal Systems Communication",
    description:
      "An event bus solution can be heavy, start with internal webhooks to communicate between internal services.",
    img: "/use-cases/retry.svg",
  },
  {
    title: "Platform Notifications",
    description:
      "Giving your platform users access to events occuring on your platform improves the experience for everyone.",
    img: "/use-cases/notification.svg",
  },
  {
    title: "Back Office Operations",
    description:
      "Let your subscribers know an order has been refunded or shipment has been delivered so they can take action.",
    img: "/use-cases/storm.svg",
  },
  {
    title: "Provide Updates On Progress",
    description:
      "As data in your system changes, update your subscribers with those changes.",
    img: "/use-cases/right-arrow.svg",
  },
];

export const pricingPlans = [
  {
    category: "Cloud",
    items: [
      {
        name: "Developer",
        description:
          "Perfect way to try out Hook0, no need to setup onpremiseing, free forever for side-projects.",
        price: "FREE",
        duration: "Forever",
        link: "https://www.hook0.com/?pricing.destination=cloud#top",
        link_text: "Sign Up",
        includes: [
          {
            icon: IoCheckmarkDone,
            status: "success",
            text: "Fully managed, no infra. to own",
          },
          {
            icon: IoCheckmarkDone,
            status: "success",
            text: "1 developer",
          },
          {
            icon: IoCheckmarkDone,
            status: "success",
            text: "1 application",
          },
          {
            icon: IoCheckmarkDone,
            status: "success",
            text: "5 event types",
          },
          {
            icon: IoCheckmarkDone,
            status: "success",
            text: "Up to 100 events per day",
          },
          {
            icon: IoCheckmarkDone,
            status: "success",
            text: "7 days data retention",
          },
          {
            icon: IoCheckmarkDone,
            status: "success",
            text: "No credit card required",
          },
          {
            icon: IoCheckmarkDone,
            status: "success",
            text: "Community support on Discord",
          },
          {
            icon: IoCheckmarkDone,
            status: "success",
            text: "Support video and documentation",
          },
          {
            icon: IoCheckmarkDone,
            status: "success",
            text: "Account security",
          },
        ],
      },
      {
        name: "Startup",
        description:
          "Enhance your webhook experience with our new features and grow your start-up!",
        price: "€59",
        duration: "month",
        link: "https://buy.stripe.com/eVaaH8agJcMT6RieV0",
        link_text: "Subscribe",
        includes: [
          {
            icon: IoCheckmarkDone,
            status: "success",
            text: "Fully managed, no infra. to own",
          },
          {
            icon: IoCheckmarkDone,
            status: "success",
            text: "25 developers",
          },
          {
            icon: IoCheckmarkDone,
            status: "success",
            text: "1 application",
          },
          {
            icon: IoCheckmarkDone,
            status: "success",
            text: "10 event types",
          },
          {
            icon: IoCheckmarkDone,
            status: "success",
            text: "Up to 30,000 events per day",
          },
          {
            icon: IoCheckmarkDone,
            status: "success",
            text: "14 days data retention",
          },
          {
            icon: IoCheckmarkDone,
            status: "success",
            text: "Standard support (3-day email)",
          },
          {
            icon: IoCheckmarkDone,
            status: "success",
            text: "Support video and documentation",
          },
          {
            icon: IoCheckmarkDone,
            status: "success",
            text: "Account security",
          },
        ],
      },
      {
        name: "Pro",
        description:
          "Unleash your data connectivity along with enterprise features and better support.",
        price: "€190",
        duration: "month",
        link: "https://buy.stripe.com/fZe02ucoR007b7y8ww",
        link_text: "Subscribe",
        includes: [
          {
            icon: IoCheckmarkDone,
            status: "success",
            text: "Fully managed, no infra. to own",
          },
          {
            icon: IoCheckmarkDone,
            status: "success",
            text: "Unlimited developers",
          },
          {
            icon: IoCheckmarkDone,
            status: "success",
            text: "Unlimited applications",
          },
          {
            icon: IoCheckmarkDone,
            status: "success",
            text: "Unlimited event types",
          },
          {
            icon: IoCheckmarkDone,
            status: "success",
            text: "Up to 100,000 events per day",
          },
          {
            icon: IoCheckmarkDone,
            status: "success",
            text: "30 days data retention",
          },
          {
            icon: IoCheckmarkDone,
            status: "success",
            text: "Premium support (3-day email)",
          },
          {
            icon: IoCheckmarkDone,
            status: "success",
            text: "Support video and documentation",
          },
          {
            icon: IoCheckmarkDone,
            status: "success",
            text: "Account security",
          },
        ],
      },
      {
        name: "Enterprise",
        description:
          "You need more? You need different? Let us know and we will build a custom plan just for you.",
        price: "Custom",
        duration: "month",
        link: "mailto:support@hook0.com",
        link_text: "Contact",
        includes: [
          {
            icon: IoCheckmarkDone,
            status: "success",
            text: "Fully managed, no infra. to own",
          },
          {
            icon: IoCheckmarkDone,
            status: "success",
            text: "Unlimited developers",
          },
          {
            icon: IoCheckmarkDone,
            status: "success",
            text: "Unlimited applications",
          },
          {
            icon: IoCheckmarkDone,
            status: "success",
            text: "Unlimited event types",
          },
          {
            icon: IoCheckmarkDone,
            status: "success",
            text: "Custom events per day",
          },
          {
            icon: IoCheckmarkDone,
            status: "success",
            text: "Custom data retention",
          },
          {
            icon: IoCheckmarkDone,
            status: "success",
            text: "Custom support level",
          },
          {
            icon: IoCheckmarkDone,
            status: "success",
            text: "Support video and documentation",
          },
          {
            icon: IoCheckmarkDone,
            status: "success",
            text: "Custom requirements",
          },
          {
            icon: IoCheckmarkDone,
            status: "success",
            text: "Enterprise security & compliance",
          },
          {
            icon: IoCheckmarkDone,
            status: "success",
            text: "Static source IPs (on demand)",
          },
        ],
      },
    ],
  },
  {
    category: "On-Premise",
    items: [
      {
        name: "Self Hosted",
        description:
          "Open-source, onpremiseed. Complete control over your data. Security and privacy compliant.",
        price: "Free",
        duration: "Forever",
        link: "https://documentation.hook0.com/docs/docker-compose",
        link_text: "Get installation instructions",
        includes: [
          {
            icon: IoCheckmarkDone,
            status: "success",
            text: "Host your own instance",
          },
          {
            icon: IoCheckmarkDone,
            status: "success",
            text: "Your data, your rules",
          },
          {
            icon: IoCheckmarkDone,
            status: "success",
            text: "Server Side Public License",
          },
          {
            icon: IoCheckmarkDone,
            status: "success",
            text: "Community support on Discord",
          },
          {
            icon: FiAlertTriangle,
            status: "danger",
            text: "Infrastructure scaling",
          },
          {
            icon: FiAlertTriangle,
            status: "danger",
            text: "99.9% uptime",
          },
          {
            icon: FiAlertTriangle,
            status: "danger",
            text: "Managed updates",
          },
        ],
      },
      {
        name: "Pro",
        description:
          "We deploy a dedicated Hook0 instance to your environment and help you maintain/update it.",
        price: "€1000",
        duration: "setup",
        second_price: "€500",
        second_duration: "month",
        link: "https://buy.stripe.com/3cs9D4gF75kr5NefZ6",
        link_text: "Subscribe",
        includes: [
          {
            icon: IoCheckmarkDone,
            status: "success",
            text: "Your data, your rules",
          },
          {
            icon: IoCheckmarkDone,
            status: "success",
            text: "Compatible with most cloud providers",
          },
          {
            icon: IoCheckmarkDone,
            status: "success",
            text: "Managed updates",
          },
          {
            icon: IoCheckmarkDone,
            status: "success",
            text: "Premium support (3-day email)",
          },
        ],
      },
      {
        name: "Enterprise",
        description:
          "We help you exactly the way you need to provide a great webhook experience to your users.",
        price: "Custom",
        duration: "month",
        link: "mailto:support@hook0.com",
        link_text: "Contact us",
        includes: [
          {
            icon: IoCheckmarkDone,
            status: "success",
            text: "Your data, your rules",
          },
          {
            icon: IoCheckmarkDone,
            status: "success",
            text: "Custom support level",
          },
          {
            icon: CiCircleQuestion,
            status: "warning",
            text: "Study of your use cases",
          },
          {
            icon: CiCircleQuestion,
            status: "warning",
            text: "Deployment on your architecture",
          },
          {
            icon: CiCircleQuestion,
            status: "warning",
            text: "Instance management",
          },
          {
            icon: CiCircleQuestion,
            status: "warning",
            text: "Custom developments",
          },
          {
            icon: CiCircleQuestion,
            status: "warning",
            text: "Training",
          },
        ],
      },
    ],
  },
];
