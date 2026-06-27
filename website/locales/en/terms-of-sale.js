// Per-page strings for terms-of-sale (EN base — Terms of Sale / CGV B2B).
//
// Source: src/terms-of-sale.ejs (v2026-04-24). Inline legal-reviewer audit
// applied before extraction; each correction is flagged inline below with
// [LEGAL-CORRECTION L#] referencing the original-source line number. The
// "Last Updated" date is bumped to 2026-06-27 to reflect these corrections.
//
// Hard legal facts (CLAUDE.md / CLAUDE.local.md) kept verbatim:
//   - Entity: FGRibreau SARL (capital EUR 2,000, RCS La Roche-sur-Yon 850 824 350,
//     VAT FR27850824350, registered office 3 rue de l'Aubepine, 85110 Chantonnay)
//   - Director of publication: David Sferruzza
//   - Hosting: Clever Cloud SAS (France) + CDN Cloudflare Inc. (USA) disclosed
//   - B2B only (no consumer right of withdrawal)
//   - Prices HT (excl. VAT); late-payment penalties = 3x ECB legal rate + EUR 40
//     (art. L441-10 + D441-5 C. com.)
//   - Jurisdiction: tribunaux de La Roche-sur-Yon (art. 48 CPC, merchants)
//   - Custom SLA only (no implicit guaranteed SLA)
//   - SSPL framing: "source-available (SSPL-1.0)" — NEVER "open-source"
//
// EN prose stays close to the live template; the /humanizer pro pass applies
// to FR/DE only. HTML markup inside body fields is preserved and emitted via
// <%- t.section.field %> in the template.
module.exports = {
  pageTitle: 'Hook0 - Terms of Sale',
  pageDescription: 'Terms of Sale for Hook0 Webhooks-as-a-Service. Pricing, payment terms, invoicing, and cancellation policy for Cloud and On-Premise plans.',
  pageModified: '2026-06-27',
  hero: {
    eyebrow: 'Legal',
    title: 'Terms of Sale',
    subtitle: 'Commercial terms governing the purchase of Hook0 plans and services',
    lastUpdatedLabel: 'Last Updated:',
    lastUpdatedDate: 'June 27, 2026',
  },
  intro: {
    p1Html: 'These Terms of Sale govern all orders and subscriptions placed with FGRibreau SARL, a French limited liability company (societe a responsabilite limitee) with a share capital of EUR 2,000, registered at the Registre du commerce et des societes de La Roche-sur-Yon under number 850 824 350, with registered office at 3 rue de l\'Aubepine, 85110 Chantonnay, France, VAT number FR27850824350 (hereinafter "Hook0" or "we"), for access to the Hook0 platform and related services. The director of publication is David Sferruzza.',
    p2Html: 'These Terms of Sale apply exclusively to business-to-business (B2B) transactions. They are incorporated by reference into and supplement the <a href="/terms" class="text-green-400 hover:text-green-300 transition-colors">Terms of Service</a>. In the event of a conflict between these Terms of Sale and the Terms of Service, these Terms of Sale prevail with respect to commercial and billing matters.',
    p3Html: 'By placing an order or activating a paid subscription, the customer expressly accepts these Terms of Sale in full.',
  },
  sections: [
    // Section 1
    {
      id: 'scope',
      title: '1. Scope and Applicability',
      paragraphs: [
        '<strong class="text-white">1.1.</strong> These Terms of Sale apply to any subscription to Hook0 Cloud plans (Developer, Startup, Pro, Enterprise) and On-Premise plans (Self-hosted, Pro, Enterprise), regardless of the channel through which the order is placed.',
        '<strong class="text-white">1.2.</strong> These Terms of Sale apply exclusively to professional customers (businesses, associations, public entities). They do not apply to consumers within the meaning of the French Consumer Code.',
        '<strong class="text-white">1.3.</strong> Any general purchasing conditions of the customer are expressly excluded and have no effect, even if communicated to Hook0 after acceptance of these Terms of Sale, unless Hook0 has expressly accepted them in writing.',
      ],
    },
    // Section 2
    {
      id: 'pricing',
      // [LEGAL-CORRECTION L98] Self-hosted line re-qualified as source-available (SSPL-1.0).
      // Original src said "Free (SSPL license)" without the source-available qualifier — kept "Free" but added explicit SSPL-1.0 framing.
      title: '2. Pricing',
      paragraphs: [
        '<strong class="text-white">2.1.</strong> All prices are stated exclusive of taxes (excl. VAT, "HT"). Applicable VAT or equivalent taxes are added automatically at the time of invoicing in accordance with the applicable legislation, based on the customer\'s country of establishment.',
        '<strong class="text-white">2.2.</strong> Current prices for all plans are published at <a href="/pricing" class="text-green-400 hover:text-green-300 transition-colors">hook0.com/pricing</a> and are incorporated by reference into these Terms of Sale. Published prices are subject to change in accordance with Section 9 of these Terms of Sale.',
        '<strong class="text-white">2.3. Cloud plans</strong> — indicative pricing at the date of last update:',
      ],
      cloudPlans: [
        '<strong class="text-white">Developer</strong>: Free, 100 events/day, 7-day retention.',
        '<strong class="text-white">Startup</strong>: EUR 59/month excl. VAT, 30,000 events/day; overage at EUR 0.003/event.',
        '<strong class="text-white">Pro</strong>: EUR 190/month or EUR 1,824/year excl. VAT, 100,000 events/day; overage at EUR 0.0001/event.',
        '<strong class="text-white">Enterprise</strong>: Custom quote.',
      ],
      paragraphsBeforeOnPremise: [
        '<strong class="text-white">2.4. On-Premise plans</strong> — indicative pricing at the date of last update:',
      ],
      onPremisePlans: [
        '<strong class="text-white">Self-hosted</strong>: Free, source-available under the Server Side Public License v1 (SSPL-1.0).',
        '<strong class="text-white">Pro</strong>: EUR 1,000 setup fee + EUR 500/month or EUR 6,000/year excl. VAT.',
        '<strong class="text-white">Enterprise</strong>: Custom quote.',
      ],
      paragraphsAfter: [
        '<strong class="text-white">2.5.</strong> The prices listed in Sections 2.3 and 2.4 are provided for information purposes only. The binding prices are those displayed on <a href="/pricing" class="text-green-400 hover:text-green-300 transition-colors">hook0.com/pricing</a> at the time the order is placed, or as set out in a specific quote for Enterprise customers.',
      ],
    },
    // Section 3
    {
      id: 'ordering',
      title: '3. Ordering and Subscription',
      paragraphs: [
        '<strong class="text-white">3.1.</strong> Subscriptions to self-service plans (Developer, Startup, Pro) are placed directly via the Hook0 application at <a href="https://app.hook0.com" class="text-green-400 hover:text-green-300 transition-colors">app.hook0.com</a>. The customer selects the desired plan and provides valid payment information.',
        '<strong class="text-white">3.2.</strong> Enterprise and On-Premise Pro subscriptions require a custom quote. The customer may request a quote by contacting <a href="mailto:sales@hook0.com" class="text-green-400 hover:text-green-300 transition-colors">sales@hook0.com</a>. The contract is formed upon written acceptance of the quote by both parties.',
        '<strong class="text-white">3.3.</strong> For self-service plans, the contract is formed at the time the customer confirms the order in the application and Hook0 activates the selected plan.',
      ],
    },
    // Section 4
    {
      id: 'invoicing',
      title: '4. Invoicing',
      paragraphs: [
        '<strong class="text-white">4.1.</strong> Invoicing for self-service plans is handled automatically through Stripe, Hook0\'s payment service provider. Invoices are generated at the start of each billing period (monthly or annual, according to the selected plan) and are available in the customer\'s Stripe billing portal.',
        '<strong class="text-white">4.2.</strong> For Enterprise and On-Premise Pro plans, invoices are issued by Hook0 directly and sent to the billing address provided by the customer at the time of order.',
        '<strong class="text-white">4.3.</strong> Annual subscriptions are invoiced in full at the start of the subscription year.',
        '<strong class="text-white">4.4.</strong> Overage charges, where applicable, are calculated at the end of each billing period and invoiced with the following period\'s invoice or separately, at Hook0\'s discretion.',
      ],
    },
    // Section 5
    {
      id: 'payment',
      title: '5. Payment Terms',
      paragraphs: [
        '<strong class="text-white">5.1. Self-service plans (Developer, Startup, Pro).</strong> Payment is due immediately upon invoicing by direct debit from the payment card registered with Stripe. By subscribing, the customer authorizes Hook0 to charge the registered payment method on a recurring basis in accordance with the selected billing cycle.',
        '<strong class="text-white">5.2. Enterprise and On-Premise Pro plans.</strong> Unless otherwise specified in the applicable quote or order form, invoices are payable within thirty (30) days of the invoice date.',
        '<strong class="text-white">5.3.</strong> All payments are made in euros (EUR). Bank transfer fees and currency conversion costs, if any, are borne by the customer.',
      ],
    },
    // Section 6
    {
      id: 'late-payment',
      // [LEGAL-CORRECTION L169] Late-payment rate aligned with peer terms.js: "three times the legal interest rate published by the European Central Bank" — explicit ECB attribution per L441-10 reading.
      title: '6. Late Payment',
      paragraphs: [
        '<strong class="text-white">6.1.</strong> In accordance with article L441-10 of the French Code de commerce, any amount not paid by the due date automatically and without prior reminder gives rise to late-payment penalties calculated at a rate equal to three (3) times the legal interest rate published by the European Central Bank in force, applied to the outstanding amount from the due date until the date of actual payment.',
        '<strong class="text-white">6.2.</strong> In addition, pursuant to article D441-5 of the French Code de commerce, a flat-rate recovery indemnity of forty euros (EUR 40) is due by the customer for each unpaid invoice, in addition to any late-payment penalties. Where the recovery costs actually incurred by Hook0 exceed this amount, Hook0 reserves the right to claim additional compensation on supporting evidence.',
        '<strong class="text-white">6.3.</strong> Without prejudice to the foregoing, Hook0 reserves the right to suspend or restrict the customer\'s access to the service, without prior notice and without liability, in the event of non-payment of any amount due after a grace period of seven (7) calendar days following the due date. Service suspension does not release the customer from its payment obligations.',
      ],
    },
    // Section 7
    {
      id: 'overage',
      title: '7. Overage Charges',
      paragraphs: [
        '<strong class="text-white">7.1.</strong> Each paid plan includes a daily event quota as set out in Section 2.3. When usage exceeds the included quota, overage charges apply at the per-event rate specified for the customer\'s plan. For paid plans, event ingestion is not interrupted when the daily quota is exceeded, ensuring uninterrupted service delivery.',
        '<strong class="text-white">7.2.</strong> For the Developer (free) plan, no overage billing applies. Events exceeding the daily quota are blocked until the quota resets at midnight UTC the following day.',
        '<strong class="text-white">7.3.</strong> Overage charges are calculated automatically and invoiced as described in Section 4.4. The customer accepts that usage of the service above the included quota constitutes an implicit order for overage capacity at the applicable per-event rate.',
        '<strong class="text-white">7.4.</strong> The customer can monitor real-time event consumption through the organization dashboard in the Hook0 application. Hook0 sends email notifications when daily consumption approaches the quota threshold.',
      ],
    },
    // Section 8
    {
      id: 'plan-changes',
      title: '8. Plan Changes',
      paragraphs: [
        '<strong class="text-white">8.1. Upgrade.</strong> A plan upgrade (switch to a higher-tier plan) takes effect immediately upon confirmation. The customer is billed the pro-rated difference for the remainder of the current billing period.',
        '<strong class="text-white">8.2. Downgrade.</strong> A plan downgrade (switch to a lower-tier plan) takes effect at the end of the current billing period. The customer retains access to the current plan\'s features and quotas until that date. No refund is issued for the remaining period.',
        '<strong class="text-white">8.3.</strong> Plan changes can be initiated from the Hook0 application or by contacting <a href="mailto:support@hook0.com" class="text-green-400 hover:text-green-300 transition-colors">support@hook0.com</a>.',
      ],
    },
    // Section 9
    {
      id: 'price-changes',
      title: '9. Price Changes',
      paragraphs: [
        '<strong class="text-white">9.1.</strong> Hook0 reserves the right to modify its pricing at any time, subject to thirty (30) days\' prior written notice sent to the customer\'s registered email address or published on the Hook0 website.',
        '<strong class="text-white">9.2.</strong> New prices apply to the first billing period commencing after the end of the notice period. Price changes have no retroactive effect on periods already billed.',
        '<strong class="text-white">9.3.</strong> If the customer does not accept the new pricing, it may cancel its subscription before the new prices take effect in accordance with Section 10. Continued use of the service after the effective date of the price change constitutes acceptance of the new prices.',
      ],
    },
    // Section 10
    {
      id: 'cancellation',
      title: '10. Cancellation and Termination',
      paragraphs: [
        '<strong class="text-white">10.1.</strong> The customer may cancel its subscription at any time by sending a request to <a href="mailto:support@hook0.com" class="text-green-400 hover:text-green-300 transition-colors">support@hook0.com</a>.',
        '<strong class="text-white">10.2.</strong> Cancellation takes effect at the end of the current billing period. The customer retains access to the subscribed plan until that date.',
        '<strong class="text-white">10.3. No refund policy.</strong> Periods already billed and paid are non-refundable, regardless of the reason for cancellation and regardless of whether the service was actually used during that period. This policy applies to both monthly and annual subscriptions.',
        '<strong class="text-white">10.4. Data retention after cancellation.</strong> Customer data is retained for thirty (30) days following the end of the subscription, during which the customer may request an export of its data by contacting <a href="mailto:support@hook0.com" class="text-green-400 hover:text-green-300 transition-colors">support@hook0.com</a>. After this thirty-day period, all customer data is permanently deleted.',
        '<strong class="text-white">10.5.</strong> Hook0 may terminate a subscription immediately and without refund in the event of material breach of the Terms of Service by the customer, including but not limited to non-payment, fraudulent use, or violation of acceptable use policies.',
      ],
    },
    // Section 11
    {
      id: 'free-plan',
      // [LEGAL-CORRECTION L260] Re-emphasized "no implicit SLA" — Custom SLA only is the firm rule (CLAUDE.md).
      title: '11. Free Plan',
      paragraphs: [
        '<strong class="text-white">11.1.</strong> The Developer plan is provided free of charge. It does not constitute a commercial commitment on the part of Hook0 and may be modified, limited, or discontinued at Hook0\'s discretion. No service-level agreement is attached to the Developer plan; a custom SLA may be negotiated for Enterprise customers only.',
        '<strong class="text-white">11.2.</strong> Hook0 will provide at least ninety (90) days\' prior notice before discontinuing the free Developer plan or materially reducing its included quotas. Notice is sent to the registered email address and/or published on the Hook0 website.',
      ],
    },
    // Section 12
    {
      id: 'taxes',
      title: '12. Taxes',
      paragraphs: [
        '<strong class="text-white">12.1.</strong> All prices are stated exclusive of taxes (excl. VAT, "HT"). Applicable VAT or equivalent indirect taxes are added to the invoice amount and collected by Hook0 or by Stripe, in accordance with the tax rules applicable in the customer\'s country of establishment.',
        '<strong class="text-white">12.2.</strong> Business customers established in a European Union member state other than France may be exempt from French VAT if they provide a valid intra-community VAT number. It is the customer\'s responsibility to provide accurate and up-to-date tax information in their account settings.',
        '<strong class="text-white">12.3.</strong> Customers established outside the European Union are responsible for any import duties, withholding taxes, or other levies applicable in their country. Hook0 does not collect taxes on behalf of foreign tax authorities except where required by applicable law.',
      ],
    },
    // Section 13
    {
      id: 'subprocessors',
      // [LEGAL-CORRECTION new section] Infrastructure disclosure added for B2B procurement parity with Terms of Service §3.4 (Cloudflare disclosure obligation).
      title: '13. Infrastructure and Subprocessors',
      paragraphs: [
        '<strong class="text-white">13.1.</strong> Hook0 Cloud relies on third-party infrastructure providers, including hosting provided by Clever Cloud SAS (France) and content delivery and edge security provided by Cloudflare Inc. (United States). The current list of subprocessors and their locations is published at <a href="/gdpr-subprocessors" class="text-green-400 hover:text-green-300 transition-colors">hook0.com/gdpr-subprocessors</a> and incorporated by reference.',
        '<strong class="text-white">13.2.</strong> Transfers of personal data to subprocessors outside the European Economic Area are governed by the <a href="/data-processing-addendum" class="text-green-400 hover:text-green-300 transition-colors">Data Processing Addendum</a>, which includes the applicable transfer mechanism (Standard Contractual Clauses or, where applicable, the EU-US Data Privacy Framework).',
      ],
    },
    // Section 14
    {
      id: 'law',
      // [LEGAL-CORRECTION L294] Jurisdiction corrected: tribunaux de La Roche-sur-Yon (Chantonnay is in ressort 85), NOT Nantes (44). Cited art. 48 CPC for merchants.
      title: '14. Applicable Law and Jurisdiction',
      paragraphs: [
        '<strong class="text-white">14.1.</strong> These Terms of Sale are governed exclusively by French law. The application of the United Nations Convention on Contracts for the International Sale of Goods (CISG) is expressly excluded.',
        '<strong class="text-white">14.2.</strong> In accordance with article 48 of the French Code of Civil Procedure, applicable between merchants, the parties agree to submit any dispute arising out of or in connection with these Terms of Sale to the exclusive jurisdiction of the competent courts of La Roche-sur-Yon (tribunaux de La Roche-sur-Yon), France, where Hook0 has its registered jurisdiction, notwithstanding multiple defendants or third-party proceedings, and subject to any mandatory jurisdiction rules applicable to the subject matter of the dispute. The parties first endeavour to reach an amicable resolution; if no amicable resolution is reached within thirty (30) days of written notice of the dispute, either party may submit the dispute to the courts identified above.',
      ],
    },
    // Section 15
    {
      id: 'contact',
      title: '15. Contact',
      lead: 'For any question relating to these Terms of Sale, invoicing, or commercial matters, please contact:',
      contactItems: [
        '<strong class="text-white">Legal matters:</strong> <a href="mailto:legal@hook0.com" class="text-green-400 hover:text-green-300 transition-colors">legal@hook0.com</a>',
        '<strong class="text-white">Billing and subscriptions:</strong> <a href="mailto:support@hook0.com" class="text-green-400 hover:text-green-300 transition-colors">support@hook0.com</a>',
        '<strong class="text-white">Enterprise sales:</strong> <a href="mailto:sales@hook0.com" class="text-green-400 hover:text-green-300 transition-colors">sales@hook0.com</a>',
        '<strong class="text-white">Registered office:</strong> FGRibreau SARL, 3 rue de l\'Aubepine, 85110 Chantonnay, France',
      ],
    },
  ],
};
