# Data Breach Notification Procedure

**Version:** 1.0
**Effective Date:** January 2025
**Owner:** Data Protection Officer (DPO)
**Contact:** dpo@hook0.com

## 1. Purpose

This document establishes Hook0's formal procedure for detecting, responding to, and notifying relevant parties of personal data breaches in compliance with GDPR Articles 33 and 34.

## 2. Scope

This procedure applies to all personal data breaches affecting data processed by Hook0, including:
- Customer account data
- Webhook event data containing personal information
- Employee and contractor data

## 3. Definitions

**Personal Data Breach:** A breach of security leading to the accidental or unlawful destruction, loss, alteration, unauthorized disclosure of, or access to, personal data transmitted, stored, or otherwise processed.

**Supervisory Authority:** For EU data subjects, this is the Commission Nationale de l'Informatique et des Libertés (CNIL) in France.

## 4. Incident Detection and Initial Assessment

### 4.1 Detection Sources
- Automated security monitoring (Sentry, infrastructure alerts)
- Employee reports
- Customer reports
- Third-party notifications
- Security researcher disclosures (security@hook0.com)

### 4.2 Initial Assessment (Within 1 hour)
Upon detection, the incident response team must:

1. **Contain the breach** - Stop ongoing unauthorized access
2. **Preserve evidence** - Document all findings
3. **Assess severity** using the following criteria:
   - Type and sensitivity of data affected
   - Number of individuals affected
   - Potential consequences for individuals
   - Whether data was encrypted

### 4.3 Severity Classification

| Level | Description | Example |
|-------|-------------|---------|
| **Critical** | High-risk data exposed, large scale | Database breach with passwords |
| **High** | Sensitive data exposed, limited scale | Single customer data leak |
| **Medium** | Non-sensitive data exposed | Email addresses leaked |
| **Low** | Potential breach, no confirmed exposure | Suspicious access attempt blocked |

## 5. Notification Requirements

### 5.1 Supervisory Authority Notification (GDPR Art. 33)

**Deadline:** Within 72 hours of becoming aware of the breach

**Required when:** The breach is likely to result in a risk to the rights and freedoms of individuals.

**Notification must include:**
1. Nature of the breach (categories and approximate number of individuals affected)
2. Name and contact details of the DPO
3. Likely consequences of the breach
4. Measures taken or proposed to address the breach

**CNIL Notification Portal:** https://www.cnil.fr/fr/notifier-une-violation-de-donnees-personnelles

### 5.2 Individual Notification (GDPR Art. 34)

**Required when:** The breach is likely to result in a **high risk** to the rights and freedoms of individuals.

**Notification must:**
- Be made without undue delay
- Use clear and plain language
- Describe the nature of the breach
- Include DPO contact details
- Describe likely consequences
- Describe measures taken to mitigate

**NOT required when:**
- Appropriate technical measures were in place (e.g., encryption)
- Subsequent measures ensure high risk is no longer likely
- Individual notification would require disproportionate effort (use public communication instead)

## 6. Notification Templates

### 6.1 Email Template for Individual Notification

```
Subject: Important Security Notice - Action May Be Required

Dear [Customer Name],

We are writing to inform you of a security incident that may have affected your personal data on Hook0.

WHAT HAPPENED:
[Brief description of the breach]

WHAT DATA WAS AFFECTED:
[List of affected data categories]

WHAT WE ARE DOING:
[Description of remediation measures]

WHAT YOU CAN DO:
[Recommended actions for the individual]

We sincerely apologize for any concern this may cause. If you have any questions, please contact our Data Protection Officer at dpo@hook0.com.

Sincerely,
Hook0 Security Team
```

### 6.2 Public Notification Template (Twitter/Status Page)

```
Security Notice: We have identified a security incident affecting [description].
We are actively investigating and will provide updates. Affected users have been
notified directly. Contact: security@hook0.com
```

## 7. Communication Channels

| Audience | Channel | Responsibility |
|----------|---------|----------------|
| CNIL | Online portal | DPO |
| Affected individuals | Email | DPO |
| All customers | Status page | Engineering |
| Public | Twitter (@hook0_) | Marketing/DPO |

## 8. Documentation Requirements

All breaches must be documented in the **Breach Register**, including:
- Date and time of detection
- Date and time of breach (if different)
- Description of the breach
- Categories and volume of data affected
- Number of individuals affected
- Assessment of risk level
- Notifications made (authority, individuals, public)
- Remediation measures taken
- Lessons learned

## 9. Post-Incident Review

Within 14 days of incident closure:
1. Conduct root cause analysis
2. Update security measures if needed
3. Update this procedure if gaps identified
4. Brief relevant team members
5. Update breach register with final details

## 10. Contact Information

| Role | Contact |
|------|---------|
| Data Protection Officer | dpo@hook0.com |
| Security Reports | security@hook0.com |
| General Support | support@hook0.com |

## 11. Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | January 2025 | DPO | Initial version |
