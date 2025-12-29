import React from 'react';
import Link from '@docusaurus/Link';
import useBaseUrl from '@docusaurus/useBaseUrl';
import styles from './styles.module.css';

// Import shared data via webpack alias (configured in docusaurus.config.js)
import sharedData from '@shared/website-data';
const sharedFooterLinks = sharedData.footerLinks;
const sharedSocialLinks = sharedData.socialLinks;
const sharedFooterIcons = sharedData.footerIcons;
const sharedSocial = sharedData.social;

// Transform shared data for Docusaurus (use docPath for internal links when available)
const footerLinks = {
  about: sharedFooterLinks.about,
  guides: {
    ...sharedFooterLinks.guides,
    items: sharedFooterLinks.guides.items.map(item => ({
      ...item,
      to: item.docPath, // Use docPath for internal Docusaurus routing
    })),
  },
  developers: {
    ...sharedFooterLinks.developers,
    items: sharedFooterLinks.developers.items.map(item => ({
      ...item,
      to: item.docPath, // Use docPath for internal Docusaurus routing
    })),
  },
  community: sharedFooterLinks.community,
};

// Helper to render icon from shared data
function Icon({ name, className }) {
  const iconData = sharedFooterIcons[name];
  if (!iconData) return null;

  if (iconData.type === 'social') {
    // Social icons are filled SVGs
    const socialData = sharedSocial[iconData.key];
    if (!socialData?.logo) return null;

    // Parse the SVG string to extract the path
    const pathMatch = socialData.logo.match(/d="([^"]+)"/);
    if (!pathMatch) return null;

    return (
      <svg className={className} fill="currentColor" viewBox="0 0 24 24">
        <path d={pathMatch[1]} />
      </svg>
    );
  }

  // Stroke icons
  const colorClass = iconData.color === 'green' ? styles.iconGreen : '';
  return (
    <svg className={`${className} ${colorClass}`} fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth={iconData.strokeWidth || 2}
        d={iconData.path}
      />
    </svg>
  );
}

const socialLinks = sharedSocialLinks;

function FooterLink({ item }) {
  const isExternal = item.href && !item.to;
  const LinkComponent = item.to ? Link : 'a';
  const linkProps = item.to
    ? { to: item.to }
    : { href: item.href, target: '_blank', rel: 'noopener noreferrer' };

  return (
    <li className={styles.linkItem}>
      <LinkComponent {...linkProps} className={styles.link}>
        {item.icon && <Icon name={item.icon} className={styles.icon} />}
        {item.label}
      </LinkComponent>
    </li>
  );
}

function FooterColumn({ title, items }) {
  return (
    <div className={styles.column}>
      <h3 className={styles.columnTitle}>{title}</h3>
      <ul className={styles.linkList}>
        {items.map((item, idx) => (
          <FooterLink key={idx} item={item} />
        ))}
      </ul>
    </div>
  );
}

export default function Footer() {
  const logoUrl = useBaseUrl('/img/logo.svg');

  return (
    <footer className={styles.footer}>
      <div className={styles.container}>
        <div className={styles.grid}>
          {/* Brand Column */}
          <div className={styles.brandColumn}>
            <Link to="/" className={styles.brand}>
              <div className={styles.logoWrapper}>
                <img src={logoUrl} alt="Hook0" className={styles.logo} />
              </div>
              <span className={styles.brandName}>Hook0</span>
            </Link>
            <p className={styles.tagline}>
              Open-Source Webhooks-as-a-Service. Built by developers for developers.
            </p>
            <div className={styles.badge}>
              <svg className={styles.badgeFlag} width="28" height="20" fill="none" viewBox="0 -4 28 28">
                <g clipPath="url(#clip0_eu)" transform="translate(0 -4)">
                  <rect width="28" height="20" x="0" y="0" fill="#fff" rx="2" />
                  <mask id="mask0_eu" width="28" height="20" x="0" y="0" maskUnits="userSpaceOnUse">
                    <rect width="28" height="20" x="0" y="0" fill="#fff" rx="2" />
                  </mask>
                  <g mask="url(#mask0_eu)">
                    <rect width="28" height="20" x="0" y="0" fill="#043cae" />
                    <path fill="#ffd429" fillRule="evenodd" d="M13.057 4.276 14 4l.943.276-.276-.943.276-.942-.943.276-.943-.276.276.942Zm0 13.334.943-.277.943.277-.276-.943.276-.943L14 16l-.943-.276.276.943Zm7.61-6.943-.943.276L20 10l-.276-.943.943.276.942-.276-.276.943.277.943Zm-14.277.276.943-.276.943.276L8 10l.276-.943-.943.276-.942-.276.276.943Zm13.383-3.61-.942.276.276-.942-.276-.943.943.276.942-.276-.276.943.276.942Zm-12.49 6.943L8.227 14l.943.276-.276-.943.276-.943-.943.277-.942-.277.276.943Zm10.05-9.383-.942.276.276-.942-.277-.943.943.276.943-.276-.276.943.276.942Zm-7.61 11.823.944-.276.943.276-.277-.943.277-.942-.943.276-.943-.276.276.943ZM19.774 14l-.942.276.276-.943-.276-.943.943.277.942-.277-.276.943.276.943ZM7.283 7.61l.943-.277.943.276-.276-.942.276-.943L8.226 6l-.942-.276.276.943Zm10.05 8.83-.942.276.276-.943-.276-.942.942.276.943-.276-.276.943.276.942ZM9.723 5.17l.944-.277.943.276-.277-.942.277-.943-.943.276-.943-.276.276.943Z" clipRule="evenodd" />
                  </g>
                </g>
                <defs>
                  <clipPath id="clip0_eu">
                    <rect width="28" height="20" fill="#fff" rx="2" />
                  </clipPath>
                </defs>
              </svg>
              Made in France
            </div>
          </div>

          {/* Link Columns */}
          <FooterColumn {...footerLinks.about} />
          <FooterColumn {...footerLinks.guides} />
          <FooterColumn {...footerLinks.developers} />
          <FooterColumn {...footerLinks.community} />
        </div>
      </div>

      {/* Bottom Bar */}
      <div className={styles.bottomBar}>
        <div className={styles.container}>
          <div className={styles.bottomContent}>
            <div className={styles.copyright}>
              <p>© {new Date().getFullYear()} Hook0. All rights reserved.</p>
              <p className={styles.bootstrapped}>100% bootstrapped, no VCs. We are here to stay.</p>
            </div>
            <div className={styles.socialLinks}>
              {socialLinks.map((social) => (
                <a
                  key={social.name}
                  href={social.href}
                  target="_blank"
                  rel="noopener noreferrer"
                  className={styles.socialLink}
                  title={social.name}
                >
                  <span className="sr-only">{social.name}</span>
                  <Icon name={social.icon} className={styles.icon} />
                </a>
              ))}
            </div>
          </div>
        </div>
      </div>
    </footer>
  );
}
