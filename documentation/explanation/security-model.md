# Security Model

Hook0 implements defense-in-depth security with multiple layers of protection. This document explains the security architecture, threat model, and security best practices.

## Security Architecture

<!-- ### Authentication & Authorization
Hook0 uses Biscuit tokens for authentication, providing cryptographically secure, fine-grained access control.

#### Biscuit Token Structure
```
Bearer EoQKCAohCiEKIH0eTOWqO...
```

Biscuit tokens contain:
- **Identity**: User or service identity
- **Capabilities**: Fine-grained permissions
- **Constraints**: Time limits, IP restrictions
- **Attenuation**: Ability to restrict token further

#### Permission Model
```
organization:read
organization:write
application:read
application:write
event:send
subscription:manage
``` -->

### Multi-Tenant Isolation

#### Organization Boundaries
- Complete data isolation between organizations
- Users can only access their organization's resources
- Database queries filtered by organization_id
- No cross-organization data leakage

#### Application Scoping
- Events scoped to specific applications
- Subscriptions tied to applications
- Service tokens can be application-specific

### API Security

#### Transport Security
- TLS 1.2+ required for all API communication
- HSTS headers enforce secure connections
- Certificate pinning recommended for clients

#### Request Validation
```rust
// Input validation and sanitization
#[derive(Validate, Deserialize)]
struct EventPayload {
    #[validate(length(min = 1, max = 255))]
    event_type: String,
    
    #[validate(custom = "validate_json_size")]
    payload: Value,
}
```

#### Rate Limiting
- Per-organization quotas
- Per-IP rate limits
- Adaptive rate limiting based on behavior
- Protection against DoS attacks

### Webhook Security

#### Signature Verification
All webhook deliveries include HMAC-SHA256 signatures in the `X-Hook0-Signature` header. Hook0 uses **v1 signatures by default**, which include selected headers for additional security:

```
X-Hook0-Signature: t=1765443663,h=content-type x-custom-header,v1=85da0586ae0b711d...
```

- **t**: Unix timestamp (seconds)
- **h**: Space-separated list of header names included in signature
- **v1**: HMAC-SHA256 signature (hex-encoded)

Signature computation: `HMAC-SHA256(secret, timestamp + "." + header_names + "." + header_values + "." + payload)`

For implementation details and code examples in JavaScript, Python, and Go, see [Implementing Webhook Authentication](../tutorials/webhook-authentication.md).

#### Target URL Validation
- HTTPS required for production webhooks
- Private IP ranges blocked by default
- URL validation and sanitization
- DNS rebinding protection

### Data Protection

#### Encryption at Rest
- Database encryption using PostgreSQL TDE
- Secrets encrypted with application keys
- Backup encryption with separate keys

#### Encryption in Transit
- TLS for all external communication
- Internal service communication encrypted
- Database connections encrypted

#### Data Retention
```sql
-- Automatic cleanup of old events
DELETE FROM events 
WHERE created_at < NOW() - INTERVAL '90 days';

-- Secure deletion with overwriting
VACUUM FULL events;
```

### Secret Management

#### Subscription Secrets
- Cryptographically random UUIDs
- Never logged or exposed in responses
- Rotatable through API
- Scoped to individual subscriptions

#### Service Tokens
- Long-lived tokens for API access
- Restricted permissions
- Audit logging for usage
- Revocable at any time

#### Key Rotation
```rust
// Regular secret rotation
impl SubscriptionSecret {
    pub fn rotate(&mut self) -> Result<Uuid, Error> {
        let new_secret = Uuid::new_v4();
        self.previous_secret = Some(self.current_secret);
        self.current_secret = new_secret;
        Ok(new_secret)
    }
}
```

## Threat Model

### Identified Threats

#### T1: Unauthorized Access
- **Threat**: Attackers gaining access to organization data
- **Mitigation**: Strong authentication, authorization, audit logging

#### T2: Event Injection  
- **Threat**: Malicious events sent to trigger unwanted webhooks
- **Mitigation**: Authentication, input validation, rate limiting

#### T3: Webhook Tampering
- **Threat**: Attackers modifying webhook payloads in transit
- **Mitigation**: HMAC signatures, TLS encryption

#### T4: Denial of Service
- **Threat**: Service disruption through resource exhaustion
- **Mitigation**: Rate limiting, quotas, circuit breakers

#### T5: Data Exfiltration
- **Threat**: Sensitive data extracted from system
- **Mitigation**: Access controls, audit logging, encryption

### Attack Vectors

#### API Endpoints
- Authentication bypass attempts
- Authorization escalation
- Input validation bypasses
- Rate limit circumvention

#### Webhook Targets
- Webhook replay attacks
- Signature bypass attempts
- Target URL manipulation
- Response time analysis

## Security Best Practices

### For Hook0 Operators

#### Infrastructure Security
```yaml
# Kubernetes security context
securityContext:
  runAsNonRoot: true
  runAsUser: 1000
  readOnlyRootFilesystem: true
  allowPrivilegeEscalation: false
```

#### Network Security
- VPC isolation
- Security groups/firewalls
- Private subnets for databases
- Network monitoring

#### Monitoring & Alerting
- Failed authentication attempts
- Unusual API usage patterns
- High error rates
- Resource exhaustion

### For Application Developers

#### Token Management
```rust
// Use environment variables for tokens
let token = env::var("HOOK0_TOKEN")
    .expect("HOOK0_TOKEN environment variable required");

// Do not log tokens
log::info!("Making API request with token: [REDACTED]");
```

#### Webhook Verification

See [Implementing Webhook Authentication](../tutorials/webhook-authentication.md) for signature verification code examples.

#### Error Handling
```python
# Do not expose internal errors
try:
    process_webhook(payload)
except Exception as e:
    logger.error(f"Webhook processing failed: {e}")
    return {"error": "Internal server error"}, 500
```

### For Webhook Recipients

#### Endpoint Security
- Use HTTPS with valid certificates
- Implement signature verification
- Validate payload structure
- Rate limit webhook endpoints

#### Idempotency
```python
# Handle duplicate events
processed_events = set()

def handle_webhook(event_id, payload):
    if event_id in processed_events:
        return {"status": "already_processed"}
    
    result = process_event(payload)
    processed_events.add(event_id)
    return result
```

## Compliance & Standards

### Standards compliance
- **SOC 2 Type II**: Security controls and monitoring
- **ISO 27001**: Information security management
- **General Data Protection Regulation (GDPR)**: Data protection and privacy rights
- **CCPA**: California consumer privacy requirements

### Security Headers
```http
Strict-Transport-Security: max-age=31536000; includeSubDomains
X-Content-Type-Options: nosniff
X-Frame-Options: DENY
X-XSS-Protection: 1; mode=block
Content-Security-Policy: default-src 'self'
```

### Audit Logging
```json
{
  "timestamp": "2024-01-15T10:30:00Z",
  "user_id": "usr_123",
  "organization_id": "org_456", 
  "action": "subscription.created",
  "resource_id": "sub_789",
  "ip_address": "192.168.1.100",
  "user_agent": "Hook0-Client/1.0"
}
```

## Security Testing

### Automated Testing
- Static code analysis (Clippy, security lints)
- Dependency vulnerability scanning
- Container image scanning
- Infrastructure as code scanning

### Penetration Testing
- Regular third-party security assessments
- API security testing
- Webhook delivery security testing
- Infrastructure penetration testing

### Bug Bounty Program
- Responsible disclosure process
- Security researcher rewards
- Public security advisory process

## Incident Response

### Security Incident Handling
1. **Detection**: Automated alerts, monitoring
2. **Assessment**: Impact and scope analysis
3. **Containment**: Isolate affected systems
4. **Eradication**: Remove vulnerabilities
5. **Recovery**: Restore normal operations
6. **Lessons Learned**: Process improvements

### Communication Plan
- Internal stakeholder notification
- Customer communication
- Public disclosure timeline
- Regulatory reporting requirements

## Next Steps

- [Securing Webhook Endpoints](../how-to-guides/secure-webhook-endpoints.md)
- [API Reference](../openapi/intro)
