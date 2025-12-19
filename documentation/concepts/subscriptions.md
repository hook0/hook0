---
title: Subscriptions
description: Understanding subscriptions in Hook0
---

# Subscriptions

A subscription is a way to receive notifications from Hook0 about specific events. Examples include customer creation and order placement notifications.

## Benefits

- **Real-Time Notifications** - Immediate updates when events occur
- **Decoupled Architecture** - Simplifies maintenance by separating concerns and improves scaling
- **Automated Actions** - Enables email notifications and database updates

## Security Considerations

- **Endpoint Security** - Use IP Whitelisting and signature verification
- **Data Encryption** - All communications use TLS protocols

## Example

### Creating a Subscription

```bash
curl -X POST "$HOOK0_API/subscriptions" \
  -H "Authorization: Bearer $HOOK0_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "'"$APP_ID"'",
    "is_enabled": true,
    "event_types": [
      "user.account.created",
      "order.purchase.completed"
    ],
    "description": "Webhook for Customer ABC Corp",
    "labels": {
      "tenant_id": "customer_123"
    },
    "target": {
      "type": "http",
      "method": "POST",
      "url": "https://customer-abc.com/webhooks/hook0",
      "headers": {
        "Content-Type": "application/json"
      }
    }
  }'
```

**Response:**
```json
{
  "subscription_id": "{SUBSCRIPTION_ID}",
  "is_enabled": true,
  "event_types": ["user.account.created", "order.purchase.completed"],
  "description": "Webhook for Customer ABC Corp",
  "secret": "d48488f1-cddc...",
  "labels": {
    "tenant_id": "customer_123"
  },
  "target": {
    "type": "http",
    "method": "POST",
    "url": "https://customer-abc.com/webhooks/hook0",
    "headers": {"content-type": "application/json"}
  }
}
```

**Important:** Save the `secret` - it's needed to verify webhook signatures.

## Usage Examples

Webhook subscriptions find utility in various domains:

- **E-commerce:**
  - Receive order notifications
  - Manage inventory, initiate shipping, or confirm via email
- **Customer Relationship Management (CRM):**
  - Notifications for new customer creation
  - Populate customer profiles in CRM
- **Payment Processing:**
  - Notifications for payment success or decline
  - Update accounts or notify customers
- **Healthcare:**
  - Notify medical staff when a new appointment is booked
  - Alert when patient medical records are updated
- **Education:**
  - Notify administrators of a new student enrollment
  - Alert students when grades are posted
- **Human Resources:**
  - Notify HR when a new job application is received
  - Trigger onboarding processes when a new employee joins
- **Finance and Banking:**
  - Notify account holders of new transactions
  - Alert fraud management systems of suspicious activities
- **Supply Chain Management:**
  - Notify when stock levels change
  - Alert customers when shipping status changes
- **Marketing:**
  - Notify marketers when a user interacts with a campaign
  - Alert sales when a new lead is generated
- **Hospitality:**
  - Send confirmation messages when a reservation is made
  - Notify staff when guests check-in
- **Retail:**
  - Notify customers or internal teams of price changes
  - Alert when an out-of-stock item is back in stock
- **Real Estate:**
  - Notify agents and clients of new property listings
  - Send reminders for upcoming open house events
- **Energy Management:**
  - Notify consumers when energy consumption exceeds set limits
  - Alert residents or businesses of power outages
- **Government Services:**
  - Notify citizens when it's time to renew licenses
  - Distribute emergency alerts and announcements
- **Manufacturing:**
  - Notify teams of production line statuses and changes
  - Alert when a quality check fails or passes
- **Entertainment Industry:**
  - Notify subscribers of new music, movie, or show releases
  - Alert when tickets for popular events become available
- **Travel and Transportation:**
  - Notify passengers of flight delays, cancellations, or gate changes
  - Send confirmation and details for booked rides
- **Telecommunications:**
  - Alert customers of network outages in their area
  - Notify customers when they change their service plan
- **Legal and Compliance:**
  - Notify clients of updates in their legal cases
  - Alert organizations of new or updated regulations
- **Environmental Monitoring:**
  - Notify users of severe weather warnings
  - Alert residents of high pollution levels
- **Non-Profit Organizations:**
  - Notify when a new donation is received
  - Alert organizers of new registrations for charitable events
- **Security and Surveillance:**
  - Alert security teams of potential breaches or unauthorized access
  - Notify of security equipment failure or maintenance needs
- **Agriculture and Farming:**
  - Alert farmers of changes in crop health
  - Notify of farming equipment status and maintenance needs

## What's Next?

- [Events](events.md)
- [Applications](/explanation/what-is-hook0#applications)
- [Verifying Webhook Signatures](/how-to-guides/secure-webhook-endpoints)
- [IP Whitelisting](/how-to-guides/secure-webhook-endpoints#step-4-implement-ip-allowlisting-and-geolocation)
