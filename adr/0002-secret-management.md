# ADR 0002: Secret Management Strategy

## Status
Accepted

## Context
Hook0 requires a secure method to store and manage sensitive authentication credentials, supporting:
- Environment variable references
- Direct encrypted storage
- Minimal external dependencies
- Secure encryption at rest

## Decision
Implement a dual-mode secret management system using:
1. Environment Variable References (`env://`)
   - Allows external secret management
   - Useful for shared or dynamically rotated secrets
   - No database storage of actual secret

2. Encrypted Database Storage
   - AES-256-GCM encryption
   - Database-specific encryption keys
   - Nonce-based encryption for each secret
   - Automatic encryption/decryption

## Consequences

### Positive
- Flexible secret storage options
- High security with AES-256-GCM
- No external secret management dependencies
- Support for various deployment scenarios

### Negative
- Additional complexity in secret resolution
- Performance overhead for encryption/decryption
- Requires careful key management

## Alternatives Considered
- HashiCorp Vault (rejected due to external dependency)
- AWS KMS (rejected due to cloud lock-in)
- Plaintext storage (rejected due to security risks)

## Additional Considerations
- Encryption keys generated using cryptographically secure random number generators
- Support for key rotation
- Environment-based key configuration
- Complete audit logging of secret access