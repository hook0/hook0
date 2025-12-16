---
title: "Penetration testing policy"
slug: "penetration-testing-policy"
excerpt: ""
hidden: false
metadata: 
  image: []
  robots: "index"
createdAt: "Sat Jul 22 2023 12:56:10 GMT+0000 (Coordinated Universal Time)"
updatedAt: "Sat Jul 22 2023 12:56:10 GMT+0000 (Coordinated Universal Time)"
---
## Summary

The penetration testing policy describes rules and principles on how vulnerability testing is applied in our organization.

## Principles

- Penetration tests should occur once per year, or after a major change to the architecture/infrastructure
- We are open to any customer initiated penetration test, providing
  - It is announced and dates are agreed upon
  - We receive an integral copy of the test report
- In case we initiate the penetration test, the tester should meet the competence requirements as defined on Security tester
- We have published a Responsible disclosure policy on our website

## Scoring

- Vulnerabilities should be scored according to the Common Vulnerability Scoring System (CVSS), version 2 or 3
- Exceptions to the CVSS score (e.g. related to impact calculation) should be agreed upon with the penetration tester involved
- If arbitration is needed, a second opinion can be obtained from another penetration testing company

## Treatment

Vulnerabilities are treated based on the CVSS score, as follows:

- **Severity: None**: CVSS2: - CVSS3: 0.0 Treatment: None
- **Severity:Low**: CVSS2: 0.0-3.9 CVSS3: 0.1-3.9 Treatment: Added to the backlog for the next version
- **Severity:Medium**: CVSS2: 4.0-6.9 CVSS3: 4.0-6.9 Treatment: Following the Incident management process, should be fixed within 1 month
- **Severity:High**: CVSS2: 7.0-10.0 CVSS3: 7.0-8.9 Treatment: Following the Incident management process, should be fixed within 1 week
- **Severity:Critical**: CVSS2: - CVSS3: 9.0-10 Treatment: Following the Incident management process, should be fixed immediately
