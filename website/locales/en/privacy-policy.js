// Per-page strings for privacy-policy (EN base — GDPR Art. 13 / Politique de
// confidentialite). Source: src/privacy-policy.ejs (legacy bilingual EN+FR, lines
// 1-567 for EN). The legacy template stacked EN and FR under one <html lang="en">
// page with a visual <hr class="lang-divider"> + <span class="lang-badge"> pair;
// the data-driven multi-locale conversion drops those legacy separators and
// renders ONE locale per page.
//
// Inline legal-reviewer audit applied before extraction; each correction flagged
// inline below with [LEGAL-CORRECTION L#] (legal substance) or [ISMS-SYNC L#]
// (drift vs documentation/hook0-cloud/*.md) referencing the original line of the
// legacy template. The "Last updated" date is bumped to 2026-06-27 to reflect
// those corrections and to track the same audit date as the just-shipped DPA +
// gdpr-subprocessors files (ISMS-synced source of truth).
//
// Hard legal facts (CLAUDE.md / CLAUDE.local.md) verbatim across locales:
//   - Data controller: FGRibreau SARL, capital 2 000 EUR, RCS La Roche-sur-Yon
//     850 824 350, VAT FR27850824350, registered office 3 rue de l'Aubepine,
//     85110 Chantonnay, France.
//   - Director of publication: David Sferruzza.
//   - Privacy contact: legal@hook0.com (DPA-aligned).
//   - Subprocessors disclosed (same set as gdpr-subprocessors.js):
//       * Clever Cloud SAS (France) - primary data plane
//       * Scaleway SAS (France) - optional dedicated workers
//       * Cloudflare, Inc. (USA, 101 Townsend St, San Francisco, CA 94107)
//         - CDN and DDoS protection
//       * Stripe Inc. (USA) - billing
//       * Brevo (France) - transactional email
//       * Postmark (USA) - transactional email fallback
//       * BetterUptime (Czech Republic) - uptime monitoring
//       * Sentry (USA) - error tracking
//       * Crisp (France) - support chat (consent-gated)
//       * Gmail / Google Workspace (USA) - support mailbox
//       * Google LLC (Google Ads, USA) - server-side conversion measurement
//         (gclid only)
//   - Transfer mechanisms: SCC 2021 + TIA; EU-US DPF where the subprocessor is
//     DPF-certified (Cloudflare, Stripe, Google LLC).
//   - Per-plan webhook retention: Developer 7d / Startup 14d / Pro 30d /
//     Enterprise custom (information-retention-policy.md).
//   - Account data retention: contract duration + 30 days after account
//     deletion.
//   - Billing records retention: 10 years (art. L102 B Livre des procedures
//     fiscales).
//   - Breach notification: 72 hours (art. 33/34 GDPR).
//   - Cookie consent TTL: 13 months max (CNIL guideline).
//   - No "100% sovereign" / "no data sharing" / "no US provider" claims
//     (L121-1 C. conso risk). RGPD = process claim, never absolute.
//   - SSPL framing rule: not applicable on this page (no license mention).
//
// EN prose stays close to the live template; /humanizer pro pass applies to
// FR/DE only. HTML markup inside *Html fields is preserved and emitted via
// <%- t.section.field %> in the template.
module.exports = {
  pageTitle: 'Hook0 - Privacy Policy',
  pageDescription: 'Hook0 Privacy Policy - GDPR Article 13 compliant. Legal basis, data retention, your rights, subprocessors, and transfers outside the EU.',
  pageModified: '2026-06-27',
  hero: {
    eyebrow: 'Legal',
    title: 'Privacy Policy',
    subtitle: 'How Hook0 collects, uses, and protects your personal data, in compliance with GDPR Article 13.',
    lastUpdatedLabel: 'Last updated:',
    lastUpdatedDate: 'June 27, 2026',
  },
  controller: {
    title: '1. Data Controller',
    p1: 'The data controller responsible for processing your personal data in connection with the Hook0 service is:',
    // [LEGAL-CORRECTION L113-118] Add capital, RCS, VAT and director of publication
    // to align with the legal-pages identity used by `mentions-legales.ejs` and the
    // DPA. The legacy block only listed the postal address and "SIRET on request",
    // which is below the standard art. 13(1)(a) GDPR + art. 6-III LCEN bar.
    identityHtml: '<strong class="text-white">FGRibreau SARL</strong>, a French limited liability company (Societe a Responsabilite Limitee) with a share capital of 2,000 EUR, registered with the Trade and Companies Register of La Roche-sur-Yon under number 850 824 350, VAT FR27850824350, with its registered office at 3 rue de l\'Aubepine, 85110 Chantonnay, France.<br>Director of publication: David Sferruzza.<br>Privacy contact: <a href="mailto:legal@hook0.com" class="text-green-400 hover:text-green-300 transition-colors">legal@hook0.com</a>',
    note: 'Hook0 is a 100% B2B SaaS platform. We do not intentionally collect data from individuals acting in a personal capacity.',
  },
  purposes: {
    title: '2. Purposes and Legal Bases',
    intro: 'The following table sets out each processing activity, the data involved, and the legal basis under Article 6 GDPR.',
    headers: ['Purpose', 'Data categories', 'Legal basis (Art. 6 GDPR)'],
    rows: [
      {
        purposeHtml: '<strong class="text-white">Service provision</strong><br><span class="text-gray-400 text-sm">Account creation, authentication, API access, webhook delivery</span>',
        data: 'Email address, name, API keys, webhook payloads, IP address, usage logs',
        basisHtml: 'Art. 6(1)(b) - Performance of a contract',
      },
      {
        purposeHtml: '<strong class="text-white">Billing and payment</strong><br><span class="text-gray-400 text-sm">Subscription management, invoicing, tax records</span>',
        data: 'Name, email, billing address, payment instrument data (processed by Stripe), subscription history',
        basisHtml: 'Art. 6(1)(b) - Performance of a contract<br>Art. 6(1)(c) - Legal obligation (French fiscal law, 10-year retention)',
      },
      {
        purposeHtml: '<strong class="text-white">Website analytics</strong><br><span class="text-gray-400 text-sm">Understanding how visitors use our site via Matomo (self-hosted)</span>',
        data: 'Anonymised IP address, pages visited, referrer, device type, session duration',
        basisHtml: 'Art. 6(1)(a) - Consent (cookie banner)',
      },
      {
        purposeHtml: '<strong class="text-white">Conversion tracking (server-side)</strong><br><span class="text-gray-400 text-sm">Google Ads conversion measurement, server-side via the click identifier (gclid) only. No email, no IP, no User-Agent transmitted to Google. Right to object at <a href="mailto:legal@hook0.com" class="text-green-400 hover:text-green-300 transition-colors">legal@hook0.com</a>.</span>',
        data: 'Click identifier (gclid), pseudonymous identifier issued by Google during the ad click',
        basisHtml: 'Art. 6(1)(f) - Legitimate interests (measuring advertising ROI)<br><span class="text-gray-400 text-sm">Right to object: Art. 21(2) GDPR</span>',
      },
      {
        purposeHtml: '<strong class="text-white">Customer support, live chat</strong><br><span class="text-gray-400 text-sm">Crisp widget (loaded only after consent)</span>',
        data: 'Name, email, chat messages, browser metadata',
        basisHtml: 'Art. 6(1)(a) - Consent',
      },
      {
        purposeHtml: '<strong class="text-white">Customer support, email</strong><br><span class="text-gray-400 text-sm">Handling support requests sent to legal@hook0.com or support@hook0.com</span>',
        data: 'Name, email, content of exchanges',
        basisHtml: 'Art. 6(1)(f) - Legitimate interests (responding to customer requests)',
      },
      {
        purposeHtml: '<strong class="text-white">Security and monitoring</strong><br><span class="text-gray-400 text-sm">Error tracking, uptime monitoring, DDoS protection, incident response</span>',
        data: 'IP address, error stack traces, request metadata, uptime check results',
        basisHtml: 'Art. 6(1)(f) - Legitimate interests (ensuring service integrity and security)',
      },
      {
        purposeHtml: '<strong class="text-white">Commercial communications</strong><br><span class="text-gray-400 text-sm">Product updates, release notes, newsletters</span>',
        data: 'Email address, first name',
        basisHtml: 'Art. 6(1)(a) - Consent',
      },
    ],
  },
  dataCategories: {
    title: '3. Categories of Personal Data',
    items: [
      '<strong class="text-white">Identity data:</strong> first name, last name, professional email address',
      '<strong class="text-white">Account data:</strong> username, encrypted password, API keys',
      '<strong class="text-white">Payment data:</strong> billing address, last 4 digits of card and expiry date (Stripe stores full card data, Hook0 never has access to full card numbers)',
      '<strong class="text-white">Technical data:</strong> IP address, browser user-agent, connection timestamps, error logs',
      '<strong class="text-white">Usage data:</strong> webhook events sent and received, API call volume, feature usage metrics',
      '<strong class="text-white">Communications:</strong> content of support exchanges, chat transcripts',
    ],
    note: 'Hook0 does not process special categories of personal data (Article 9 GDPR) and does not perform automated decision-making or profiling with legal or similarly significant effects.',
  },
  subprocessors: {
    title: '4. Recipients and Subprocessors',
    introHtml: 'We share data with our subprocessors strictly as needed to provide the Service. The complete and up-to-date list is maintained at <a href="./gdpr-subprocessors" class="text-green-400 hover:text-green-300 transition-colors">/gdpr-subprocessors</a>. A summary is provided below.',
    groups: [
      {
        title: 'Infrastructure',
        headers: ['Subprocessor', 'Country', 'Purpose'],
        rows: [
          ['Clever Cloud SAS', 'France (EU)', 'Database, API, web application hosting'],
          ['Cloudflare, Inc.', 'USA', 'DNS and DDoS protection'],
        ],
      },
      {
        title: 'Service operation',
        headers: ['Subprocessor', 'Country', 'Purpose'],
        rows: [
          ['Clever Cloud SAS', 'France (EU)', 'Workers calling webhook subscription endpoints'],
          ['Scaleway SAS', 'France (EU)', 'Private dedicated workers (selected plans)'],
          ['Stripe, Inc.', 'USA', 'Subscription and payment management'],
          ['Brevo (Sendinblue)', 'France (EU)', 'Automated transactional emails'],
          ['Postmark (ActiveCampaign)', 'USA', 'Automated transactional emails'],
          ['BetterUptime', 'Czech Republic (EU)', 'Uptime monitoring and status page'],
          ['Sentry, Inc.', 'USA', 'Application error tracking'],
          ['Crisp', 'France (EU)', 'Customer support chat (consent-gated)'],
          ['Google LLC (Gmail)', 'USA', 'Support inbox'],
        ],
      },
      {
        title: 'Marketing measurement (legitimate interest, server-side)',
        headers: ['Subprocessor', 'Country', 'Purpose'],
        rows: [
          ['Google LLC (Google Ads)', 'USA', 'Server-side conversion measurement (gclid only). See Section 9b.'],
        ],
      },
      {
        title: 'Analytics (consent-gated)',
        headers: ['Service', 'Country', 'Purpose'],
        rows: [
          ['Matomo (self-hosted on matomo.hook0.com)', 'France (EU)', 'Website analytics'],
        ],
      },
    ],
    note: 'A Data Processing Agreement (DPA) is in place with each subprocessor. For transfers outside the EU, see Section 5.',
  },
  transfers: {
    title: '5. Transfers Outside the European Union',
    // [LEGAL-CORRECTION L361] Add EU-US DPF as a complementary transfer mechanism
    // for DPF-certified subprocessors (Cloudflare, Stripe, Google LLC). The legacy
    // version only mentioned SCC 2021, which is incomplete and forces an undue
    // reliance on the TIA where DPF certification already provides an adequacy
    // route. Aligned with gdpr-subprocessors.js + DPA section 6.
    p1Html: 'Several subprocessors are established in the United States: Cloudflare, Stripe, Postmark, Sentry, Gmail (Google), and Google Ads. These transfers are governed by the <strong class="text-white">Standard Contractual Clauses (SCCs)</strong> approved by the European Commission (Decision 2021/914) and a documented Transfer Impact Assessment, and where applicable by the <strong class="text-white">EU-US Data Privacy Framework</strong> (Cloudflare, Stripe and Google LLC are DPF-certified). Together these mechanisms provide an adequate level of protection for personal data.',
    cloudActHtml: '<strong>CLOUD Act notice:</strong> US-established providers may be subject to the CLOUD Act (Clarifying Lawful Overseas Use of Data Act), which may allow US authorities to require access to data held by those providers, even when stored outside the US. Hook0 applies a data minimisation approach and limits the personal data shared with US-based subprocessors to what is strictly necessary.',
  },
  retention: {
    title: '6. Retention Periods',
    headers: ['Data category', 'Retention period', 'Justification'],
    rows: [
      ['Account data', 'Duration of the contract + 30 days after account deletion', 'Contractual necessity; 30-day grace period to allow data export'],
      ['Billing and invoicing records', '10 years from the date of the transaction', 'Legal obligation, French General Tax Code, Art. L102 B of the Tax Procedures Book'],
      // [ISMS-SYNC L392] Replace the legacy "7 to 30 days depending on the
      // subscription plan" with the precise per-plan breakdown from
      // information-retention-policy.md and the DPA Annex 2.
      ['Webhook event logs', 'Developer 7 days, Startup 14 days, Pro 30 days, Enterprise custom', 'Service delivery; configurable per subscription plan'],
      ['Website analytics (Matomo)', '25 months', 'CNIL recommendation for analytics data'],
      ['Support communications', '3 years from the last exchange', 'Legitimate interests; statutory limitation period for contractual claims'],
      ['Consent records', '5 years from the date of consent', 'Ability to demonstrate compliance (Art. 7(1) GDPR)'],
      // [ISMS-SYNC] Add server logs row to mirror DPA Appendix 2.
      ['Server logs', '30 days minimum, then automatic rotation and deletion', 'Service operation, security and incident response'],
    ],
  },
  rights: {
    title: '7. Your Rights',
    intro: 'Under the GDPR, you have the following rights with respect to your personal data:',
    items: [
      '<strong class="text-white">Right of access</strong> (Art. 15), obtain a copy of the personal data we hold about you',
      '<strong class="text-white">Right to rectification</strong> (Art. 16), correct inaccurate or incomplete data',
      '<strong class="text-white">Right to erasure</strong> (Art. 17), request deletion of your data, subject to legal retention obligations',
      '<strong class="text-white">Right to restriction</strong> (Art. 18), request that we restrict processing in certain circumstances',
      '<strong class="text-white">Right to data portability</strong> (Art. 20), receive your data in a structured, machine-readable format, where processing is based on consent or contract',
      '<strong class="text-white">Right to object</strong> (Art. 21), object to processing based on legitimate interests or for direct marketing purposes',
      '<strong class="text-white">Right to withdraw consent</strong> (Art. 7(3)), withdraw consent at any time without affecting the lawfulness of prior processing',
    ],
    contactHtml: 'To exercise any of these rights, send a request to <a href="mailto:legal@hook0.com" class="text-green-400 hover:text-green-300 transition-colors">legal@hook0.com</a>. We will respond within 30 days. We may ask you to verify your identity before processing your request.',
  },
  cnil: {
    title: '8. Right to Lodge a Complaint with the CNIL',
    p1: 'If you consider that the processing of your personal data infringes the GDPR, you have the right to lodge a complaint with the French data protection authority:',
    addressHtml: '<strong class="text-white">Commission Nationale de l\'Informatique et des Libertes (CNIL)</strong><br>3 Place de Fontenoy, TSA 80715<br>75334 Paris Cedex 07, France<br>Website: <a href="https://www.cnil.fr" class="text-green-400 hover:text-green-300 transition-colors" target="_blank" rel="noopener">www.cnil.fr</a>',
    note: 'You may also contact any EU supervisory authority in the Member State of your habitual residence or place of work.',
  },
  cookies: {
    title: '9. Cookies and Trackers',
    intro: 'Hook0 uses a consent management mechanism on its website. The following services are only loaded after you have given explicit consent:',
    items: [
      '<strong class="text-white">Matomo Analytics</strong> (self-hosted), website usage analytics, anonymised by default',
      '<strong class="text-white">Crisp</strong>, live chat widget',
      '<strong class="text-white">hook0_gclid cookie</strong> (Domain <code class="text-green-400">.hook0.com</code>, 30-day TTL), bridges the Google Ads click identifier between www.hook0.com and app.hook0.com so a deferred signup can still be attributed. Set only after consent and only when an ad click brought you here. Cleared when consent is withdrawn. See Section 9b for details.',
    ],
    consentScopeHtml: 'Your consent on www.hook0.com covers all hook0.com subdomains (including app.hook0.com). Consent preferences are stored in <code class="text-green-400">localStorage</code> with a validity of <strong class="text-white">13 months</strong>, in line with CNIL guidelines. You can change your preferences at any time:',
    changeButton: 'Change Cookie Settings',
  },
  serverSideTracking: {
    title: '9b. Server-Side Conversion Measurement (Google Ads)',
    intro: 'When you reach our service by clicking on a Google Ads advertisement, Google Ads automatically appends a click identifier ("gclid") to the destination URL. This gclid is forwarded to our backend during your account creation and uploaded server-side to Google Ads to measure the effectiveness of our advertising campaigns.',
    items: [
      '<strong class="text-white">Purpose</strong>: measure cost-per-acquisition of our paid campaigns to allocate marketing budget.',
      '<strong class="text-white">Legal basis</strong>: Art. 6(1)(f) GDPR, legitimate interests. Documented balance test available on request.',
      '<strong class="text-white">Data transmitted to Google</strong>: gclid, conversion type, conversion date/time. <strong>No email, IP address, or User-Agent</strong> is transmitted to Google in this context.',
      '<strong class="text-white">Joint Controller</strong>: Google LLC, under the Customer Data Processing Terms (Art. 26 GDPR). Transfer to the USA is governed by Standard Contractual Clauses (Decision 2021/914) and, where applicable, the EU-US Data Privacy Framework (Google LLC is DPF-certified).',
      '<strong class="text-white">Retention</strong>: the gclid is processed in memory during the registration HTTP request and is not persisted in our databases after transmission to Google Ads.',
      '<strong class="text-white">Right to object</strong> (Art. 21(2) GDPR): you may object to this processing at any time by emailing <a href="mailto:legal@hook0.com" class="text-green-400 hover:text-green-300 transition-colors">legal@hook0.com</a>. We will mark your account so the gclid is not transmitted to Google Ads. Your registration is not affected.',
    ],
    footnoteHtml: 'Note: this server-side measurement does <strong>not</strong> rely on cookies, gtag.js, or any client-side tracker. Article 82 of the French Data Protection Act (transposing Article 5(3) of the e-Privacy Directive) does not apply to this processing.',
  },
  security: {
    title: '10. Security',
    p1: 'Hook0 implements appropriate technical and organisational measures to protect personal data against accidental loss, unauthorised access, disclosure, alteration, or destruction. These include encryption in transit (TLS 1.2+), encryption at rest, access controls, and regular security reviews.',
    // [ISMS-SYNC] Anchor the Security page reference (existing) and align the
    // breach-notification wording with DPA section 8 + BCDR policy (CNIL within
    // 72h, individuals without undue delay where required).
    p2Html: 'Details of our security practices are available on our <a href="./security" class="text-green-400 hover:text-green-300 transition-colors">Security page</a>.',
    p3Html: 'In the event of a personal data breach likely to result in a risk to your rights and freedoms, we will notify the CNIL within 72 hours (Art. 33 GDPR) and affected individuals without undue delay where required (Art. 34 GDPR). If you discover a potential data exposure, please report it immediately to <a href="mailto:legal@hook0.com" class="text-green-400 hover:text-green-300 transition-colors">legal@hook0.com</a>.',
  },
  changes: {
    title: '11. Changes to This Policy',
    p1: 'We may update this Privacy Policy from time to time. When we do, we will update the "Last updated" date at the top of this page. For material changes, we will notify you by email to the address associated with your account or by a prominent notice on the website at least 30 days before the change takes effect.',
  },
};
