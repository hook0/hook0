// Per-page strings for data-processing-addendum (EN base — DPA art. 28 GDPR).
//
// Source: src/data-processing-addendum.ejs (legacy v2026-04-24). Inline
// legal-reviewer audit applied before extraction; each correction is flagged
// inline below with [LEGAL-CORRECTION L#] (legal substance) or [ISMS-SYNC L#]
// (drift vs documentation/hook0-cloud/*.md) referencing the original line of
// the legacy template. The "Last Update" date is bumped to 2026-06-27 to
// reflect those corrections.
//
// Hard legal facts (CLAUDE.md / CLAUDE.local.md) verbatim across locales:
//   - Processor:  FGRibreau SARL, capital 2 000 EUR, RCS La Roche-sur-Yon
//                 850 824 350, TVA FR27850824350, registered office
//                 3 rue de l'Aubépine, 85110 Chantonnay, France.
//   - Controller: the Customer (as identified in the Terms of Service).
//   - Subprocessors disclosed in Annex 1 (consistent with the dedicated
//     gdpr-subprocessors page):
//       * Clever Cloud SAS (France) — primary data plane
//       * Cloudflare, Inc. (USA, 101 Townsend St, San Francisco, CA 94107)
//         — CDN and DDoS protection, SCC 2021 + TIA / EU-US DPF
//   - Breach notification: 72 hours (GDPR art. 33/34, BCDR policy).
//   - Backup: daily, 30-day retention (backup-policy.md).
//   - Password hashing: Argon2 (password-policy.md / secure-engineering).
//   - MFA: enforced for infrastructure access (Clever Cloud, GitLab, Stripe);
//     not yet enforced for individual customer accounts, planned for a future
//     release (access-control-policy.md).
//   - No "100% sovereign" / "no data sharing" claims (L121-1 C. conso risk).
//   - SSPL framing rule: not applicable on this page (no license mention).
//
// EN prose stays close to the live template; /humanizer pro pass applies to
// FR/DE only. HTML markup inside *Html fields is preserved and emitted via
// <%- t.section.field %> in the template.
module.exports = {
  pageTitle: 'Hook0 - Data Processing Agreement (DPA)',
  pageDescription: 'Hook0 Data Processing Agreement covering GDPR compliance, data processing operations, security measures, and subprocessor management.',
  pageModified: '2026-06-27',
  hero: {
    eyebrow: 'Legal',
    title: 'Data Processing Agreement',
    subtitle: 'Our commitment to protecting your data under GDPR and data protection regulations.',
    lastUpdatedLabel: 'Last Update:',
    lastUpdatedDate: 'June 27, 2026',
  },
  preamble: {
    // [LEGAL-CORRECTION L107] Add a parties block making the controller / processor
    // identification explicit at the top of the document (art. 28(3) GDPR).
    title: 'Data Processing Terms of Service',
    partiesHtml: 'This Data Processing Agreement (the "<strong>DPA</strong>") is entered into between the Customer, acting as Data Controller, and <strong>FGRibreau SARL</strong>, a French limited liability company (Société à Responsabilité Limitée) with a share capital of 2,000 EUR, registered with the Trade and Companies Register of La Roche-sur-Yon under number 850 824 350, with its registered office at 3 rue de l\'Aubépine, 85110 Chantonnay, France, acting as Data Processor (referred to as "<strong>Hook0</strong>" or the "<strong>Processor</strong>").',
    p1: 'The purpose of this DPA is to reflect the parties\' agreement with regard to the processing of Personal Data in accordance with the requirements of Data Protection Regulations.',
    p2: 'In respect of the processing of Personal Data of the Customer by Hook0 under the Terms of Services, the parties acknowledge that the Customer is the Data Controller and Hook0 is the Data Processor and both agree to comply with all corresponding obligations as per the Data Protection Regulations.',
    p3: 'The Customer gives instructions to Hook0 to process such Personal Data on its behalf as it is necessary for the purposes of the Terms of Services as defined in Appendix 1 "Description of Personal Data processing". The Appendix 1 is filled out by the Customer and shall be updated if any change is made by the Customer.',
  },
  section1: {
    title: '1. Compliance with Data Protection Regulations',
    p1: 'Each party shall comply with its obligations under the Data Protection Regulations.',
    p2: 'All capitalized words in the DPA shall have the meaning ascribed to them in the GDPR, the Data Protection Regulations and in the Terms of Services.',
  },
  section2: {
    title: '2. Data Processing Operations under the DPA',
    p1: 'As a reminder, for every processing carried out under this DPA, the Customer shall:',
    items: [
      'Document the instructions related to Personal Data,',
      'Provide the information related to the processing to fill the Appendix 1 by contacting Hook0 via the support email address: <a href="mailto:support@hook0.com">support@hook0.com</a>.',
    ],
    p2: 'The Customer warrants to Hook0 that it is entitled to transfer the Personal Data to Hook0 and/or the Sub-processor(s) in full compliance with Data Protection Regulations, including as needed, compliance to any prior required formalities and Data Subject rights, such as information and/or consent when such is required under Data Protection Regulations.',
    p3: 'The Customer acknowledges that it is and shall remain solely responsible for determining the purposes and the means of Hook0\'s processing the Personal Data. The Data Controller remains solely responsible for the accuracy and adequacy of the aforementioned instructions. Any changes to the instructions given or the security measures that are required by the Customer, including in order to comply with applicable data protection laws, shall be agreed by the parties and/or via an amendment to this DPA. Any costs incurred by Hook0 in complying with such changes shall be borne by the Customer.',
    p4: 'The Customer undertakes that the Data Subjects have been informed or will be informed before the transfer of their Personal Data to Hook0 in the scope of the Services.',
    p5: 'The Product is not intended to process Special Categories of Personal Data. Therefore, the Customer undertakes to prevent any processing of Special Categories of Personal Data through the Product and the Services. However, at the Customer\'s request, processing of Special Categories of Personal Data may be performed by Hook0. In such case, the Processing shall be covered by a specific addendum to the DPA to be entered into between the Customer and Hook0.',
    p6: 'In case the Customer expressly requests the assistance of Hook0 for the fulfilment of its obligation under the Data Protection Regulations, then Hook0 shall address to the Customer the estimated costs for such assistance. Upon express acceptance of the estimated cost, Hook0 shall provide assistance pursuant to the instructions of the Customer and the terms of the present DPA.',
  },
  section3: {
    title: '3. Scope & Instructions',
    p1: 'Hook0 undertakes to:',
    items: [
      'solely process the Customer\'s Personal Data disclosed by the Customer as well as those collected or produced during the Terms of Services for the purpose(s) fulfilling its obligations under the Terms of Services and in compliance with the Customer\'s documented instructions, unless otherwise required by applicable Data Protection Regulations;',
      'ensure that any person acting under its authority, who has access to the Customer\'s Personal Data disclosed by the Customer as well as those collected or produced during the Terms of Services, will process those data solely for the purpose of fulfilling Hook0\'s obligations under this Terms of Services and on instructions from the Customer, unless required by applicable Data Protection Regulations;',
      'refrain from using Customer\'s Personal Data for any misappropriated, fraudulent or personal use, including for commercial purposes;',
      'immediately inform the Customer if, in its opinion, a Customer\'s instruction infringes applicable Data Protection Regulations.',
    ],
  },
  section4: {
    title: '4. Communication of Customer\'s Personal Data to Third Parties',
    p1: 'The Customer\'s Personal Data processed under the DPA shall not be subject to any assignment, lease, concession, communication or disclosure to a third party, including sub-Processors of Hook0, except otherwise required by the Terms of Services or by a legal or regulatory mandatory provision.',
    p2: 'In such a case, Hook0 shall inform the Customer of that legal requirement before Processing, unless that legal or regulatory mandatory provision prohibits such information on important grounds of public interest.',
  },
  section5: {
    title: '5. Sub-Processing',
    p1Html: 'With respect to the conditions referred to in paragraphs 2 and 4 of article 28 of GDPR for engaging another Data Processor (the "<strong>Sub-processor</strong>"), the Customer agrees that Hook0 may sub-process the Processing of the Customer\'s Personal Data.',
    p2Html: 'Notwithstanding the general consent given by the Customer, Hook0 shall inform the Customer of any intended changes concerning the addition or replacement of any Sub-processor within a reasonable time prior to implementation of such change, giving the Customer a reasonable opportunity to object before the change takes effect. The list of the sub-Processors under the authority of Hook0 is available to the Customer at <a href="./gdpr-subprocessors">Hook0 / GDPR Sub-processors</a>.',
    p3: 'Where Hook0 engages a Sub-processor who shall process the Customer\'s Personal Data, the same data protection obligations as set out in the DPA shall be imposed on the Sub-processor by Hook0.',
    p4: 'This agreement must in particular provide for an obligation of the Sub-processor to provide sufficient guarantees to implement appropriate technical and organisational measures in such a manner that the Processing will meet the requirements of Data Protection Regulations and of the DPA.',
  },
  section6: {
    title: '6. Transfer of Customer\'s Personal Data outside the European Economic Area (EEA)',
    // [LEGAL-CORRECTION L169] The legacy line said the Personal Data are "located
    // in France or in the European Union" without qualification. Cloudflare (USA)
    // is in the CDN / DDoS chain (disclosed on the subprocessors page), so the
    // statement must distinguish the webhook data plane (EU only) from the
    // ancillary edge layer (US, framed by SCC 2021 + TIA / EU-US DPF) to stay
    // truthful (L121-1 C. conso) and aligned with gdpr-subprocessors.js.
    p1Html: 'Hook0 warrants that the webhook data plane (Customer\'s webhook payloads, database, application backups) is located in France or within the European Economic Area (EEA). The ancillary edge layer (CDN and DDoS protection provided by Cloudflare, Inc.) involves transfers to the United States, framed by the Standard Contractual Clauses 2021 adopted by the European Commission and a documented Transfer Impact Assessment, and where applicable by the EU-US Data Privacy Framework. The full subprocessor list and applicable transfer mechanisms are kept up to date at <a href="./gdpr-subprocessors">Hook0 / GDPR Sub-processors</a>.',
    p2Html: 'At the request of the Customer and upon instructions, Hook0 shall store or transfer Personal Data to other Hook0 entities and/or to Sub-processors located in countries outside the EEA ("Third Countries"). In that case and when Third Countries have not been subject to an adequacy decision of the European Commission, Hook0 undertakes that the transfer will be carried out in accordance with the Data Protection Regulations and will be subject to appropriate safeguards to guarantee a level of protection equivalent to the one guaranteed by the Data Protection Regulations, such as the signing of the Standard Contractual Clauses adopted by the European Commission and available at <a href="https://commission.europa.eu/law/law-topic/data-protection/international-dimension-data-protection/standard-contractual-clauses-scc_en">commission.europa.eu</a>.',
    p3: 'The Customer hereby mandates Hook0 to sign on its behalf the Standard Contractual Clauses with Hook0 entities and sub-Processors located in Third Countries.',
    p4: 'At the request of the Customer, Hook0 agrees to assist the Customer to perform a Transfer Impact Assessment to identify any gaps between the Data Protection Regulations and the laws of the Third Country and to implement the necessary supplementary measures to guarantee a level of protection equivalent to the one guaranteed by the Data Protection Regulations.',
  },
  section7: {
    title: '7. Security Measures and Confidentiality of the Processing',
    p1: 'Hook0 shall take, insofar as this is relevant to the provision of the Services or compliance with its other obligations in the DPA, adequate measures to ensure a level of security of the Customer\'s Personal Data appropriate to the risk and to take into account the principles of data protection by design and by default in the execution of the DPA.',
    p2: 'Hook0 undertakes to:',
    items: [
      'implement all appropriate technical and organisational measures in order to protect Personal Data against accidental or unlawful destruction, loss, alteration, unauthorized disclosure or access to Personal Data transmitted, stored or otherwise processed and, in particular, all the measures mentioned in Appendix 2;',
      'respect all the instructions communicated by the Customer in relation to security and confidentiality measures that can be reasonably implemented;',
      'make Customer\'s Personal Data accessible and consultable only to duly authorised persons;',
      'ensure confidentiality of the Customer\'s Personal Data processed under the DPA and that all the persons authorised to process the Customer\'s Personal Data under the authority of Hook0 (including employees and sub-Processors) undertake to respect the confidentiality of the said data or are under an appropriate statutory obligation of confidentiality.',
    ],
  },
  section8: {
    // [ISMS-SYNC L192] Aligned with business-continuity-disaster-recovery.md:
    // 72h applies to both authority notification (CNIL) and customer notification
    // (Customer acting as Data Controller). Wording adjusted to track the BCDR
    // procedure (legal@hook0.com as the privacy contact, individual notification
    // under art. 34 GDPR when high risk).
    title: '8. Personal Data Breach Notification',
    p1Html: 'Hook0 shall notify the Customer of any Personal Data Breach without undue delay, and in any event within <strong>72 hours</strong> of becoming aware of the breach, in accordance with Article 33 of the GDPR, and in writing after it becomes aware of a Personal Data Breach. When the information is available to Hook0, such notification shall:',
    items: [
      'describe the nature of the Personal Data Breach including where possible, the categories and approximate number of the concerned Data Subjects and the categories and approximate number of Personal Data concerned;',
      'communicate the name and contact details of the privacy contact (<a href="mailto:legal@hook0.com">legal@hook0.com</a>) or other contact point where more information can be obtained;',
      'describe the likely consequences of the Personal Data Breach;',
      'describe the measures taken or proposed to be taken to address the Personal Data Breach, including, where appropriate, measures to mitigate its possible adverse effects.',
    ],
    p2: 'Where, and in so far as, it is not possible to provide the information at the same time, the information may be provided in phases without undue further delay.',
    p3: 'At the request of the Customer, Hook0 also undertakes to provide the Customer with reasonable assistance and co-operation to notify the Personal Data Breach to the competent Data Protection Authority and to communicate such Personal Data Breach to the Data Subjects pursuant to Article 34 of the GDPR, in compliance with applicable Data Protection Regulations.',
  },
  section9: {
    title: '9. Rights of the Data Subjects',
    p1: 'Based on the nature of the Personal Data Processing activities, Hook0 undertakes to:',
    items: [
      'promptly notify the Customer of any request or complaint received relating to data protection of Customer\'s Personal Data;',
      'at the request of the Customer, provide the Customer with reasonable assistance and co-operation, to allow the Customer to respond (i) to requests presented by Data Subjects for exercising their rights (right of access, rights to rectification, erasure, limitation, portability and objection), or (ii) to respond to the competent data protection authorities\' requests or the Customer\'s Data Protection Officer requests; in particular, implement appropriate technical and organisational measures to allow the Customer to promptly satisfy in writing to any request for information of the Customer;',
      'duly provide the Data Subjects with the adequate information on the Personal Data Processing operations carried out concerning their Personal Data under the Terms of Services, where requested by and at the expense of the Customer.',
    ],
  },
  section10: {
    title: '10. Data Protection Impact Assessment',
    p1: 'At the request of the Customer, Hook0 undertakes to provide the Customer with reasonable assistance and co-operation to carry out an assessment of the impact of the Personal Data Processing operations carried out under the present DPA on the protection of Personal Data and to consult the competent data protection authorities, where necessary and at the expense of the Customer (based on a time and materials fee).',
  },
  section11: {
    title: '11. Retention, Return or Destruction of the Personal Data',
    p1: 'The Customer remains solely responsible for implementing and managing Personal Data retention periods, and undertakes to use the Product accordingly.',
    p2: 'Without prejudice to the applicable laws and regulations, Hook0 undertakes to, at the end of the Terms of Services:',
    items: [
      'return or destroy, at the Customer\'s request, all Customer\'s Personal Data in an automated or manual way, following processes and prescriptions previously agreed between the Parties;',
      'delete all existing copies of the Personal Data unless and to the extent that Hook0 is required to retain copies of the Personal Data in accordance with applicable laws (in particular billing and invoicing records, retained for 10 years pursuant to French tax law);',
      'certify the destruction of the Personal Data in writing.',
    ],
  },
  section12: {
    title: '12. Documentation and Audit',
    p1: 'Upon prior written notice of thirty (30) business days sent by the Customer, Hook0 shall disclose to the Customer the information strictly necessary to demonstrate compliance with the obligations laid down in this Terms of Services.',
    p2: 'At the request of the Customer and once a year, Hook0 undertakes to allow for and contribute to reasonable audits, including inspections, conducted by or on behalf of the Customer, for the purposes of assessing Hook0\'s compliance with the Data Protection Regulations and the provisions of the DPA.',
    p3: 'Hook0 also undertakes to allow for and contribute to audits conducted by competent Data Protection Authorities.',
    p4: 'The Customer shall have no right to view or access any systems, data, records or other information relating or pertaining to Hook0\'s other customers.',
    p5: 'Any such audit by or on behalf of the Customer shall be conducted at its own costs. The Customer shall provide Hook0 with a copy of the audit report.',
    p6: 'In the event that the Customer is subject to an investigation or a request for information by a competent data protection authority and concerning any of the processing operations carried out by Hook0 on behalf of the Customer, the Customer undertakes to inform Hook0 as soon as possible and to satisfy such investigation or request, to the best of its ability, at the expense of the Customer, and in accordance with the procedures adopted by the data protection authority.',
    p7: 'The Customer undertakes to comply with any confidentiality provisions, policies and/or site rules Hook0 may notify to the Customer in relation to the audit.',
  },
  appendix1: {
    title: 'Appendix 1 - Personal Data Processing Activities Carried Out by Hook0 on Behalf of the Customer',
    rows: [
      {
        label: 'Data Controller',
        valueHtml: 'The Customer (as identified in the Terms of Service).',
      },
      {
        // [LEGAL-CORRECTION L256] Restate the processor identity in full at the
        // Appendix 1 level (art. 28(3) GDPR) — capital, RCS, VAT, registered office.
        label: 'Data Processor',
        valueHtml: 'FGRibreau SARL, a French SARL with a share capital of 2,000 EUR, registered with the RCS of La Roche-sur-Yon under number 850 824 350, VAT FR27850824350, registered office at 3 rue de l\'Aubépine, 85110 Chantonnay, France.',
      },
      {
        label: 'Nature of the Processing operations',
        valueHtml: '<ul><li>Receiving, storing, and forwarding webhook events on behalf of the Customer</li><li>Retry management for failed webhook deliveries</li><li>Logging and monitoring of webhook delivery attempts</li><li>User authentication and access management for the Hook0 platform</li><li>Billing and subscription management (via Stripe)</li></ul>',
      },
      {
        label: 'Purpose(s) of Processing',
        valueHtml: 'Provision of the Hook0 webhook-as-a-service platform as described in the Terms of Service.',
      },
      {
        label: 'Name and contact details of the Customer\'s Data Protection Officer (if applicable)',
        valueHtml: '<em>[to be completed by the Customer]</em>',
      },
      {
        label: 'Category/ies of Personal Data',
        valueHtml: 'Email addresses, names, IP addresses, webhook payload data (content determined by Customer), authentication tokens, billing information (processed by Stripe).<br><br><strong>Sensitive Data:</strong> None by default. The Customer is responsible for ensuring that webhook payloads do not contain special categories of data unless agreed upon in writing.<br><br>At the Customer\'s request, processing of Special Categories of Personal Data may be performed by Hook0. In such case, the Processing shall be covered by a specific addendum to the DPA to be entered into between the Customer and Hook0.',
      },
      {
        label: 'Category/ies of Data Subjects',
        valueHtml: 'Customer\'s end users whose data is transmitted via webhooks; Customer\'s authorized users of the Hook0 platform.',
      },
      {
        // [LEGAL-CORRECTION L292] Reword location row to mirror section 6 and the
        // subprocessors page: the webhook data plane is in France / EEA, the CDN
        // layer involves US transfers under SCC 2021 + TIA / EU-US DPF.
        label: 'Location(s) of Processing operations',
        valueHtml: 'Webhook data plane: France / EEA.<br>CDN and DDoS protection: United States (Cloudflare, Inc.), framed by SCC 2021 + TIA and, where applicable, the EU-US Data Privacy Framework.<br><br>If the Customer requests the Personal Data to be located outside the EEA, such Processing shall be covered by a separate agreement between the Customer and Hook0.<br><br>See: <a href="./gdpr-subprocessors">Hook0 / GDPR Sub-processors</a>',
      },
      {
        label: 'Identity of the sub-Processor(s)',
        valueHtml: 'See: <a href="./gdpr-subprocessors">Hook0 / GDPR Sub-processors</a>',
      },
      {
        label: 'Frequency of Processing',
        valueHtml: 'Continuous, automated processing.',
      },
      {
        // [ISMS-SYNC L304] Aligned with information-retention-policy.md:
        // account data is retained for the duration of the service contract + 30
        // days after account deletion; webhook event data is retained per plan
        // (Developer 7d / Startup 14d / Pro 30d / Enterprise custom); billing
        // records 10 years (French tax law).
        label: 'Duration of Processing operations',
        valueHtml: 'For the duration of the Terms of Service, plus 30 days after account deletion (account data). Webhook event data retention depends on the Customer\'s plan, with 7 days on Developer, 14 days on Startup, 30 days on Pro, and a custom duration on Enterprise. Billing records are retained for 10 years pursuant to French tax law.',
      },
    ],
  },
  appendix2: {
    title: 'Appendix 2 - Appropriate Technical and Organisational Measures Implemented',
    intro: 'The following technical and organisational measures are implemented by Hook0 in order to protect Personal Data against accidental or unlawful destruction, loss, alteration, unauthorized disclosure or access to Personal Data transmitted, stored or otherwise processed:',
    groups: [
      {
        // [ISMS-SYNC L312] Cross-checked with backup-policy.md (daily backups,
        // 30-day retention, multi-region S3-like distributed storage on Clever
        // Cloud fr-par, encryption, integrity controls). The "ISO 27001-certified
        // security programme" claim about Clever Cloud datacentres has been
        // softened to "their documented security programme" because Hook0 does
        // not control the wording of third-party certifications.
        title: 'Infrastructure Security (managed by Clever Cloud SAS)',
        items: [
          'Application hosted on Clever Cloud SAS infrastructure in France (EU);',
          'Database encryption at rest (managed by Clever Cloud);',
          'TLS 1.2+ encryption for all data in transit (rustls, with post-quantum cryptography support);',
          'Automated daily backups with 30-day retention, stored in a multi-region distributed system (S3-like) on Clever Cloud fr-par; backup integrity is verified and restoration is tested monthly;',
          'CDN and DDoS protection provided by Cloudflare, Inc. (USA), framed by SCC 2021 + TIA and, where applicable, the EU-US Data Privacy Framework;',
          'Physical access controls for datacentre facilities are delegated to Clever Cloud SAS in accordance with their documented security programme.',
        ],
      },
      {
        // [ISMS-SYNC L322] Cross-checked with password-policy.md (Argon2 with
        // default parameters, unique salt, no plaintext) and access-control-policy.md
        // (MFA infra yes / MFA customer accounts not yet; planned). Wording kept
        // honest per CLAUDE.local.md "MFA infra (yes), MFA clients (no, future work)".
        title: 'Application Security',
        items: [
          'Password hashing with Argon2 (memory-hard, resistant to GPU and ASIC attacks; unique random salt per password; never stored in plaintext or with reversible encryption);',
          'Capability-based authorization tokens (Biscuit);',
          'Role-Based Access Control (RBAC) for platform access;',
          'Automatic session expiration.',
        ],
        // [ISMS-SYNC L329] Mirrors access-control-policy.md MFA section verbatim.
        noteHtml: '<strong>Note on multi-factor authentication (MFA):</strong> MFA is enforced for all infrastructure access (Clever Cloud, GitLab, Stripe). MFA for individual Hook0 customer accounts is not yet implemented at the application level and is planned for a future release. Until customer-facing MFA is available, strong password requirements (Argon2 hashing, minimum complexity) and session expiration provide baseline protection.',
      },
      {
        // [ISMS-SYNC L331] Cross-checked with secure-development-policy.md:
        // SAST (GitLab template), DAST (GitLab template), Trivy, osv-scanner,
        // GitLab secret detection, Clippy with -D warnings, cargo fmt --check.
        title: 'Development Security',
        items: [
          'All code changes require peer review via merge requests;',
          'Automated CI/CD pipeline including:<ul><li>Static Application Security Testing (SAST, GitLab template);</li><li>Dynamic Application Security Testing (DAST, GitLab template);</li><li>Container and filesystem scanning (Trivy);</li><li>Dependency vulnerability scanning (osv-scanner);</li><li>Secret detection (GitLab template).</li></ul>',
          'Strict code linting (Clippy with warnings treated as errors) and consistent formatting (cargo fmt --check) enforced in CI.',
        ],
      },
      {
        // [ISMS-SYNC L346] Cross-checked with secure-engineering-policy.md
        // (Sentry, OpenTelemetry, BetterUptime) and responsible-disclosure-policy.md.
        title: 'Monitoring and Incident Response',
        items: [
          'Error tracking via Sentry;',
          'Distributed tracing via OpenTelemetry (OTLP export);',
          'Uptime monitoring via BetterUptime with public status page;',
          'Personal data breach notification within 72 hours pursuant to Article 33 GDPR (see section 8);',
          'Responsible disclosure policy with PGP-secured reporting.',
        ],
      },
      {
        // [ISMS-SYNC L355] Cross-checked with information-classification-policy.md
        // (Public/Internal/Confidential/Sensitive), penetration-testing-policy.md
        // (annual or after major architectural changes), supplier-policy.md and
        // code-of-conduct.md.
        title: 'Organisational Measures',
        items: [
          'Information classification policy (Public, Internal, Confidential, Sensitive);',
          'NDAs required for all personnel;',
          'Security awareness practices;',
          'Need-to-know access principle;',
          'MFA enforced for infrastructure access (Clever Cloud, GitLab, Stripe);',
          'Penetration testing conducted annually or after major architectural changes.',
        ],
      },
      {
        // [ISMS-SYNC L365] Cross-checked with information-retention-policy.md:
        // service-specific retention periods made explicit (per-plan webhook event
        // retention, account data duration + 30d, billing 10 years French tax law,
        // server logs 30d minimum). The legacy "7-30 days" range is replaced by
        // the precise per-plan breakdown.
        title: 'Data Retention',
        items: [
          'Webhook event data, per Customer plan, with 7 days on Developer, 14 days on Startup, 30 days on Pro, and a custom duration on Enterprise;',
          'Account data (username, email, hashed password, API keys), retained for the duration of the service contract plus 30 days after account deletion;',
          'Billing and invoicing records, retained for 10 years (French tax law, art. L102 B Livre des procédures fiscales);',
          'Server logs, 30 days minimum, then automatic rotation and deletion;',
          'Support communications, 3 years from the last exchange (statutory limitation period for contractual claims).',
        ],
      },
    ],
  },
};
