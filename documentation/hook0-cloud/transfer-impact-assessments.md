# Transfer Impact Assessments (TIA)

**Version:** 1.0
**Effective Date:** January 2025
**Owner:** Data Protection Officer (DPO)
**Contact:** dpo@hook0.com

## Overview

This document contains Transfer Impact Assessments (TIAs) for data transfers to third countries (primarily USA) as required by GDPR Article 46 and the Schrems II ruling (CJEU C-311/18).

Hook0's infrastructure is primarily hosted in the European Union (France). However, certain auxiliary services require data transfers to the United States. For each transfer, we have conducted an assessment to ensure appropriate safeguards are in place.

---

## TIA - Cloudflare

### 1. Nature of the Transfer

| Aspect | Details |
|--------|---------|
| **Data Transferred** | DNS queries, HTTP headers (IP addresses, User-Agent), request metadata |
| **Frequency** | Continuous (all web traffic) |
| **Purpose** | DNS resolution, DDoS protection, CDN |
| **Data Volume** | High (all web requests) |

### 2. Third Country Legislation Assessment

| Factor | Assessment |
|--------|------------|
| **Country** | United States |
| **Relevant Laws** | FISA Section 702, Executive Order 12333, CLOUD Act |
| **Probability of Access** | Low |
| **Risk Level** | Medium |

**Analysis:** Cloudflare processes primarily technical/operational data (IP addresses, request metadata) rather than content data. Under FISA 702, Cloudflare as an Electronic Communications Service Provider could be subject to surveillance requests. However:
- Data is transient (not stored long-term)
- Cloudflare has challenged government requests and publishes transparency reports
- The EU-US Data Privacy Framework provides additional protections

### 3. Safeguards Implemented

| Safeguard | Status |
|-----------|--------|
| **EU Standard Contractual Clauses (2021)** | Yes |
| **EU-US Data Privacy Framework Certification** | Yes |
| **Encryption in Transit** | Yes (TLS 1.3) |
| **Encryption at Rest** | Yes |
| **Data Minimization** | Applied (logs retained less than 4 hours) |
| **Contractual DPA** | [View DPA](https://www.hook0.com/legal/Cloudflare_Customer_DPA_v6.3_June_20__2025.pdf) |

### 4. Conclusion

**Transfer Justified:** Yes

The transfer to Cloudflare is justified because:
1. Standard Contractual Clauses are in place
2. Cloudflare is certified under the EU-US Data Privacy Framework
3. Data processed is primarily technical/operational (not sensitive content)
4. Strong encryption and data minimization practices are applied
5. The benefits (DDoS protection, global performance) outweigh the minimal privacy risks

---

## TIA - Stripe

### 1. Nature of the Transfer

| Aspect | Details |
|--------|---------|
| **Data Transferred** | Customer name, email, billing address, payment card tokens |
| **Frequency** | On subscription creation/update/cancellation |
| **Purpose** | Payment processing and subscription management |
| **Data Volume** | Low (only paying customers) |

### 2. Third Country Legislation Assessment

| Factor | Assessment |
|--------|------------|
| **Country** | United States |
| **Relevant Laws** | FISA Section 702, Executive Order 12333, CLOUD Act |
| **Probability of Access** | Very Low |
| **Risk Level** | Low |

**Analysis:** Stripe processes financial transaction data, which is subject to strong regulatory protections (PCI-DSS, financial regulations). Government access to payment data typically requires specific court orders. Stripe:
- Is PCI-DSS Level 1 certified
- Does not store raw card numbers (only tokens)
- Has robust legal team challenging inappropriate requests
- Is certified under the EU-US Data Privacy Framework

### 3. Safeguards Implemented

| Safeguard | Status |
|-----------|--------|
| **EU Standard Contractual Clauses (2021)** | Yes |
| **EU-US Data Privacy Framework Certification** | Yes |
| **Encryption in Transit** | Yes (TLS 1.2+) |
| **Encryption at Rest** | Yes (AES-256) |
| **PCI-DSS Compliance** | Level 1 |
| **Tokenization** | Card data tokenized |
| **Contractual DPA** | [View DPA](https://www.hook0.com/legal/Stripe-DPA_2025-Nov-18_.pdf) |

### 4. Conclusion

**Transfer Justified:** Yes

The transfer to Stripe is justified because:
1. Payment processing requires specialized infrastructure not available in EU
2. Stripe is certified under EU-US Data Privacy Framework
3. Strong contractual protections (SCC) are in place
4. PCI-DSS Level 1 certification ensures highest security standards
5. Card data is tokenized, minimizing actual sensitive data transfer
6. Financial data is subject to additional legal protections

---

## TIA - Postmark

:::tip EU Alternative Under Evaluation
We are actively evaluating **[Brevo](https://www.brevo.com/)** (formerly Sendinblue), an EU-based transactional email provider headquartered in France, as a potential replacement for Postmark. This would eliminate the need for US data transfers for transactional emails.
:::

### 1. Nature of the Transfer

| Aspect | Details |
|--------|---------|
| **Data Transferred** | Email addresses, user names, email content (transactional) |
| **Frequency** | On account events (registration, password reset, notifications) |
| **Purpose** | Transactional email delivery |
| **Data Volume** | Low (occasional transactional emails only) |

### 2. Third Country Legislation Assessment

| Factor | Assessment |
|--------|------------|
| **Country** | United States |
| **Relevant Laws** | FISA Section 702, Electronic Communications Privacy Act |
| **Probability of Access** | Low |
| **Risk Level** | Medium |

**Analysis:** Postmark processes email communications which could theoretically be subject to surveillance. However:
- Emails are transactional only (not marketing or personal correspondence)
- Content is ephemeral (delivery confirmations, password resets)
- Postmark is owned by ActiveCampaign, which has EU data protection commitments

### 3. Safeguards Implemented

| Safeguard | Status |
|-----------|--------|
| **EU Standard Contractual Clauses (2021)** | Yes |
| **Encryption in Transit** | Yes (TLS) |
| **Encryption at Rest** | Yes |
| **Data Retention** | 45 days (logs), no long-term content storage |
| **GDPR Compliance Page** | [View](https://postmarkapp.com/eu-privacy) |

### 4. Supplementary Measures

Given the nature of email transmission:
1. We minimize personal data in email content where possible
2. We use EU-based alternative (Brevo) for marketing communications
3. Transactional emails contain minimal personal information

### 5. Conclusion

**Transfer Justified:** Yes, with supplementary measures

The transfer to Postmark is justified because:
1. Standard Contractual Clauses are in place
2. Data transferred is minimal and transactional
3. No sensitive personal data is included in email content
4. Short retention periods minimize exposure

---

## TIA - Google Workspace (Gmail)

:::tip EU Alternative Under Evaluation
We are actively evaluating **[Proton Mail](https://proton.me/mail)** (or similar EU-based providers), a Swiss/EU privacy-focused email provider, as a potential replacement for Google Workspace. Switzerland benefits from an EU adequacy decision, ensuring GDPR-equivalent data protection.
:::

### 1. Nature of the Transfer

| Aspect | Details |
|--------|---------|
| **Data Transferred** | Email addresses, names, email content (support correspondence) |
| **Frequency** | On customer support requests |
| **Purpose** | Customer support mailbox |
| **Data Volume** | Very Low (support emails only) |

### 2. Third Country Legislation Assessment

| Factor | Assessment |
|--------|------------|
| **Country** | United States |
| **Relevant Laws** | FISA Section 702, CLOUD Act, Stored Communications Act |
| **Probability of Access** | Low |
| **Risk Level** | Medium |

**Analysis:** Google as a major Electronic Communications Service Provider is subject to US surveillance laws. However:
- Google has challenged government requests extensively
- Publishes detailed transparency reports
- Is certified under EU-US Data Privacy Framework
- Provides strong encryption and security measures

### 3. Safeguards Implemented

| Safeguard | Status |
|-----------|--------|
| **EU Standard Contractual Clauses (2021)** | Yes |
| **EU-US Data Privacy Framework Certification** | Yes |
| **Encryption in Transit** | Yes (TLS) |
| **Encryption at Rest** | Yes (AES-256) |
| **2-Factor Authentication** | Enabled |
| **Contractual DPA** | [View](https://cloud.google.com/terms/data-processing-addendum) |

### 4. Supplementary Measures

1. Support requests are directed to dedicated support@hook0.com
2. Staff trained to minimize personal data in correspondence
3. Sensitive technical issues can be handled through secure channels upon request

### 5. Conclusion

**Transfer Justified:** Yes

The transfer to Google Workspace is justified because:
1. Strong contractual protections (SCC + DPF) are in place
2. Data volume is minimal (support correspondence only)
3. Google provides industry-leading security measures
4. Transparency reports demonstrate commitment to user privacy

---

## TIA - Sentry (Error Tracking)

### 1. Nature of the Transfer

| Aspect | Details |
|--------|---------|
| **Data Transferred** | Error stack traces, user identifiers (anonymized), request metadata, IP addresses |
| **Frequency** | On application errors |
| **Purpose** | Error tracking and performance monitoring |
| **Data Volume** | Low to Medium (depends on error frequency) |

### 2. Third Country Legislation Assessment

| Factor | Assessment |
|--------|------------|
| **Country** | United States |
| **Relevant Laws** | FISA Section 702, CLOUD Act |
| **Probability of Access** | Low |
| **Risk Level** | Low-Medium |

**Analysis:** Sentry processes technical error data which is primarily operational in nature. The data contains:
- Stack traces (code paths, not user content)
- Request metadata (URLs, headers)
- User identifiers (can be anonymized/pseudonymized)
- IP addresses (can be scrubbed)

Sentry:
- Is certified under the EU-US Data Privacy Framework
- Provides data scrubbing options for PII
- Offers configurable data retention
- Publishes transparency reports

### 3. Safeguards Implemented

| Safeguard | Status |
|-----------|--------|
| **EU Standard Contractual Clauses (2021)** | Yes |
| **EU-US Data Privacy Framework Certification** | Yes |
| **Encryption in Transit** | Yes (TLS 1.2+) |
| **Encryption at Rest** | Yes |
| **PII Scrubbing** | Enabled (IP addresses, user data) |
| **Data Retention** | 90 days |
| **Contractual DPA** | [View DPA](https://www.hook0.com/legal/sentry-dpa.pdf) |

### 4. Supplementary Measures

1. PII scrubbing enabled to remove sensitive data before transmission
2. IP address anonymization configured
3. User IDs are internal identifiers, not personal data
4. Error data does not contain customer webhook payloads

### 5. Conclusion

**Transfer Justified:** Yes

The transfer to Sentry is justified because:
1. Standard Contractual Clauses are in place
2. Sentry is certified under EU-US Data Privacy Framework
3. Data transferred is primarily technical/operational (stack traces, not user content)
4. PII scrubbing minimizes personal data exposure
5. The benefits (rapid error detection, service reliability) outweigh the minimal privacy risks

---

## Summary Table

| Subprocessor | Country | Transfer Mechanism | Risk Level | Transfer Justified |
|--------------|---------|-------------------|------------|-------------------|
| Cloudflare | USA | SCC + DPF | Medium | Yes |
| Stripe | USA | SCC + DPF | Low | Yes |
| Postmark | USA | SCC | Medium | Yes |
| Gmail/Google | USA | SCC + DPF | Medium | Yes |
| Sentry | USA | SCC + DPF | Low-Medium | Yes |

---

## Review Schedule

These Transfer Impact Assessments will be reviewed:
- Annually (at minimum)
- When subprocessor changes their data processing practices
- When relevant legislation changes (e.g., new CJEU rulings)
- When data transfer mechanisms are invalidated

---

## Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | January 2025 | DPO | Initial version |
