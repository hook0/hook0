// Per-page strings for gdpr-subprocessors (EN base — GDPR Subprocessors / art. 28 RGPD).
//
// Source: src/gdpr-subprocessors.ejs (legacy v2024-09-28). Inline legal-reviewer
// audit applied before extraction; each correction is flagged inline below with
// [LEGAL-CORRECTION L#] referencing the original line of the legacy template.
// The "Last Update" date is bumped to 2026-06-27 to reflect those corrections.
//
// Hard legal facts (CLAUDE.md / CLAUDE.local.md) kept verbatim across locales:
//   - Controller relationship: FGRibreau SARL (data processor for Customer Content)
//   - Subprocessor entities (verbatim across locales):
//       * Clever Cloud SAS (France) — primary data plane
//       * Scaleway SAS (France) — optional dedicated workers
//       * Cloudflare, Inc. (USA, 101 Townsend St, San Francisco, CA 94107) — CDN
//         and DDoS protection, disclosed for CLOUD Act transparency
//       * Stripe Inc. (USA) — billing
//       * Brevo (France) — transactional email
//       * Postmark (USA) — transactional email fallback
//       * BetterUptime (Czech Republic) — uptime monitoring
//       * Sentry (USA) — error tracking
//       * Crisp (France) — customer relations
//       * Gmail / Google Workspace (USA) — support mailbox
//   - Transfer mechanisms: SCC 2021 (Standard Contractual Clauses) + TIA
//     (Transfer Impact Assessment) for US transfers; EU-US DPF (Data Privacy
//     Framework) when the subprocessor is certified.
//   - No "100% sovereign" / "no data sharing" claims (L121-1 C. conso risk).
//   - SSPL framing rule: not applicable on this page (no license mention).
//
// EN prose stays close to the live template; the /humanizer pro pass applies
// to FR/DE only. HTML markup inside body fields is preserved and emitted via
// <%- t.section.field %> in the template.
module.exports = {
  pageTitle: 'Hook0 - GDPR Subprocessors',
  pageDescription: 'Learn about Hook0 GDPR compliance and the subprocessors we use to provide our webhook services. Full transparency on data processing in Europe and on US transfers.',
  pageModified: '2026-06-27',
  hero: {
    eyebrow: 'Compliance',
    title: 'GDPR & Subprocessors',
    subtitle: 'Our commitment to data protection and the partners we work with.',
    lastUpdatedLabel: 'Last Update:',
    lastUpdatedDate: 'June 27, 2026',
  },
  intro: {
    // [LEGAL-CORRECTION L78] Fix "DSVGO" typo → "DSGVO".
    p1Html: 'The General Data Protection Regulation (GDPR / DSGVO) is the toughest privacy and security law in the world. It imposes obligations onto organizations anywhere, so long as they target or collect data related to people in the EU. The regulation was approved by the EU Parliament in April 2016 and came into effect on May 25, 2018.',
    p2Html: 'Hook0 uses certain sub-processors to assist it in providing Application Services to its customers, as described in the Master Services Agreement or Terms of Use available at <a href="./terms" class="text-green-400 hover:text-green-300 transition-colors">terms-of-service</a> or such other location as the Terms of Use may be posted from time to time (as applicable, the "Agreement"). Defined terms used herein shall have the same meaning as defined in the Agreement.',
  },
  whatIsPersonalData: {
    title: 'What is Personal Data?',
    bodyHtml: 'GDPR is especially concerned about protecting personal data of individuals. Personal data (Art. 4 GDPR) consists of any information that allows us to identify a person directly or indirectly and can be anything such as a name, email address, credit card number, or documents with personal information.',
  },
  howWeProcess: {
    title: 'How We Process Personal Data',
    bodyHtml: 'When you visit our websites or use our services, we will most likely process your personal data in one way or another. You can find all relevant information about which data we process, our legal basis for processing, and your rights regarding your personal data in our <a href="./privacy-policy" class="text-green-400 hover:text-green-300 transition-colors">privacy policy</a>.',
  },
  roles: {
    title: 'Subprocessors and Their Roles',
    // [LEGAL-CORRECTION] Add explicit art. 28(2)/(4) framing and right to object.
    p1Html: 'A subprocessor is a third-party data processor engaged by Hook0, including entities from within the Hook0 group, who has or potentially will have access to or process Customer Content (which may contain Personal Data). Hook0 engages different types of subprocessors to perform various functions as explained in the tables below.',
    p2Html: 'Under Article 28(2) and 28(4) GDPR, you grant Hook0 a general written authorisation to engage the subprocessors listed below. We will inform you of any intended changes to this list, including the addition or replacement of subprocessors, giving you a reasonable opportunity to object before the change takes effect.',
  },
  // [LEGAL-CORRECTION L156, L207] Transfer mechanism column added per art. 46 GDPR.
  infrastructure: {
    title: 'Infrastructure',
    intro: 'We use the following subprocessors to provide our cloud infrastructure environment and storage of our Customer Content:',
    table: {
      headers: ['Subprocessor', 'Country of Processing', 'Purpose', 'Transfer Mechanism'],
      rows: [
        {
          name: 'Clever Cloud SAS',
          country: 'France, Europe',
          countryIsEU: true,
          purpose: 'Hook0 customer database, API, and web application',
          transfer: 'EU processing (no transfer outside EEA)',
        },
        {
          name: 'Cloudflare, Inc. (101 Townsend St, San Francisco, CA 94107)',
          country: 'USA',
          countryIsEU: false,
          purpose: 'DNS and DDoS protection',
          transfer: 'SCC 2021 + TIA; EU-US DPF (Cloudflare is DPF-certified)',
        },
      ],
    },
  },
  customerContent: {
    title: 'Processing of Customer Content',
    intro: 'Hook0 works with various subprocessors that monitor, maintain, and support the Application Services. These subprocessors may, but not necessarily will, have access to Customer Content:',
    table: {
      headers: ['Subprocessor', 'Country', 'Purpose', 'Transfer Mechanism'],
      rows: [
        {
          name: 'Clever Cloud SAS',
          country: 'France, Europe',
          countryIsEU: true,
          purpose: 'Workers that call the webhook subscription endpoints',
          transfer: 'EU processing (no transfer outside EEA)',
        },
        {
          name: 'Scaleway SAS',
          country: 'France, Europe',
          countryIsEU: true,
          purpose: 'Private dedicated workers that call the webhook subscription endpoints (only for relevant customers)',
          transfer: 'EU processing (no transfer outside EEA)',
        },
        {
          name: 'Stripe Inc.',
          country: 'USA',
          countryIsEU: false,
          purpose: 'Hook0\'s customer subscription management',
          transfer: 'SCC 2021 + TIA; EU-US DPF (Stripe is DPF-certified)',
        },
        {
          name: 'Brevo',
          country: 'France, Europe',
          countryIsEU: true,
          purpose: 'Automated emailing',
          transfer: 'EU processing (no transfer outside EEA)',
        },
        {
          name: 'Postmark',
          country: 'USA',
          countryIsEU: false,
          purpose: 'Automated emailing',
          transfer: 'SCC 2021 + TIA',
        },
        {
          name: 'BetterUptime',
          country: 'Czech Republic, Europe',
          countryIsEU: true,
          purpose: 'Uptime monitoring, status page, and on-call management',
          transfer: 'EU processing (no transfer outside EEA)',
        },
        {
          name: 'Sentry',
          country: 'USA',
          countryIsEU: false,
          purpose: 'Error tracking',
          transfer: 'SCC 2021 + TIA',
        },
        {
          name: 'Crisp',
          country: 'France, Europe',
          countryIsEU: true,
          purpose: 'Customer relations management',
          transfer: 'EU processing (no transfer outside EEA)',
        },
        {
          name: 'Gmail (Google Workspace)',
          country: 'USA',
          countryIsEU: false,
          purpose: 'Support mailbox',
          transfer: 'SCC 2021 + TIA; EU-US DPF (Google LLC is DPF-certified)',
        },
      ],
    },
    // [LEGAL-CORRECTION L260] Tighten incomplete footnote; reference the page's modified date.
    footnoteHtml: '* The list of subprocessors above applies to any new Hook0 customers as of the date shown at the top of this page, and to existing Hook0 customers who have not otherwise received notice of a different effective date.',
  },
  // [LEGAL-CORRECTION L273] Replace "unmatched GDPR compliance" / "best global
  // infrastructure" / "highest levels" superlatives with a sober process-claim
  // framing, per L121-1 C. conso and CLAUDE.local.md DSGVO/RGPD process-claim rule.
  control: {
    title: 'Stay in Control',
    bodyHtml: 'Hook0 is a French SaaS designed for GDPR compliance. We rely on infrastructure and partners selected for confidentiality, integrity, and availability of your data. If you would rather not rely on our or our subprocessors\' measures, you can still access our support services without disclosing your production data.',
  },
  // [LEGAL-CORRECTION L286] The legacy line "no data transfers outside of the
  // EU for your deployment" was false (Stripe US, Sentry US, Postmark US, Gmail
  // US, Cloudflare US are all in the chain). Rewritten to honestly distinguish
  // the data plane (EU) from the ancillary services (US, framed by SCC 2021 + TIA).
  dataOwnership: {
    title: 'Data Ownership and Management',
    p1Html: 'Your webhook payload data plane (Clever Cloud workers, and optionally Scaleway dedicated workers) is operated in the EU and your webhook content is not transferred outside the EEA for the purpose of webhook delivery. Backups are stored in French data centres. Ancillary services such as billing (Stripe), error tracking (Sentry), transactional email fallback (Postmark), the support mailbox (Gmail) and the CDN / DDoS layer (Cloudflare) do involve transfers to the United States, governed by the Standard Contractual Clauses 2021 (SCC 2021) and a documented Transfer Impact Assessment, and where applicable by the EU-US Data Privacy Framework. All Hook0 staff and consultants who may access your deployment are based in the EU.',
    p2Html: 'Regarding your own user database, you must establish the required processes to comply with GDPR yourself and declare all data transfers that you handle independently. In this case, Hook0 acts as a subprocessor, and our <a href="./data-processing-addendum" class="text-green-400 hover:text-green-300 transition-colors">DPA (Data Processing Agreement)</a> specifies what we do.',
  },
};
