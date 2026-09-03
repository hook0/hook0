---
title: Personal Data Breach Notification Procedure
description: How Hook0 Cloud detects, assesses, and notifies personal data breaches under Articles 33 and 34 GDPR, including the 72-hour supervisory authority deadline and processor obligations toward customers.
keywords: [gdpr, data breach, article 33, article 34, cnil, breach notification, data protection, incident response]
---

## Summary

This procedure describes how Hook0 Cloud identifies, assesses, and notifies personal data breaches in accordance with Articles 33 and 34 of Regulation (EU) 2016/679 (GDPR). It sets out the internal escalation chain, the severity assessment methodology, the notification obligations toward the competent supervisory authority, affected individuals, and Hook0's own customers, and the mandatory internal breach register.

This procedure is maintained by Hook0's Data Protection Officer and reviewed at least once a year or after any breach that triggers a supervisory authority notification.

## Scope and definitions

### Personal data breach

Under Article 4(12) GDPR, a personal data breach is "a breach of security leading to the accidental or unlawful destruction, loss, alteration, unauthorised disclosure of, or access to, personal data transmitted, stored or otherwise processed." This covers three distinct types of impact, which can occur individually or in combination:

- **Confidentiality breach**: unauthorised or accidental disclosure of, or access to, personal data.
- **Integrity breach**: unauthorised or accidental alteration of personal data.
- **Availability breach**: accidental or unauthorised loss of access to, or destruction of, personal data (including a breach that is only temporary, such as extended service unavailability).

### Hook0's dual role

Hook0 processes two distinct categories of personal data, and its obligations differ depending on which one is affected:

| Data category | Hook0's role | Examples |
|---|---|---|
| Account, billing, and platform usage data | **Controller** | Customer identity data, billing details, authentication logs, support communications |
| Customer Content (webhook payloads routed through the platform) | **Processor**, on behalf of the customer organization that is the controller | Event payloads, subscription endpoints, delivery metadata |

A single incident can affect both categories simultaneously. When it does, the two tracks below are run in parallel and are never conflated: the controller track determines whether Hook0 itself must notify the CNIL and affected individuals for account/billing data; the processor track determines what Hook0 must tell its affected customers about their Customer Content, leaving the customer's own notification decisions (as controller of that content) to the customer.

## Internal escalation chain

| Step | Who | Action | Target timing |
|---|---|---|---|
| 1. Detection | Any employee, contractor, monitoring system (error tracking, uptime monitoring, security alerts), or external report (bug bounty, customer, researcher) | Report the suspected incident immediately to the Data Protection Officer and the on-call engineering lead | Immediately upon discovery |
| 2. Triage | DPO + engineering lead | Confirm whether personal data is involved, establish a preliminary timeline, preserve logs and evidence, begin containment | Within hours of the report |
| 3. Awareness (T0) | DPO | Formally record the moment Hook0 achieves reasonable certainty that a breach involving personal data has occurred. This starts the 72-hour clock (Article 33(1)) | As soon as reasonable certainty is reached, not upon full confirmation of scope |
| 4. Severity assessment | DPO, with engineering input | Run the severity methodology below and determine the notification obligations | Before the 72-hour deadline |
| 5. Notification | DPO | Notify the CNIL, affected individuals, and/or affected customers as required | Per the deadlines below |
| 6. Documentation | DPO | Complete the internal breach register entry (mandatory in every case, including non-notifiable breaches) | Within the same working week |

Gaps between suspicion and the formal T0 of more than 24 hours should be documented with the specific reason (for example, time needed to distinguish a genuine breach from a false alarm). T0 is the moment of reasonable certainty that personal data was affected, not the moment the full scope of the incident is known; where facts remain incomplete, Hook0 notifies with the information available and supplements it later (Article 33(4)).

## Severity assessment

Hook0 assesses the severity of every confirmed personal data breach using the ENISA severity assessment methodology, which the European Union Agency for Cybersecurity (ENISA) developed at the request of European data protection authorities and which is commonly used to support the risk judgment required by Articles 33(1) and 34(1) GDPR.

**Formula**: `SE = (DPC × EI) + CB`

| Component | Meaning | Range |
|---|---|---|
| DPC | Data Processing Context — sensitivity of the data category involved (simple contact data through special categories of data under Article 9) | 1 to 4 |
| EI | Ease of Identification — how readily the exposed data can be linked to a specific individual | 0.25 to 1.00 |
| CB | Circumstances of the Breach — additive score for confidentiality, integrity, and/or availability loss, plus malicious intent | 0 to 2 (additive) |

| SE score | Severity | Consequence |
|---|---|---|
| Below 2 | Low | Internal register only (Article 33(5)); no external notification is required |
| 2 to below 3 | Medium | Notify the CNIL (Article 33) |
| 3 to below 4 | High | Notify the CNIL and affected individuals (Articles 33 and 34) |
| 4 or above | Very high | Notify the CNIL and affected individuals; consider public communication |

Where encryption is involved, Hook0 applies the following logic before concluding that the Article 34(3)(a) exemption is available: the encryption algorithm must be current (state of the art), the key must not be compromised, and the key must be stored separately from the encrypted data. If any of these conditions is not met, the data is treated as if unencrypted for the purposes of the assessment. Encryption never removes the obligation to notify the supervisory authority (Article 33) or to maintain the internal register (Article 33(5)); at most it can remove the obligation to notify individuals (Article 34).

Every severity assessment is documented, including its DPC/EI/CB components and the reasoning behind each score, so that the classification can be justified to the CNIL if requested.

## Notification to the CNIL (Article 33)

**Threshold**: notification is required unless the breach is "unlikely to result in a risk to the rights and freedoms of natural persons" (Article 33(1)). In practice, this corresponds to a Medium severity or above under the methodology described above.

**Deadline**: within 72 hours of Hook0 achieving reasonable certainty that a breach has occurred (T0), without undue delay and, where feasible, as soon as possible. Hook0's competent supervisory authority is the CNIL (Commission Nationale de l'Informatique et des Libertés), as Hook0's main establishment is in France.

**Content** (Article 33(3)): the notification includes, at minimum:

- The nature of the breach, including the categories and approximate number of data subjects and personal data records concerned.
- The contact details of the Data Protection Officer.
- The likely consequences of the breach.
- The measures taken or proposed to address the breach, including measures to mitigate its possible adverse effects.

**Phased notification**: where complete information is not available within 72 hours, Article 33(4) explicitly permits notifying with the information available and providing the remainder in phases, without undue further delay, rather than delaying the initial notification.

**Non-notification**: if the DPO determines that notification is not required (Low severity), this decision and its justification are documented in the internal register in the same level of detail as a notified breach (see below). A breach that is not notified to the CNIL still requires the internal register entry mandated by Article 33(5).

## Notification to affected individuals (Article 34)

**Threshold**: required when the breach is likely to result in a "high risk" to the rights and freedoms of natural persons — in practice, High or Very High severity under the methodology above.

**Deadline**: without undue delay, in clear and plain language.

**Content** (Article 34(2), by reference to Article 33(3)(b)-(d)): a description of the nature of the breach in plain language, the DPO's contact details, the likely consequences, and the measures taken or proposed to address the breach and mitigate its effects, together with concrete steps the individual can take to protect themselves.

**Exemptions** (Article 34(3)): individual notification is not required where any of the following applies, and Hook0 documents which exemption is relied on:

- (a) Hook0 had implemented appropriate technical protection measures (in particular encryption) that render the data unintelligible to any person not authorised to access it, applied to the data affected by the breach.
- (b) Hook0 has taken subsequent measures that ensure the high risk is no longer likely to materialise.
- (c) Individual notification would involve disproportionate effort — in which case Hook0 uses an equally effective public communication instead (for example, a notice on hook0.com and in the application).

The CNIL may independently require Hook0 to notify individuals even where Hook0 concluded this was not necessary; this possibility is factored into every Article 34 assessment.

## Notification to customers, when Hook0 acts as processor (Article 33(2))

Where a breach affects Customer Content that Hook0 processes as a processor on behalf of its customers, Hook0 does not perform the controller-side risk assessment on the customer's behalf. Instead, Article 33(2) GDPR requires Hook0 to notify the affected customer(s) without undue delay after becoming aware of the breach, so that each customer can independently assess and meet its own Article 33 and 34 obligations as controller of that data.

Hook0's Data Processing Addendum commits to notifying the affected customer without undue delay, and in any event within 72 hours of becoming aware of the breach, in writing, describing (to the extent the information is available):

- The nature of the breach, including, where possible, the categories and approximate number of affected data subjects and personal data records.
- The likely consequences of the breach.
- The measures taken or proposed to address the breach, including measures to mitigate its possible adverse effects.

Hook0 provides reasonable assistance to the affected customer on request, including making relevant technical logs and incident details available, to support the customer's own notification to its supervisory authority and to affected individuals under Articles 33 and 34.

**Important distinction**: this processor-to-customer notification is a factual report, not a risk determination. Hook0 does not decide, on the customer's behalf, whether the customer must notify its own supervisory authority or its own end users — that determination and its timing belong to the customer as controller of that data.

### Sub-processor chain

Where a breach originates at one of Hook0's own sub-processors (see the [GDPR & Subprocessors](/gdpr-subprocessors) page for the current list), the notification chain follows the contractual hierarchy: the sub-processor notifies Hook0 without undue delay under its own data processing agreement with Hook0, and Hook0 then notifies its affected customers as described above. Hook0 does not wait for the sub-processor's full incident report before initiating its own notification; it notifies with the information available and supplements it as the investigation progresses.

## Internal breach register (Article 33(5))

Article 33(5) requires Hook0 to document every personal data breach, regardless of whether it was notified to the CNIL, including the facts, its effects, and the remedial action taken. This documentation must allow the CNIL to verify compliance with Article 33 on request.

Every entry in the internal register records:

1. **Facts**: discovery date/time, T0 (awareness) date/time and its triggering event, breach type(s) (confidentiality, integrity, availability), and a factual description of what occurred.
2. **Data affected**: categories of personal data, approximate number of records and individuals, whether special categories of data (Article 9) or vulnerable individuals are involved.
3. **Severity assessment**: the DPC, EI, and CB scores, the resulting SE score, and the severity level, with the reasoning behind each score.
4. **Remedial action**: containment measures taken, mitigation measures taken, and preventive measures planned or implemented.
5. **Notification decisions**: whether the CNIL was notified and when, whether affected individuals were notified and how, whether affected customers were notified and when, and the justification for any decision not to notify.
6. **Sign-off**: prepared by, reviewed by, and approved by, with names, roles, and dates.

Register entries are retained for as long as needed to demonstrate accountability under Article 5(2) GDPR, and are made available to the CNIL upon request.

## Notification template

A minimal Article 33 notification to the CNIL includes: Hook0's identity and DPO contact; the date/time of the breach and of awareness (T0); the nature of the breach; the categories and approximate number of data subjects and records affected; the likely consequences; the containment, mitigation, and preventive measures taken or planned; and, where applicable, whether and when affected individuals were or will be notified. The same structure, adapted to plain language and addressed directly to the affected individual, is used for Article 34 notifications, and a factual variant, addressed to the affected customer and expressly noting that Hook0's own risk assessment does not substitute for the customer's, is used for Article 33(2) processor notifications.

## Contact

Data Protection Officer: [dpo@hook0.com](mailto:dpo@hook0.com)

Anyone who believes they have identified a potential exposure of personal data, whether inside or outside Hook0, is asked to report it immediately to this address.
