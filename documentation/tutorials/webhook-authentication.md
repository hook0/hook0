# Implementing Webhook Authentication

This tutorial covers various webhook authentication methods, from basic signature verification to advanced security patterns. You'll learn how to secure your webhook endpoints and verify webhook authenticity.

## Prerequisites

- Completed [Getting Started](./getting-started.md) tutorial up to the signature verification.
- Understanding of cryptographic concepts (HMAC, hashing)
- Basic knowledge of HTTP security

### Set Up Environment Variables

```bash
# Set your service token (from dashboard)
export HOOK0_TOKEN="YOUR_TOKEN_HERE"
export HOOK0_API="https://app.hook0.com/api/v1" # Replace by your domain (or http://localhost:8081 locally)

# Set your application ID (shown in dashboard URL or application details)
export APP_ID="YOUR_APPLICATION_ID_HERE"
```

Save these values:
```bash
# Save to .env file for later use
cat > .env <<EOF
HOOK0_TOKEN=$HOOK0_TOKEN
HOOK0_API=$HOOK0_API
APP_ID=$APP_ID
EOF
```

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

Hook0 signs every webhook request with HMAC-SHA256. The default signature version is **v1**, which includes selected headers:

```
X-Hook0-Signature: t=1765443663,h=content-type x-custom-header,v1=85da0586ae0b711d...
```

### Signature Components
- **t**: Unix timestamp (seconds)
- **h**: Space-separated list of header names included in signature
- **v1**: HMAC-SHA256 signature (hex-encoded)

### Signature Computation (v1)

```
HMAC-SHA256(secret, timestamp + "." + header_names + "." + header_values + "." + payload)
```

Where `header_names` is the `h=` value and `header_values` is the values of those headers joined by `.`

## Step 2: Basic Signature Verification

### Node.js Implementation

```javascript
const crypto = require('crypto');
const express = require('express');
const app = express();

app.use('/webhooks', express.json());

function verifyHook0Signature(body, signature, headers, secret) {
  const parts = Object.fromEntries(signature.split(',').map(p => p.split('=')));
  const headerNames = parts.h ? parts.h.split(' ') : [];
  const headerValues = headerNames.map(h => headers[h] || '').join('.');
  // Reconstruct payload from parsed body
  const payload = JSON.stringify(body);
  const signedData = parts.h
    ? `${parts.t}.${parts.h}.${headerValues}.${payload}`
    : `${parts.t}.${payload}`;
  const expected = crypto.createHmac('sha256', secret).update(signedData).digest('hex');
  return parts.v1 === expected;
}

app.post('/webhooks/secure', (req, res) => {
  const signature = req.headers['x-hook0-signature'];

  if (!signature || !verifyHook0Signature(req.body, signature, req.headers, process.env.WEBHOOK_SECRET)) {
    return res.status(401).json({ error: 'Invalid signature' });
  }

  console.log('Verified webhook:', req.body);
  res.json({ status: 'verified' });
});

app.listen(3000);
```

:::tip Using the Official SDK
For production use, we recommend using the official `hook0-client` npm package which handles signature verification automatically:

```javascript
const { verifyWebhookSignature } = require('hook0-client');

app.post('/webhooks', express.json(), (req, res) => {
  const signature = req.headers['x-hook0-signature'];

  try {
    verifyWebhookSignature(signature, req.body, req.headers, process.env.WEBHOOK_SECRET, 300);
    // Process webhook...
    res.json({ received: true });
  } catch (error) {
    res.status(401).json({ error: error.message });
  }
});
```
:::

### Python Implementation

```python
import hmac
import hashlib
import os
from flask import Flask, request, jsonify

app = Flask(__name__)

def verify_hook0_signature(body: bytes, signature: str, headers: dict, secret: str) -> bool:
    parts = dict(p.split('=') for p in signature.split(','))
    header_names = parts.get('h', '').split(' ') if parts.get('h') else []
    header_values = '.'.join(headers.get(h, '') for h in header_names)
    signed_data = f"{parts['t']}.{parts['h']}.{header_values}.{body.decode()}" if parts.get('h') else f"{parts['t']}.{body.decode()}"
    expected = hmac.new(secret.encode(), signed_data.encode(), hashlib.sha256).hexdigest()
    return hmac.compare_digest(expected, parts['v1'])

@app.route('/webhooks/secure', methods=['POST'])
def handle_webhook():
    signature = request.headers.get('X-Hook0-Signature')
    if not signature or not verify_hook0_signature(request.data, signature, request.headers, os.environ['WEBHOOK_SECRET']):
        return jsonify({'error': 'Invalid signature'}), 401

    payload = request.get_json()
    print(f"Verified webhook: {payload}")
    return jsonify({'status': 'verified'})

if __name__ == '__main__':
    app.run(port=3000)
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
	"io"
	"net/http"
	"os"
	"strings"
)

func verifyHook0Signature(body []byte, signature, secret string, headers http.Header) bool {
	parts := make(map[string]string)
	for _, p := range strings.Split(signature, ",") {
		kv := strings.SplitN(p, "=", 2)
		if len(kv) == 2 {
			parts[kv[0]] = kv[1]
		}
	}

	var signedData string
	if h, ok := parts["h"]; ok && h != "" {
		headerNames := strings.Split(h, " ")
		var headerValues []string
		for _, name := range headerNames {
			headerValues = append(headerValues, headers.Get(name))
		}
		signedData = fmt.Sprintf("%s.%s.%s.%s", parts["t"], h, strings.Join(headerValues, "."), body)
	} else {
		signedData = fmt.Sprintf("%s.%s", parts["t"], body)
	}

	mac := hmac.New(sha256.New, []byte(secret))
	mac.Write([]byte(signedData))
	expected := hex.EncodeToString(mac.Sum(nil))

	return hmac.Equal([]byte(expected), []byte(parts["v1"]))
}

func webhookHandler(w http.ResponseWriter, r *http.Request) {
	body, _ := io.ReadAll(r.Body)
	signature := r.Header.Get("X-Hook0-Signature")

	if !verifyHook0Signature(body, signature, os.Getenv("WEBHOOK_SECRET"), r.Header) {
		http.Error(w, "Invalid signature", http.StatusUnauthorized)
		return
	}

	var payload map[string]interface{}
	json.Unmarshal(body, &payload)
	fmt.Printf("Verified webhook: %v\n", payload)

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(map[string]string{"status": "verified"})
}

func main() {
	http.HandleFunc("/webhooks/secure", webhookHandler)
	http.ListenAndServe(":3000", nil)
}
```

## Step 3: Advanced Signature Verification

### Secret Rotation Support

During rotation, accept signatures from both old and new secrets:

```javascript
function verifyWithSecrets(payload, signature, secrets) {
  return secrets.some(s => verifyHook0Signature(payload, signature, s));
}

const secrets = [process.env.WEBHOOK_SECRET, process.env.WEBHOOK_SECRET_OLD].filter(Boolean);
if (!verifyWithSecrets(JSON.stringify(req.body), signature, secrets)) {
  return res.status(401).json({ error: 'Invalid signature' });
}
```

### Replay Attack Protection

The timestamp (`t`) in the signature prevents replay attacks. To enforce a time window:

```javascript
function verifyHook0Signature(payload, signature, headers, secret, toleranceSec = 300) {
  const parts = Object.fromEntries(signature.split(',').map(p => p.split('=')));
  const now = Math.floor(Date.now() / 1000);
  if (Math.abs(now - parseInt(parts.t)) > toleranceSec) return false;

  const headerNames = parts.h ? parts.h.split(' ') : [];
  const headerValues = headerNames.map(h => headers[h] || '').join('.');
  const signedData = parts.h
    ? `${parts.t}.${parts.h}.${headerValues}.${payload}`
    : `${parts.t}.${payload}`;
  const expected = crypto.createHmac('sha256', secret).update(signedData).digest('hex');
  return parts.v1 === expected;
}
```

## Step 4: Custom Header Authentication

Add additional authentication headers to your subscriptions:

### Create Subscription with Custom Headers

```bash
curl -X POST "$HOOK0_API/subscriptions" \
  -H "Authorization: Bearer $HOOK0_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "'"$APP_ID"'",
    "is_enabled": true,
    "event_types": ["user.account.created"],
    "description": "Webhook with custom authentication",
    "label_key": "environment",
    "label_value": "production",
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
// Note: Express.js normalizes all header names to lowercase
app.post('/webhooks/authenticated', (req, res) => {
  const signature = req.headers['x-hook0-signature'];
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
  
  console.log('Multi-factor authenticated webhook:', req.body.event_type);

  res.json({ status: 'authenticated' });
});
```

## Step 5: IP Allowlisting

### Application-Level IP Filtering

:::warning Behind a Load Balancer or Proxy?
In most production environments (Kubernetes, Heroku, AWS, etc.), your application is behind a reverse proxy or load balancer. In this case, `req.ip` will return the proxy's IP, not Hook0's IP.

**Solution:** Configure Express to trust the proxy and use `X-Forwarded-For`:
```javascript
app.set('trust proxy', 1); // Trust first proxy
```
:::

```javascript
const allowedIPs = [
  '203.0.113.10',
  '203.0.113.11',
  '198.51.100.20'
];

function isIPAllowed(clientIP) {
  return allowedIPs.includes(clientIP);
}

// If behind a proxy, uncomment this line:
// app.set('trust proxy', 1);

app.post('/webhooks/ip-filtered', (req, res) => {
  // req.ip respects 'trust proxy' setting and uses X-Forwarded-For when enabled
  const clientIP = req.ip || req.connection.remoteAddress;
  
  if (!isIPAllowed(clientIP)) {
    console.log('Blocked request from:', clientIP);
    return res.status(403).json({ error: 'IP not allowed' });
  }
  
  // Continue with signature verification
  // Note: Express.js normalizes all header names to lowercase
  const signature = req.headers['x-hook0-signature'];
  if (!verifyHook0Signature(req.body, signature, process.env.WEBHOOK_SECRET)) {
    return res.status(401).json({ error: 'Invalid signature' });
  }

  res.json({ status: 'verified' });
});
```

## Step 6: Mutual TLS (mTLS)

For high-security environments, configure mutual TLS:

:::info Infrastructure-Level mTLS
In most production environments, TLS termination happens at the load balancer level (Nginx, AWS ALB, Cloudflare). The Node.js application never sees the client certificate directly.

If you need mTLS, configure it at your infrastructure layer and verify the client certificate there. The code below is for cases where Node.js handles TLS directly (rare in production).
:::

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
  // Note: Express.js normalizes all header names to lowercase
  const signature = req.headers['x-hook0-signature'];
  if (!verifyHook0Signature(req.body, signature, process.env.WEBHOOK_SECRET)) {
    return res.status(401).json({ error: 'Invalid signature' });
  }

  res.json({ status: 'mTLS verified' });
});

https.createServer(options, app).listen(8443, () => {
  console.log('HTTPS Server with mTLS running on port 8443');
});
```

## Step 7: Authentication Middleware

Create reusable authentication middleware that properly handles the Hook0 signature format (`t=...,v1=...`):

```javascript
// auth-middleware.js
const crypto = require('crypto');

class WebhookAuth {
  constructor(options = {}) {
    this.secrets = options.secrets || [];
    this.timestampTolerance = options.timestampTolerance || 300;
    this.allowedIPs = options.allowedIPs || [];
    this.requiredHeaders = options.requiredHeaders || {};
  }

  // Parse the Hook0 signature format: "t=123456,h=content-type,v1=abc123..."
  parseSignature(signature) {
    return Object.fromEntries(
      signature.split(',').map(part => {
        const [key, ...valueParts] = part.split('=');
        return [key, valueParts.join('=')]; // Handle '=' in values
      })
    );
  }

  // Compute the expected signature based on Hook0's spec
  computeSignature(payload, headers, parts, secret) {
    let signedData;
    if (parts.h) {
      const headerNames = parts.h.split(' ');
      const headerValues = headerNames.map(h => headers[h.toLowerCase()] || '').join('.');
      signedData = `${parts.t}.${parts.h}.${headerValues}.${payload}`;
    } else {
      signedData = `${parts.t}.${payload}`;
    }
    return crypto.createHmac('sha256', secret).update(signedData).digest('hex');
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
      // 1. IP verification
      const clientIP = req.ip || req.connection.remoteAddress;
      if (!this.verifyIP(clientIP)) {
        return res.status(403).json({ error: 'IP not allowed' });
      }

      // 2. Custom header verification
      if (!this.verifyHeaders(req.headers)) {
        return res.status(401).json({ error: 'Required headers missing or invalid' });
      }

      // 3. Signature header presence
      const signatureHeader = req.headers['x-hook0-signature'];
      if (!signatureHeader) {
        return res.status(401).json({ error: 'Missing signature' });
      }

      // 4. Parse the signature format: t=...,v1=...
      const parts = this.parseSignature(signatureHeader);
      if (!parts.t || !parts.v1) {
        return res.status(401).json({ error: 'Invalid signature format' });
      }

      // 5. Timestamp validation (replay attack protection)
      const now = Math.floor(Date.now() / 1000);
      if (Math.abs(now - parseInt(parts.t)) > this.timestampTolerance) {
        return res.status(401).json({ error: 'Timestamp too old or in future' });
      }

      // 6. Verify signature against all secrets (rotation support)
      const payload = JSON.stringify(req.body);
      let signatureValid = false;

      for (const secret of this.secrets) {
        const expected = this.computeSignature(payload, req.headers, parts, secret);
        try {
          if (crypto.timingSafeEqual(Buffer.from(parts.v1), Buffer.from(expected))) {
            signatureValid = true;
            break;
          }
        } catch (e) {
          // Length mismatch - signatures don't match
          continue;
        }
      }

      if (!signatureValid) {
        return res.status(401).json({ error: 'Invalid signature' });
      }

      // 7. Attach parsed body for the controller
      req.webhook = req.body;

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
    process.env.WEBHOOK_SECRET_PREVIOUS  // For rotation support
  ].filter(Boolean),
  timestampTolerance: 300,  // 5 minutes
  allowedIPs: [],  // Empty = allow all IPs
  requiredHeaders: {
    'x-webhook-source': 'hook0'  // Optional custom header check
  }
});

app.use('/webhooks', express.json());
app.use('/webhooks', webhookAuth.middleware());

app.post('/webhooks/secure', (req, res) => {
  // req.webhook contains the parsed and verified payload
  console.log('Authenticated webhook:', req.webhook.event_type);
  res.json({ status: 'authenticated' });
});
```

## Step 8: Testing Authentication

Create a test script to verify your authentication. The script must generate signatures in Hook0's format (`t=...,v1=...`):

```javascript
// test-auth.js (requires Node.js 18+)
const crypto = require('crypto');

// Generate a Hook0-compatible signature: t={timestamp},v1={hmac}
function generateHook0Signature(payload, secret, timestamp) {
  // Simple format without header signing (h= is optional)
  const signedData = `${timestamp}.${payload}`;
  const v1 = crypto.createHmac('sha256', secret).update(signedData).digest('hex');
  return `t=${timestamp},v1=${v1}`;
}

async function testWebhook(url, payloadObj, secret, extraHeaders = {}) {
  const payload = JSON.stringify(payloadObj);
  const timestamp = Math.floor(Date.now() / 1000);
  const signature = generateHook0Signature(payload, secret, timestamp);

  console.log(`\n📤 Sending webhook to ${url}`);
  console.log(`   Signature: ${signature}`);

  try {
    const response = await fetch(url, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'X-Hook0-Signature': signature,
        'X-Webhook-Source': 'hook0',
        ...extraHeaders
      },
      body: payload
    });

    const text = await response.text();
    if (response.ok) {
      console.log(`   ✅ ${response.status}: ${text}`);
    } else {
      console.log(`   ❌ ${response.status}: ${text}`);
    }
  } catch (err) {
    console.error(`   ❌ Request failed:`, err.message);
  }
}

async function runTests() {
  const SECRET = 'your-webhook-secret';
  const URL = 'http://localhost:3000/webhooks/secure';

  console.log('🧪 Testing Webhook Authentication\n');

  // Test 1: Valid signature
  console.log('Test 1: Valid signature');
  await testWebhook(URL, {
    event_id: 'evt_123',
    event_type: 'user.account.created',
    payload: { user_id: 'user_123' }
  }, SECRET);

  // Test 2: Invalid signature (wrong secret)
  console.log('\nTest 2: Invalid signature (wrong secret)');
  await testWebhook(URL, {
    event_id: 'evt_456',
    event_type: 'user.account.created',
    payload: { user_id: 'user_456' }
  }, 'wrong-secret');

  // Test 3: Expired timestamp (replay attack)
  console.log('\nTest 3: Expired timestamp (6 minutes ago)');
  const oldTimestamp = Math.floor(Date.now() / 1000) - 360;
  const payload = JSON.stringify({ event_id: 'evt_789', event_type: 'test' });
  const oldSignature = generateHook0Signature(payload, SECRET, oldTimestamp);

  const response = await fetch(URL, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'X-Hook0-Signature': oldSignature,
      'X-Webhook-Source': 'hook0'
    },
    body: payload
  });
  console.log(`   ${response.ok ? '✅' : '❌'} ${response.status}: ${await response.text()}`);

  console.log('\n✨ Tests completed!');
}

runTests();
```

Run the tests:
```bash
node test-auth.js
```

## Best Practices

### Security
- ✅ Always verify HMAC signatures
- ✅ Use timing-safe comparison for signatures
- ✅ Implement timestamp validation to prevent replay attacks
- ✅ Store secrets securely (environment variables, key management)
- ✅ Use HTTPS for all webhook endpoints
- ✅ Log authentication failures for monitoring
- ❌ Do not log webhook secrets
- ❌ Do not rely solely on IP allowlisting
- ❌ Do not skip signature verification

### Implementation
- ✅ Create reusable authentication middleware
- ✅ Support secret rotation with multiple valid secrets
- ✅ Return appropriate HTTP status codes
- ✅ Validate request format before processing
- ❌ Do not process webhooks with invalid authentication
- ❌ Do not expose internal error details in responses

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
