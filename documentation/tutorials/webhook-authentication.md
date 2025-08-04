# Implementing Webhook Authentication

This tutorial covers various webhook authentication methods, from basic signature verification to advanced security patterns. You'll learn how to secure your webhook endpoints and verify webhook authenticity.

## Prerequisites

- Completed [Getting Started](./getting-started.md) tutorial
- Understanding of cryptographic concepts (HMAC, hashing)
- Basic knowledge of HTTP security

## Authentication Methods Overview

### 1. HMAC Signature Verification (Recommended)
Hook0's default method using HMAC-SHA256 signatures.

### 2. Custom Headers
Using API keys or tokens in HTTP headers.

### 3. IP Allowlisting
Restricting webhook sources by IP address.

### 4. mTLS (Mutual TLS)
Certificate-based authentication for high-security environments.

## Step 1: Understanding Hook0 Signatures

Hook0 signs every webhook request with HMAC-SHA256:

```
Hook0-Signature: sha256=a1b2c3d4e5f6...
```

The signature is computed as:
```
HMAC-SHA256(subscription_secret, request_body)
```

### Signature Components
- **Algorithm**: SHA256
- **Key**: Subscription secret (UUID)
- **Data**: Raw request body (JSON string)
- **Output Format**: `sha256={hex_encoded_hash}`

## Step 2: Basic Signature Verification

### Node.js Implementation

```javascript
const crypto = require('crypto');
const express = require('express');
const app = express();

// Middleware to capture raw body
app.use('/webhooks', express.raw({ type: 'application/json' }));

function verifyHook0Signature(body, signature, secret) {
  // Compute expected signature
  const expectedSignature = crypto
    .createHmac('sha256', secret)
    .update(body)
    .digest('hex');
  
  // Compare signatures using timing-safe comparison
  const receivedSignature = signature.replace('sha256=', '');
  
  return crypto.timingSafeEqual(
    Buffer.from(expectedSignature, 'hex'),
    Buffer.from(receivedSignature, 'hex')
  );
}

app.post('/webhooks/secure', (req, res) => {
  const signature = req.headers['hook0-signature'];
  const secret = process.env.WEBHOOK_SECRET;
  
  if (!signature) {
    return res.status(401).json({ error: 'Missing signature' });
  }
  
  if (!verifyHook0Signature(req.body, signature, secret)) {
    return res.status(401).json({ error: 'Invalid signature' });
  }
  
  // Parse JSON after verification
  const payload = JSON.parse(req.body);
  console.log('Verified webhook:', payload.event_type);
  
  res.json({ status: 'verified' });
});
```

### Python Implementation

```python
import hmac
import hashlib
import json
from flask import Flask, request, jsonify

app = Flask(__name__)

def verify_hook0_signature(body, signature, secret):
    """Verify Hook0 webhook signature"""
    expected_signature = hmac.new(
        secret.encode('utf-8'),
        body,
        hashlib.sha256
    ).hexdigest()
    
    received_signature = signature.replace('sha256=', '')
    
    return hmac.compare_digest(expected_signature, received_signature)

@app.route('/webhooks/secure', methods=['POST'])
def handle_webhook():
    signature = request.headers.get('Hook0-Signature')
    secret = os.environ.get('WEBHOOK_SECRET')
    
    if not signature:
        return jsonify({'error': 'Missing signature'}), 401
    
    if not verify_hook0_signature(request.data, signature, secret):
        return jsonify({'error': 'Invalid signature'}), 401
    
    payload = request.get_json()
    print(f"Verified webhook: {payload['event_type']}")
    
    return jsonify({'status': 'verified'})
```

### Go Implementation

```go
package main

import (
    "crypto/hmac"
    "crypto/sha256"
    "encoding/hex"
    "encoding/json"
    "fmt"
    "io/ioutil"
    "net/http"
    "os"
    "strings"
)

func verifyHook0Signature(body []byte, signature, secret string) bool {
    mac := hmac.New(sha256.New, []byte(secret))
    mac.Write(body)
    expectedSignature := hex.EncodeToString(mac.Sum(nil))
    
    receivedSignature := strings.Replace(signature, "sha256=", "", 1)
    
    return hmac.Equal([]byte(expectedSignature), []byte(receivedSignature))
}

func webhookHandler(w http.ResponseWriter, r *http.Request) {
    signature := r.Header.Get("Hook0-Signature")
    secret := os.Getenv("WEBHOOK_SECRET")
    
    if signature == "" {
        http.Error(w, "Missing signature", http.StatusUnauthorized)
        return
    }
    
    body, err := ioutil.ReadAll(r.Body)
    if err != nil {
        http.Error(w, "Failed to read body", http.StatusBadRequest)
        return
    }
    
    if !verifyHook0Signature(body, signature, secret) {
        http.Error(w, "Invalid signature", http.StatusUnauthorized)
        return
    }
    
    var payload map[string]interface{}
    if err := json.Unmarshal(body, &payload); err != nil {
        http.Error(w, "Invalid JSON", http.StatusBadRequest)
        return
    }
    
    fmt.Printf("Verified webhook: %s\n", payload["event_type"])
    
    w.Header().Set("Content-Type", "application/json")
    json.NewEncoder(w).Encode(map[string]string{"status": "verified"})
}
```

## Step 3: Advanced Signature Verification

### Multi-Secret Support (Secret Rotation)

```javascript
function verifyWithMultipleSecrets(body, signature, secrets) {
  for (const secret of secrets) {
    if (verifyHook0Signature(body, signature, secret)) {
      return { valid: true, secretUsed: secret };
    }
  }
  return { valid: false, secretUsed: null };
}

app.post('/webhooks/multi-secret', (req, res) => {
  const signature = req.headers['hook0-signature'];
  const secrets = [
    process.env.WEBHOOK_SECRET_CURRENT,
    process.env.WEBHOOK_SECRET_PREVIOUS  // For rotation support
  ].filter(Boolean);
  
  if (!signature) {
    return res.status(401).json({ error: 'Missing signature' });
  }
  
  const verification = verifyWithMultipleSecrets(req.body, signature, secrets);
  
  if (!verification.valid) {
    return res.status(401).json({ error: 'Invalid signature' });
  }
  
  // Log which secret was used (for monitoring rotation)
  console.log('Verified with secret:', verification.secretUsed.substring(0, 8) + '...');
  
  const payload = JSON.parse(req.body);
  res.json({ status: 'verified' });
});
```

### Timestamp Verification (Replay Attack Protection)

```javascript
function verifyTimestamp(timestamp, toleranceSeconds = 300) {
  const now = Math.floor(Date.now() / 1000);
  const webhookTime = Math.floor(new Date(timestamp).getTime() / 1000);
  
  return Math.abs(now - webhookTime) <= toleranceSeconds;
}

app.post('/webhooks/timestamp-verified', (req, res) => {
  const signature = req.headers['hook0-signature'];
  const secret = process.env.WEBHOOK_SECRET;
  
  // Verify signature first
  if (!verifyHook0Signature(req.body, signature, secret)) {
    return res.status(401).json({ error: 'Invalid signature' });
  }
  
  const payload = JSON.parse(req.body);
  
  // Verify timestamp to prevent replay attacks
  if (!verifyTimestamp(payload.timestamp)) {
    return res.status(401).json({ error: 'Request too old or too far in future' });
  }
  
  console.log('Verified webhook with timestamp:', payload.timestamp);
  res.json({ status: 'verified' });
});
```

## Step 4: Custom Header Authentication

Add additional authentication headers to your subscriptions:

### Create Subscription with Custom Headers

```bash
curl -X POST "https://api.hook0.com/api/v1/applications/{app-id}/subscriptions" \
  -H "Authorization: Bearer biscuit:YOUR_TOKEN_HERE" \
  -H "Content-Type: application/json" \
  -d '{
    "event_types": ["user.created"],
    "description": "Webhook with custom authentication",
    "target": {
      "type": "http",
      "method": "POST",
      "url": "https://your-api.com/webhooks/authenticated",
      "headers": {
        "Content-Type": "application/json",
        "X-API-Key": "your-api-key-here",
        "X-Webhook-Source": "hook0",
        "Authorization": "Bearer your-bearer-token"
      }
    }
  }'
```

### Verify Custom Headers

```javascript
app.post('/webhooks/authenticated', (req, res) => {
  const signature = req.headers['hook0-signature'];
  const apiKey = req.headers['x-api-key'];
  const webhookSource = req.headers['x-webhook-source'];
  const authorization = req.headers['authorization'];
  
  // Verify Hook0 signature
  if (!verifyHook0Signature(req.body, signature, process.env.WEBHOOK_SECRET)) {
    return res.status(401).json({ error: 'Invalid signature' });
  }
  
  // Verify custom API key
  if (apiKey !== process.env.EXPECTED_API_KEY) {
    return res.status(401).json({ error: 'Invalid API key' });
  }
  
  // Verify webhook source
  if (webhookSource !== 'hook0') {
    return res.status(401).json({ error: 'Invalid webhook source' });
  }
  
  // Verify bearer token
  const expectedToken = `Bearer ${process.env.EXPECTED_BEARER_TOKEN}`;
  if (authorization !== expectedToken) {
    return res.status(401).json({ error: 'Invalid authorization' });
  }
  
  const payload = JSON.parse(req.body);
  console.log('Multi-factor authenticated webhook:', payload.event_type);
  
  res.json({ status: 'authenticated' });
});
```

## Step 5: IP Allowlisting

### Configure Firewall Rules

If using Hook0 Cloud, allowlist these IP ranges:
```
# Hook0 Cloud IP ranges (example - check current documentation)
203.0.113.0/24
198.51.100.0/24
```

### Application-Level IP Filtering

```javascript
const allowedIPs = [
  '203.0.113.10',
  '203.0.113.11',
  '198.51.100.20'
];

function isIPAllowed(clientIP) {
  return allowedIPs.includes(clientIP);
}

app.post('/webhooks/ip-filtered', (req, res) => {
  const clientIP = req.ip || req.connection.remoteAddress;
  
  if (!isIPAllowed(clientIP)) {
    console.log('Blocked request from:', clientIP);
    return res.status(403).json({ error: 'IP not allowed' });
  }
  
  // Continue with signature verification
  const signature = req.headers['hook0-signature'];
  if (!verifyHook0Signature(req.body, signature, process.env.WEBHOOK_SECRET)) {
    return res.status(401).json({ error: 'Invalid signature' });
  }
  
  const payload = JSON.parse(req.body);
  res.json({ status: 'verified' });
});
```

## Step 6: Mutual TLS (mTLS)

For high-security environments, configure mutual TLS:

### Generate Client Certificates

```bash
# Generate CA private key
openssl genrsa -out ca-key.pem 4096

# Generate CA certificate
openssl req -new -x509 -days 365 -key ca-key.pem -sha256 -out ca.pem

# Generate client private key
openssl genrsa -out client-key.pem 4096

# Generate client certificate signing request
openssl req -subj '/CN=hook0-client' -new -key client-key.pem -out client.csr

# Sign client certificate
openssl x509 -req -days 365 -in client.csr -CA ca.pem -CAkey ca-key.pem -out client-cert.pem
```

### Configure Express.js for mTLS

```javascript
const https = require('https');
const fs = require('fs');

const options = {
  key: fs.readFileSync('server-key.pem'),
  cert: fs.readFileSync('server-cert.pem'),
  ca: fs.readFileSync('ca.pem'),
  requestCert: true,
  rejectUnauthorized: true
};

app.post('/webhooks/mtls', (req, res) => {
  // Client certificate is automatically verified by Node.js
  const clientCert = req.connection.getPeerCertificate();
  
  console.log('Client certificate CN:', clientCert.subject.CN);
  
  // Additional signature verification
  const signature = req.headers['hook0-signature'];
  if (!verifyHook0Signature(req.body, signature, process.env.WEBHOOK_SECRET)) {
    return res.status(401).json({ error: 'Invalid signature' });
  }
  
  const payload = JSON.parse(req.body);
  res.json({ status: 'mTLS verified' });
});

https.createServer(options, app).listen(8443, () => {
  console.log('HTTPS Server with mTLS running on port 8443');
});
```

## Step 7: Authentication Middleware

Create reusable authentication middleware:

```javascript
// auth-middleware.js
const crypto = require('crypto');

class WebhookAuth {
  constructor(options = {}) {
    this.secrets = options.secrets || [];
    this.timestampTolerance = options.timestampTolerance || 300;
    this.requireTimestampValidation = options.requireTimestampValidation || false;
    this.allowedIPs = options.allowedIPs || [];
    this.requiredHeaders = options.requiredHeaders || {};
  }
  
  verifySignature(body, signature, secret) {
    const expectedSignature = crypto
      .createHmac('sha256', secret)
      .update(body)
      .digest('hex');
    
    const receivedSignature = signature.replace('sha256=', '');
    
    return crypto.timingSafeEqual(
      Buffer.from(expectedSignature, 'hex'),
      Buffer.from(receivedSignature, 'hex')
    );
  }
  
  verifyTimestamp(timestamp) {
    if (!this.requireTimestampValidation) return true;
    
    const now = Math.floor(Date.now() / 1000);
    const webhookTime = Math.floor(new Date(timestamp).getTime() / 1000);
    
    return Math.abs(now - webhookTime) <= this.timestampTolerance;
  }
  
  verifyIP(clientIP) {
    if (this.allowedIPs.length === 0) return true;
    return this.allowedIPs.includes(clientIP);
  }
  
  verifyHeaders(headers) {
    for (const [key, expectedValue] of Object.entries(this.requiredHeaders)) {
      if (headers[key.toLowerCase()] !== expectedValue) {
        return false;
      }
    }
    return true;
  }
  
  middleware() {
    return (req, res, next) => {
      // IP verification
      const clientIP = req.ip || req.connection.remoteAddress;
      if (!this.verifyIP(clientIP)) {
        return res.status(403).json({ error: 'IP not allowed' });
      }
      
      // Header verification
      if (!this.verifyHeaders(req.headers)) {
        return res.status(401).json({ error: 'Required headers missing or invalid' });
      }
      
      // Signature verification
      const signature = req.headers['hook0-signature'];
      if (!signature) {
        return res.status(401).json({ error: 'Missing signature' });
      }
      
      let signatureValid = false;
      for (const secret of this.secrets) {
        if (this.verifySignature(req.body, signature, secret)) {
          signatureValid = true;
          break;
        }
      }
      
      if (!signatureValid) {
        return res.status(401).json({ error: 'Invalid signature' });
      }
      
      // Parse payload
      try {
        req.webhook = JSON.parse(req.body);
      } catch (error) {
        return res.status(400).json({ error: 'Invalid JSON payload' });
      }
      
      // Timestamp verification
      if (!this.verifyTimestamp(req.webhook.timestamp)) {
        return res.status(401).json({ error: 'Invalid timestamp' });
      }
      
      next();
    };
  }
}

module.exports = WebhookAuth;
```

### Use the Middleware

```javascript
const WebhookAuth = require('./auth-middleware');

const webhookAuth = new WebhookAuth({
  secrets: [
    process.env.WEBHOOK_SECRET_CURRENT,
    process.env.WEBHOOK_SECRET_PREVIOUS
  ],
  timestampTolerance: 300,
  requireTimestampValidation: true,
  allowedIPs: ['203.0.113.10', '203.0.113.11'],
  requiredHeaders: {
    'x-webhook-source': 'hook0',
    'x-api-key': process.env.EXPECTED_API_KEY
  }
});

app.use('/webhooks', express.raw({ type: 'application/json' }));
app.use('/webhooks', webhookAuth.middleware());

app.post('/webhooks/secure', (req, res) => {
  // req.webhook contains parsed and verified payload
  console.log('Authenticated webhook:', req.webhook.event_type);
  res.json({ status: 'authenticated' });
});
```

## Step 8: Testing Authentication

Create a test script to verify your authentication:

```javascript
// test-auth.js
const crypto = require('crypto');
const fetch = require('node-fetch');

function generateSignature(body, secret) {
  return 'sha256=' + crypto
    .createHmac('sha256', secret)
    .update(body)
    .digest('hex');
}

async function testWebhook(url, payload, secret, headers = {}) {
  const body = JSON.stringify(payload);
  const signature = generateSignature(body, secret);
  
  const response = await fetch(url, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Hook0-Signature': signature,
      ...headers
    },
    body
  });
  
  console.log(`${response.status}: ${await response.text()}`);
}

// Test valid request
testWebhook('http://localhost:3000/webhooks/secure', {
  event_id: 'evt_123',
  event_type: 'user.created',
  payload: { user_id: 'user_123' },
  timestamp: new Date().toISOString()
}, 'your-webhook-secret');

// Test invalid signature
testWebhook('http://localhost:3000/webhooks/secure', {
  event_id: 'evt_123',
  event_type: 'user.created',
  payload: { user_id: 'user_123' },
  timestamp: new Date().toISOString()
}, 'wrong-secret');
```

## Best Practices

### Security
- ✅ Always verify HMAC signatures
- ✅ Use timing-safe comparison for signatures
- ✅ Implement timestamp validation to prevent replay attacks
- ✅ Store secrets securely (environment variables, key management)
- ✅ Use HTTPS for all webhook endpoints
- ✅ Log authentication failures for monitoring
- ❌ Don't log webhook secrets
- ❌ Don't rely solely on IP allowlisting
- ❌ Don't skip signature verification

### Implementation
- ✅ Create reusable authentication middleware
- ✅ Support secret rotation with multiple valid secrets
- ✅ Return appropriate HTTP status codes
- ✅ Validate request format before processing
- ❌ Don't process webhooks with invalid authentication
- ❌ Don't expose internal error details in responses

## What You've Learned

✅ Implemented HMAC-SHA256 signature verification  
✅ Built multi-language webhook authentication  
✅ Created advanced security patterns (timestamp validation, IP filtering)  
✅ Configured custom header authentication  
✅ Set up mutual TLS for high-security environments  
✅ Built reusable authentication middleware  
✅ Tested authentication implementations  

## Next Steps

- [Self-hosting Hook0 with Docker](./self-hosting-docker.md)
- [Securing Webhook Endpoints](../how-to-guides/secure-webhook-endpoints.md)
- [Debugging Failed Webhook Deliveries](../how-to-guides/debug-failed-webhooks.md)

## Troubleshooting

### Signature Verification Fails
1. Check you're using the raw request body (not parsed JSON)
2. Verify the correct subscription secret
3. Ensure consistent character encoding (UTF-8)
4. Check HMAC algorithm (SHA256, not SHA1)

### Timestamp Validation Issues
1. Verify timestamp format (ISO 8601)
2. Check server clock synchronization
3. Adjust tolerance window if needed
4. Handle timezone differences correctly

### IP Allowlisting Problems
1. Check if you're behind a proxy/load balancer
2. Verify actual client IP address
3. Account for IPv4/IPv6 differences
4. Consider using X-Forwarded-For header