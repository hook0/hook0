---
title: "Information retention policy"
slug: "information-retention-policy"
excerpt: ""
hidden: false
metadata: 
  image: []
  robots: "index"
createdAt: "Sat Jul 22 2023 13:10:10 GMT+0000 (Coordinated Universal Time)"
updatedAt: "Sat Jul 22 2023 13:10:10 GMT+0000 (Coordinated Universal Time)"
---
## Summary

The information retention policy defines the retention periods for all information in our organization.

The policy is applicable to all internal and external personnel.

## Principles

Refer to the table below for relevant retention periods and disposal instructions for all data types:

### Applicant data

- Retention period: 1 year after application, only if applicant agrees to be kept on file
- Location: Digitally on file server
- Disposal instructions: Delete

### Personnel records (review forms, ...)

- Retention period: 1 year after termination
- Location: Digitally on file server
- Disposal instructions: Delete

### Customer records

- Retention period: 7 years after last transaction
- Location: Digitally in CRM system
- Disposal instructions: Delete

### Email

- Retention period: unlimited, unless it contains any of the data types above
- Location: Digitally on mail server
- Disposal instructions: Delete

### Source code

- Retention period: unlimited
- Location: Digitally on file server(s)
- Disposal instructions: Delete

### Financial records (invoices, tax, records)

- Retention period: 7 years
- Location: Digitally on file server(s) and payment system
- Disposal instructions: Delete

## Service-specific retention periods

### Account data (username, email, encrypted password, API keys)

- Retention period: Duration of the service contract + 30 days after account deletion
- Location: Digitally in production database
- Disposal instructions: Delete

### Billing and invoicing records

- Retention period: 10 years (French tax law, art. L102 B Livre des procédures fiscales)
- Location: Digitally on payment system and file server(s)
- Disposal instructions: Delete

### Webhook event data

- Retention period: Per customer plan — Developer: 7 days, Startup: 14 days, Pro: 30 days, Enterprise: custom
- Location: Digitally in production database
- Disposal instructions: Automatic purge after retention window

### Website analytics (Matomo)

- Retention period: 25 months (CNIL recommendation)
- Location: Matomo instance
- Disposal instructions: Automatic anonymization and deletion

### Support communications

- Retention period: 3 years from the last exchange (statutory limitation period for contractual claims)
- Location: Digitally on support platform and mail server
- Disposal instructions: Delete

### Consent records

- Retention period: 5 years (ability to demonstrate GDPR compliance, Art. 7(1))
- Location: Digitally in production database
- Disposal instructions: Delete

### Server logs

- Retention period: 30 days minimum
- Location: Digitally on log management system
- Disposal instructions: Automatic rotation and deletion
