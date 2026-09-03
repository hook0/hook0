---
title: Transfer Impact Assessment
description: Post-Schrems II Transfer Impact Assessment for Hook0 Cloud's sub-processors established outside the EEA — Stripe, Cloudflare, Sentry, and Postmark — covering the applicable transfer mechanism, supplementary measures, and residual risk.
keywords: [gdpr, transfer impact assessment, schrems ii, standard contractual clauses, data privacy framework, international transfers, article 46]
---

## Purpose

This Transfer Impact Assessment (TIA) documents, for each Hook0 Cloud sub-processor established outside the European Economic Area, the nature of the personal data transferred, the applicable transfer mechanism under Chapter V GDPR, the supplementary measures in place, and Hook0's conclusion on the residual level of risk.

This assessment implements the methodology set out by the European Data Protection Board in its **Recommendations 01/2020 on measures that supplement transfer tools** (adopted 18 June 2021), which followed the Court of Justice of the European Union's judgment of 16 July 2020 in Case C-311/18, *Data Protection Commissioner v Facebook Ireland Ltd and Maximillian Schrems* ("**Schrems II**"). That judgment invalidated the EU-US Privacy Shield and held that reliance on Standard Contractual Clauses alone is not sufficient where the law or practice of the destination country may prevent the data importer from complying with them; exporters must instead assess each transfer and, where necessary, adopt supplementary measures.

The list of sub-processors referenced below is the same as the one published at [hook0.com/gdpr-subprocessors](https://www.hook0.com/gdpr-subprocessors); this document expands, for the sub-processors located outside the EEA, on the transfer analysis summarised there.

## Methodology

For each transfer, Hook0 follows the EDPB's six-step approach:

1. **Map the transfer** — identify the sub-processor, the data categories, and the purpose of the transfer.
2. **Identify the transfer tool relied upon** — an adequacy decision under Article 45 GDPR (including the EU-U.S. Data Privacy Framework, where the importer is certified), and/or Standard Contractual Clauses under Article 46(2)(c), using the module applicable to the relationship between Hook0 and the sub-processor.
3. **Assess the effectiveness of that tool in practice** — whether the law or surveillance practices of the destination country could prevent the importer from honouring its contractual commitments, having regard to the European Essential Guarantees for government access to data.
4. **Identify and adopt supplementary measures** where step 3 reveals a gap — technical (encryption, pseudonymisation, minimisation), contractual, and organisational measures.
5. **Take the necessary formal steps** — execute the applicable Standard Contractual Clauses module, document the assessment, and monitor the sub-processor's own transparency reporting where available.
6. **Re-evaluate periodically** — at least annually, and immediately upon any material change (loss of Data Privacy Framework certification, new sub-processor, change in the destination country's legal framework, or a relevant CJEU/EDPB decision).

**Legal transfer mechanisms considered below**:

- **EU-U.S. Data Privacy Framework (DPF)**: the adequacy decision adopted by the European Commission on 10 July 2023 (Commission Implementing Decision (EU) 2023/1795) under Article 45 GDPR, applicable to US organisations self-certified under the DPF. Where an importer is DPF-certified, transfers to it benefit from an adequacy decision and do not, in principle, require additional safeguards — subject to Hook0 monitoring the certification's continued validity and any legal challenge to the adequacy decision itself.
- **Standard Contractual Clauses (SCC)**: the clauses approved by the European Commission in Implementing Decision (EU) 2021/914 of 4 June 2021, used as the Article 46(2)(c) transfer tool for importers that are not (or no longer) covered by an adequacy decision, or as a contractual fallback alongside DPF certification. The applicable module depends on the parties' roles: **Module 2** (controller-to-processor) governs transfers of data for which Hook0 is the controller (account, billing, and platform data); flow-down obligations equivalent to **Module 3** (processor-to-processor) apply where the transferred data is Customer Content that Hook0 itself processes as a processor on behalf of its customers, consistent with Article 28(4) GDPR.

## Stripe, Inc. — payment and subscription management

| | |
|---|---|
| **Purpose** | Subscription billing, invoicing, and payment processing for Hook0 customer accounts |
| **Data transferred** | Name, email address, billing address, subscription history; payment instrument data is collected and stored directly by Stripe (Hook0 never receives or stores full card numbers) |
| **Destination country** | United States |
| **Role** | Hook0 is the controller of this billing data; Stripe is Hook0's processor |
| **Transfer mechanism** | **Primary**: EU-U.S. Data Privacy Framework — Stripe, Inc. is self-certified under the EU-U.S. DPF, its UK extension, and the Swiss-U.S. DPF. **Fallback**: Standard Contractual Clauses (Module 2, Decision 2021/914) and the UK International Data Transfer Addendum, which apply automatically should the DPF certification lapse or be invalidated |
| **Supplementary measures** | Encryption in transit (TLS) and at rest; strict data minimisation — Hook0's systems only hold tokenised payment references, never full card data; contractual audit and information rights under the Stripe DPA |
| **Residual risk** | **Low.** The transfer is covered by an adequacy decision (DPF) with a contractual SCC fallback, and the personal data actually held by Hook0 in relation to this transfer is limited to ordinary business contact and billing data, not special categories of data |
| **Conclusion** | Transfer mechanism appropriate; no additional supplementary measures required beyond those already in place |

## Cloudflare, Inc. — CDN, DNS, and DDoS protection

| | |
|---|---|
| **Purpose** | Content delivery, DNS resolution, and DDoS protection for hook0.com and app.hook0.com |
| **Data transferred** | IP addresses, HTTP request metadata, and TLS handshake metadata for traffic reaching Hook0's website and application at the network edge |
| **Destination country** | United States |
| **Role** | Hook0 is the controller of website/application access data; for the portion of API traffic that carries Customer Content and transits the same edge network, Cloudflare acts as a sub-processor and Hook0's Article 28(4) flow-down obligations apply |
| **Transfer mechanism** | **Primary**: EU-U.S. Data Privacy Framework — Cloudflare, Inc. is self-certified under the EU-U.S. DPF, its UK extension, and the Swiss-U.S. DPF. **Fallback**: Standard Contractual Clauses incorporated in Cloudflare's Data Processing Addendum, which Cloudflare undertakes to apply automatically if its DPF certification lapses |
| **Supplementary measures** | TLS 1.2+ enforced end-to-end; Cloudflare does not persistently store the body of proxied requests for this use case; Cloudflare holds ISO/IEC 27701:2019 (privacy information management) certification in addition to ISO 27001, SOC 2 Type II, and Global Cross-Border Privacy Rules (CBPR) certification |
| **Residual risk** | **Low.** Webhook payload delivery to customer endpoints itself runs through Hook0's own EU workers (Clever Cloud, and optionally Scaleway) and is not transferred outside the EEA for that purpose; Cloudflare's exposure is limited to network-edge metadata for the public-facing site and API surface |
| **Conclusion** | Transfer mechanism appropriate; no additional supplementary measures required |

## Sentry (Functional Software, Inc.) — application error tracking

| | |
|---|---|
| **Purpose** | Capturing application errors, stack traces, and request metadata to detect and diagnose incidents in the Hook0 platform |
| **Data transferred** | IP addresses, error stack traces, request metadata, and any personal data incidentally captured in error context if not filtered before submission |
| **Destination country** | United States (Sentry also offers an EU data-residency option, hosted in Germany, which Hook0 does not currently use) |
| **Role** | Hook0 is the controller of this operational/technical data; Sentry is Hook0's processor |
| **Transfer mechanism** | **Primary basis documented by Hook0**: Standard Contractual Clauses (Module 2, Decision 2021/914), incorporated in Sentry's Data Processing Addendum. Sentry additionally self-certifies under the EU-U.S. DPF, its UK extension, and the Swiss-U.S. DPF; Hook0 treats this as an **additional safeguard** rather than the primary basis for this assessment, pending confirmation against Sentry's current DPA at the next periodic review (see Points of attention) |
| **Supplementary measures** | TLS in transit; scrubbing rules configured to strip authentication headers, cookies, and known-sensitive fields from error events before they leave the application; Customer Content (webhook payloads) is not intentionally sent to Sentry; access to the Sentry project is restricted to engineering personnel |
| **Residual risk** | **Low to medium.** Error-tracking tooling is a recognised source of incidental over-collection if scrubbing rules are incomplete or drift over time; this is the primary residual risk factor for this sub-processor, rather than the transfer mechanism itself |
| **Conclusion** | Transfer mechanism appropriate. Recommended action: (a) periodically audit Sentry scrubbing rules against actual captured events to confirm no Customer Content or unexpected personal data reaches Sentry, and (b) confirm Sentry's current DPF self-certification status and, if current, update Hook0's public sub-processor disclosure to credit it as an additional adequacy-based safeguard alongside SCC |

## Postmark (ActiveCampaign, LLC) — transactional email

| | |
|---|---|
| **Purpose** | Delivery of automated transactional emails (account verification, notifications), used as a fallback alongside Hook0's primary EU-based transactional email provider |
| **Data transferred** | Name, email address, and the content of transactional messages sent through this channel |
| **Destination country** | United States |
| **Role** | Hook0 is the controller of this data; Postmark (operated by ActiveCampaign, LLC) is Hook0's processor |
| **Transfer mechanism** | Standard Contractual Clauses (Module 2 — controller to processor — under Decision 2021/914) and the UK International Data Transfer Addendum where applicable, incorporated in ActiveCampaign's Data Processing Addendum. No EU-U.S. Data Privacy Framework certification was identified for ActiveCampaign/Postmark at the time of this assessment |
| **Supplementary measures** | TLS in transit; strict purpose limitation to transactional (non-marketing) messages; retention limited to what is required for delivery and support purposes; use limited to a fallback role behind an EU-based primary provider |
| **Residual risk** | **Low to medium.** Of the four sub-processors assessed here, this is the one relying solely on SCC without a DPF adequacy layer as a fallback, which increases (without eliminating) reliance on the contractual and organisational safeguards described above. As with any US-established provider, the data may in principle be subject to compelled disclosure under the US CLOUD Act; Hook0 mitigates this through data minimisation and by limiting the volume and sensitivity of data routed through this channel |
| **Conclusion** | Transfer mechanism appropriate given the limited and non-sensitive nature of the data involved (identity data and transactional message content, no special categories of data). No additional supplementary measures identified as necessary at this time; this sub-processor is prioritised for re-assessment at the next annual review given the absence of a DPF fallback |

## Overall conclusion

Based on the assessment above, Hook0 concludes that transfers to Stripe, Cloudflare, Sentry, and Postmark are each supported by an appropriate Article 46 transfer mechanism (Standard Contractual Clauses, reinforced by Data Privacy Framework adequacy for Stripe and Cloudflare), together with technical measures (encryption in transit and at rest, data minimisation, access controls) and organisational measures (data processing agreements, purpose limitation) that address the residual risk identified for each transfer. No transfer assessed in this document is currently suspended or requires additional supplementary measures beyond those already described, subject to the recommended actions noted for Sentry and Postmark.

## Points of attention

- Sentry's current EU-U.S. DPF self-certification status should be re-verified against its published Data Processing Addendum before Hook0's public sub-processor disclosure is updated to reference it.
- Postmark/ActiveCampaign should be monitored for any future DPF self-certification, which would strengthen its transfer basis.
- This assessment does not cover Google LLC (Google Ads server-side conversion measurement) or other sub-processors already documented elsewhere; those are addressed in the Privacy Policy and the GDPR & Subprocessors page.

## Review

This Transfer Impact Assessment is re-evaluated at least once a year, and immediately upon: the addition of a new non-EEA sub-processor, the loss or suspension of a relied-upon Data Privacy Framework certification, a material change in the destination country's surveillance law or practice, or a relevant decision of the CJEU or the EDPB affecting the transfer tools used here.

Questions about this assessment can be sent to the Data Protection Officer at [dpo@hook0.com](mailto:dpo@hook0.com).
