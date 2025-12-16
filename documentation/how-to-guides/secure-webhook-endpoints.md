# Securing Webhook Endpoints

This guide provides comprehensive security practices for webhook endpoints, from basic authentication to advanced security patterns. Learn how to protect your webhook receivers against common attacks and security vulnerabilities.

## Quick Start (5 minutes)

1. **Verify HMAC signature** - Check `X-Hook0-Signature` header
2. **Validate payload** - Use Joi/Zod to validate JSON structure
3. **Implement idempotency** - Track processed `event_id` to avoid duplicates
4. **Log requests** - Record all webhook attempts for debugging

For detailed implementation, see sections below.

## Security Threat Model

### Common Webhook Security Threats

:::danger Spoofing Attacks
Attackers sending fake webhook requests.

**Mitigation**: Signature verification, IP allowlisting
:::

:::warning Replay Attacks
Reusing legitimate webhook requests.

**Mitigation**: Timestamp validation, nonce tracking
:::

:::info Man-in-the-Middle
Intercepting and modifying webhook data.

**Mitigation**: HTTPS, certificate pinning
:::

:::danger Denial of Service
Overwhelming webhook endpoints with requests.

**Mitigation**: Rate limiting, request validation
:::

:::warning Data Injection
Malicious payloads causing application vulnerabilities.

**Mitigation**: Input validation, sanitization
:::

## Step 1: Implement Robust Signature Verification

Hook0 uses HMAC-SHA256 signatures with the v1 format. For complete signature verification implementation in JavaScript, Python, and Go, see [Implementing Webhook Authentication](../tutorials/webhook-authentication.md).

Key security considerations for signature verification:
- Use timing-safe comparison to prevent timing attacks
- Support multiple secrets for rotation
- Validate timestamp to prevent replay attacks
- Use raw request body (not parsed JSON)

## Step 2: Implement Advanced Input Validation

### Schema-Based Validation

```javascript
// webhook-validator.js
const Joi = require('joi');

// Define webhook payload schemas
const eventSchemas = {
  'user.account.created': Joi.object({
    event_id: Joi.string().uuid().required(),
    event_type: Joi.string().valid('user.account.created').required(),
    timestamp: Joi.string().isoDate().required(),
    payload: Joi.object({
      user_id: Joi.string().required(),
      email: Joi.string().email().required(),
      name: Joi.string().min(1).max(255).required(),
      plan: Joi.string().optional()
    }).required(),
    labels: Joi.object().optional()
  }),
  
  'order.completed': Joi.object({
    event_id: Joi.string().uuid().required(),
    event_type: Joi.string().valid('order.completed').required(),
    timestamp: Joi.string().isoDate().required(),
    payload: Joi.object({
      order_id: Joi.string().required(),
      customer_id: Joi.string().required(),
      total_amount: Joi.number().positive().required(),
      currency: Joi.string().length(3).uppercase().required()
    }).required(),
    labels: Joi.object().optional()
  })
};

function validateWebhookPayload(payload) {
  const schema = eventSchemas[payload.event_type];
  
  if (!schema) {
    return {
      valid: false,
      error: `Unknown event type: ${payload.event_type}`
    };
  }
  
  const { error, value } = schema.validate(payload, {
    abortEarly: false,
    stripUnknown: true
  });
  
  if (error) {
    return {
      valid: false,
      error: 'Validation failed',
      details: error.details.map(d => ({
        field: d.path.join('.'),
        message: d.message
      }))
    };
  }
  
  return { valid: true, data: value };
}

module.exports = { validateWebhookPayload };
```

### Sanitization and Security Filters

```javascript
// security-filters.js
const DOMPurify = require('isomorphic-dompurify');
const validator = require('validator');

class SecurityFilters {
  static sanitizeString(input, options = {}) {
    if (typeof input !== 'string') return input;
    
    let sanitized = input;
    
    // HTML sanitization
    if (options.allowHtml) {
      sanitized = DOMPurify.sanitize(sanitized);
    } else {
      sanitized = validator.escape(sanitized);
    }
    
    // Length limits
    if (options.maxLength) {
      sanitized = sanitized.substring(0, options.maxLength);
    }
    
    // Remove control characters
    sanitized = sanitized.replace(/[\x00-\x1F\x7F]/g, '');
    
    return sanitized;
  }
  
  static validateEmail(email) {
    return validator.isEmail(email) && email.length <= 254;
  }
  
  static validateURL(url) {
    return validator.isURL(url, {
      protocols: ['https'],
      require_protocol: true,
      require_valid_protocol: true
    });
  }
  
  static sanitizePayload(payload, rules = {}) {
    const sanitized = {};
    
    for (const [key, value] of Object.entries(payload)) {
      const rule = rules[key] || {};
      
      if (typeof value === 'string') {
        sanitized[key] = this.sanitizeString(value, rule);
      } else if (typeof value === 'object' && value !== null) {
        sanitized[key] = this.sanitizePayload(value, rule.nested || {});
      } else {
        sanitized[key] = value;
      }
    }
    
    return sanitized;
  }
}

module.exports = SecurityFilters;
```

## Step 3: Implement Request Deduplication

### Idempotency with In-Memory Storage

```javascript
// idempotency-manager.js
class IdempotencyManager {
  constructor(options = {}) {
    this.store = new Map();
    this.defaultTTL = options.ttl || 24 * 60 * 60 * 1000; // 24 hours in ms
    this.maxSize = options.maxSize || 10000;

    // Cleanup old entries periodically
    this.cleanupInterval = setInterval(() => this.cleanup(), 60 * 60 * 1000); // Every hour
  }

  generateKey(eventId, subscriptionId) {
    return `${subscriptionId}:${eventId}`;
  }

  async checkAndSetProcessed(eventId, subscriptionId, ttl = this.defaultTTL) {
    const key = this.generateKey(eventId, subscriptionId);
    const existing = this.store.get(key);

    if (existing) {
      // Key already exists, this is a duplicate
      return {
        isFirst: false,
        key,
        previousProcessing: existing.data
      };
    }

    // Key doesn't exist, set it
    this.store.set(key, {
      data: {
        processedAt: new Date().toISOString(),
        eventId,
        subscriptionId
      },
      expiresAt: Date.now() + ttl
    });

    // Enforce max size
    if (this.store.size > this.maxSize) {
      this.cleanup();
    }

    return { isFirst: true, key };
  }

  async markCompleted(eventId, subscriptionId, result) {
    const key = this.generateKey(eventId, subscriptionId);
    const entry = this.store.get(key);

    if (entry) {
      entry.data = {
        ...entry.data,
        completedAt: new Date().toISOString(),
        result: result,
        status: 'completed'
      };
    }
  }

  async markFailed(eventId, subscriptionId, error) {
    const key = this.generateKey(eventId, subscriptionId);
    const entry = this.store.get(key);

    if (entry) {
      entry.data = {
        ...entry.data,
        failedAt: new Date().toISOString(),
        error: {
          message: error.message,
          stack: error.stack
        },
        status: 'failed'
      };
    }
  }

  cleanup() {
    const now = Date.now();
    let removed = 0;

    for (const [key, entry] of this.store.entries()) {
      if (entry.expiresAt < now) {
        this.store.delete(key);
        removed++;
      }
    }

    console.log(`Cleaned up ${removed} expired entries. Current size: ${this.store.size}`);
  }

  destroy() {
    clearInterval(this.cleanupInterval);
    this.store.clear();
  }
}

// Usage in webhook handler
const idempotency = new IdempotencyManager({
  ttl: 24 * 60 * 60 * 1000, // 24 hours
  maxSize: 10000
});

app.post('/webhook', async (req, res) => {
  const { event_id } = req.webhook;
  const subscriptionId = req.headers['hook0-subscription-id'];

  try {
    const idempotencyCheck = await idempotency.checkAndSetProcessed(
      event_id,
      subscriptionId
    );

    if (!idempotencyCheck.isFirst) {
      console.log(`Duplicate webhook ignored: ${event_id}`);
      return res.json({
        status: 'duplicate',
        previousProcessing: idempotencyCheck.previousProcessing
      });
    }

    // Process the webhook
    const result = await processWebhook(req.webhook);

    await idempotency.markCompleted(event_id, subscriptionId, result);

    res.json({ status: 'processed', result });

  } catch (error) {
    await idempotency.markFailed(event_id, subscriptionId, error);

    console.error('Webhook processing failed:', error);
    res.status(500).json({ error: 'Processing failed' });
  }
});

module.exports = IdempotencyManager;
```

:::tip Production Note
For production environments with multiple instances, use PostgreSQL for idempotency storage instead of in-memory. Query the database to check if an event has been processed before.
:::

## Step 4: Implement IP Allowlisting and Geolocation

### Advanced IP Filtering

```javascript
// ip-security.js
const geoip = require('geoip-lite');
const ipRangeCheck = require('ip-range-check');

class IPSecurity {
  constructor(config = {}) {
    this.allowedIPs = config.allowedIPs || [];
    this.allowedRanges = config.allowedRanges || [];
    this.allowedCountries = config.allowedCountries || [];
    this.blockedCountries = config.blockedCountries || [];
    this.requireAllowlist = config.requireAllowlist || false;
  }
  
  isIPAllowed(clientIP) {
    // Remove IPv6 wrapper if present
    const ip = clientIP.replace(/^::ffff:/, '');
    
    // Check explicit IP allowlist
    if (this.allowedIPs.includes(ip)) {
      return { allowed: true, reason: 'explicit_allow' };
    }
    
    // Check IP ranges
    for (const range of this.allowedRanges) {
      if (ipRangeCheck(ip, range)) {
        return { allowed: true, reason: 'range_allow' };
      }
    }
    
    // Check geolocation
    const geo = geoip.lookup(ip);
    if (geo) {
      // Check blocked countries first
      if (this.blockedCountries.includes(geo.country)) {
        return { allowed: false, reason: 'country_blocked', country: geo.country };
      }
      
      // Check allowed countries
      if (this.allowedCountries.length > 0) {
        if (this.allowedCountries.includes(geo.country)) {
          return { allowed: true, reason: 'country_allow', country: geo.country };
        } else {
          return { allowed: false, reason: 'country_not_allowed', country: geo.country };
        }
      }
    }
    
    // If allowlist is required and no explicit allow, block
    if (this.requireAllowlist) {
      return { allowed: false, reason: 'not_in_allowlist' };
    }
    
    // Default allow if no specific rules matched
    return { allowed: true, reason: 'default_allow' };
  }
  
  middleware() {
    return (req, res, next) => {
      const clientIP = req.ip || 
                      req.connection.remoteAddress ||
                      req.headers['x-forwarded-for']?.split(',')[0]?.trim();
      
      const ipCheck = this.isIPAllowed(clientIP);
      
      // Log all IP checks for monitoring
      console.log('IP Security Check:', {
        ip: clientIP,
        allowed: ipCheck.allowed,
        reason: ipCheck.reason,
        country: ipCheck.country,
        timestamp: new Date().toISOString()
      });
      
      if (!ipCheck.allowed) {
        return res.status(403).json({
          error: 'Access denied',
          reason: ipCheck.reason,
          country: ipCheck.country
        });
      }
      
      req.clientIP = clientIP;
      req.ipSecurity = ipCheck;
      next();
    };
  }
}

module.exports = IPSecurity;
```

## Step 5: Implement Request Logging and Monitoring

### Comprehensive Request Logging

```javascript
// webhook-logger.js
const winston = require('winston');
const crypto = require('crypto');

class WebhookLogger {
  constructor(options = {}) {
    this.logger = winston.createLogger({
      level: options.level || 'info',
      format: winston.format.combine(
        winston.format.timestamp(),
        winston.format.errors({ stack: true }),
        winston.format.json()
      ),
      transports: [
        new winston.transports.Console(),
        new winston.transports.File({ 
          filename: options.logFile || 'webhooks.log' 
        })
      ]
    });
  }
  
  hashSensitiveData(data) {
    return crypto.createHash('sha256').update(JSON.stringify(data)).digest('hex').substring(0, 16);
  }
  
  logWebhookRequest(req, res, responseTime) {
    const requestId = crypto.randomUUID();
    const payloadHash = this.hashSensitiveData(req.webhook);
    
    const logData = {
      requestId,
      timestamp: new Date().toISOString(),
      method: req.method,
      path: req.path,
      statusCode: res.statusCode,
      responseTime,
      clientIP: req.clientIP,
      userAgent: req.headers['user-agent'],
      contentLength: req.headers['content-length'],
      payloadHash,
      eventType: req.webhook?.event_type,
      eventId: req.webhook?.event_id,
      signatureValid: req.webhookAuth?.signature?.valid,
      timestampValid: req.webhookAuth?.timestamp?.valid,
      ipSecurityReason: req.ipSecurity?.reason,
      ipSecurityCountry: req.ipSecurity?.country
    };
    
    if (res.statusCode >= 400) {
      this.logger.error('Webhook request failed', logData);
    } else {
      this.logger.info('Webhook request processed', logData);
    }
    
    return requestId;
  }
  
  logSecurityEvent(type, details) {
    this.logger.warn('Security event detected', {
      type,
      details,
      timestamp: new Date().toISOString()
    });
  }
  
  middleware() {
    return (req, res, next) => {
      const startTime = Date.now();
      
      // Override res.end to capture response time
      const originalEnd = res.end;
      res.end = (...args) => {
        const responseTime = Date.now() - startTime;
        this.logWebhookRequest(req, res, responseTime);
        originalEnd.apply(res, args);
      };
      
      next();
    };
  }
}

module.exports = WebhookLogger;
```

## Step 6: Implement Attack Detection

### Anomaly Detection System

```javascript
// anomaly-detector.js
class AnomalyDetector {
  constructor(options = {}) {
    this.suspiciousPatterns = options.suspiciousPatterns || [];
    this.rateLimits = options.rateLimits || {
      requestsPerMinute: 60,
      failuresPerMinute: 10
    };
    this.monitoring = {
      requests: new Map(),
      failures: new Map(),
      patterns: new Map()
    };
  }
  
  detectSuspiciousPayload(payload) {
    const payloadStr = JSON.stringify(payload).toLowerCase();
    const suspiciousPatterns = [
      /script\s*>/i,
      /<iframe/i,
      /javascript:/i,
      /on\w+\s*=/i,
      /\bexec\b/i,
      /\beval\b/i,
      /\.\./,  // Path traversal
      /union\s+select/i,  // SQL injection
      /insert\s+into/i,
      /drop\s+table/i
    ];
    
    for (const pattern of suspiciousPatterns) {
      if (pattern.test(payloadStr)) {
        return {
          suspicious: true,
          pattern: pattern.toString(),
          matched: payloadStr.match(pattern)?.[0]
        };
      }
    }
    
    return { suspicious: false };
  }
  
  trackRequest(clientIP, success = true) {
    const now = Date.now();
    const minute = Math.floor(now / 60000);
    
    // Track requests per minute
    const requestKey = `${clientIP}:${minute}`;
    this.monitoring.requests.set(
      requestKey,
      (this.monitoring.requests.get(requestKey) || 0) + 1
    );
    
    // Track failures per minute
    if (!success) {
      this.monitoring.failures.set(
        requestKey,
        (this.monitoring.failures.get(requestKey) || 0) + 1
      );
    }
    
    // Cleanup old entries
    this.cleanupOldEntries();
  }
  
  detectRateLimitViolation(clientIP) {
    const now = Date.now();
    const minute = Math.floor(now / 60000);
    const requestKey = `${clientIP}:${minute}`;
    
    const requests = this.monitoring.requests.get(requestKey) || 0;
    const failures = this.monitoring.failures.get(requestKey) || 0;
    
    return {
      requestsExceeded: requests > this.rateLimits.requestsPerMinute,
      failuresExceeded: failures > this.rateLimits.failuresPerMinute,
      requests,
      failures,
      limits: this.rateLimits
    };
  }
  
  cleanupOldEntries() {
    const now = Date.now();
    const currentMinute = Math.floor(now / 60000);
    
    // Keep only last 5 minutes of data
    for (const [key, value] of this.monitoring.requests.entries()) {
      const [ip, minute] = key.split(':');
      if (parseInt(minute) < currentMinute - 5) {
        this.monitoring.requests.delete(key);
        this.monitoring.failures.delete(key);
      }
    }
  }
  
  analyzeRequest(req) {
    const clientIP = req.clientIP;
    const payload = req.webhook;
    
    const analysis = {
      timestamp: new Date().toISOString(),
      clientIP,
      suspiciousPayload: this.detectSuspiciousPayload(payload),
      rateLimitViolation: this.detectRateLimitViolation(clientIP),
      anomalies: []
    };
    
    // Check for anomalies
    if (analysis.suspiciousPayload.suspicious) {
      analysis.anomalies.push('suspicious_payload');
    }
    
    if (analysis.rateLimitViolation.requestsExceeded) {
      analysis.anomalies.push('rate_limit_exceeded');
    }
    
    if (analysis.rateLimitViolation.failuresExceeded) {
      analysis.anomalies.push('failure_rate_exceeded');
    }
    
    return analysis;
  }
  
  middleware() {
    return (req, res, next) => {
      const analysis = this.analyzeRequest(req);
      
      // Track the request
      this.trackRequest(req.clientIP, res.statusCode < 400);
      
      // Block if anomalies detected
      if (analysis.anomalies.length > 0) {
        console.warn('Anomalies detected:', analysis);
        
        // You might want to block or just log depending on severity
        if (analysis.anomalies.includes('suspicious_payload')) {
          return res.status(403).json({
            error: 'Request blocked due to suspicious content'
          });
        }
      }
      
      req.anomalyAnalysis = analysis;
      next();
    };
  }
}

module.exports = AnomalyDetector;
```

## Step 7: Complete Secure Webhook Implementation

### Full Secure Webhook Handler

```javascript
// secure-webhook-server.js
const express = require('express');
const crypto = require('crypto');
const helmet = require('helmet');
const IPSecurity = require('./ip-security');
const WebhookLogger = require('./webhook-logger');
const AnomalyDetector = require('./anomaly-detector');
const IdempotencyManager = require('./idempotency-manager');
const { validateWebhookPayload } = require('./webhook-validator');

const app = express();

// Basic security headers
app.use(helmet({
  contentSecurityPolicy: {
    directives: {
      defaultSrc: ["'self'"]
    }
  }
}));

// Trust proxy for correct IP detection
app.set('trust proxy', 1);

// Hook0 signature verification (v1 format)
// See tutorials/webhook-authentication.md for detailed explanation
function verifyHook0Signature(payload, signature, headers, secret) {
  const parts = Object.fromEntries(signature.split(',').map(p => p.split('=')));
  const headerNames = parts.h ? parts.h.split(' ') : [];
  const headerValues = headerNames.map(h => headers[h] || '').join('.');
  const signedData = parts.h
    ? `${parts.t}.${parts.h}.${headerValues}.${payload}`
    : `${parts.t}.${payload}`;
  const expected = crypto.createHmac('sha256', secret).update(signedData).digest('hex');
  return parts.v1 === expected;
}

// Initialize security components
const ipSecurity = new IPSecurity({
  allowedCountries: ['US', 'CA', 'GB', 'DE', 'FR'],
  blockedCountries: ['CN', 'RU'],
  requireAllowlist: false
});

const webhookLogger = new WebhookLogger({
  level: 'info',
  logFile: 'secure-webhooks.log'
});

const anomalyDetector = new AnomalyDetector({
  rateLimits: {
    requestsPerMinute: 100,
    failuresPerMinute: 5
  }
});

const idempotencyManager = new IdempotencyManager({
  ttl: 24 * 60 * 60 * 1000,
  maxSize: 10000
});

// Middleware pipeline
app.use('/webhook', express.json({ limit: '1mb' }));
app.use('/webhook', ipSecurity.middleware());
app.use('/webhook', webhookLogger.middleware());
app.use('/webhook', anomalyDetector.middleware());

// Secure webhook handler
app.post('/webhook', async (req, res) => {
  try {
    // Verify signature
    const signature = req.headers['x-hook0-signature'];
    const secrets = [process.env.WEBHOOK_SECRET_CURRENT, process.env.WEBHOOK_SECRET_PREVIOUS].filter(Boolean);
    const isValid = secrets.some(s => verifyHook0Signature(JSON.stringify(req.body), signature, req.headers, s));

    if (!isValid) {
      return res.status(401).json({ error: 'Invalid signature' });
    }

    // Validate payload (already parsed by express.json())
    const webhook = req.body;
    const validation = validateWebhookPayload(webhook);
    if (!validation.valid) {
      return res.status(400).json({
        error: 'Payload validation failed',
        details: validation.details || validation.error
      });
    }

    // Check for duplicate processing
    const { event_id } = webhook;
    const subscriptionId = req.headers['hook0-subscription-id'] || 'default';
    
    const idempotencyCheck = await idempotencyManager.checkAndSetProcessed(
      event_id,
      subscriptionId
    );
    
    if (!idempotencyCheck.isFirst) {
      return res.json({
        status: 'duplicate_ignored',
        previousProcessing: idempotencyCheck.previousProcessing
      });
    }
    
    // Process the webhook securely
    const result = await processWebhookSecurely(validation.data);
    
    // Mark as completed
    await idempotencyManager.markCompleted(event_id, subscriptionId, result);
    
    res.json({
      status: 'processed',
      eventId: event_id,
      processedAt: new Date().toISOString()
    });
    
  } catch (error) {
    console.error('Secure webhook processing failed:', error);
    res.status(500).json({
      error: 'Processing failed',
      timestamp: new Date().toISOString()
    });
  }
});

async function processWebhookSecurely(webhook) {
  // Your secure webhook processing logic here
  console.log('Processing webhook:', webhook.event_type);
  
  // Simulate processing
  await new Promise(resolve => setTimeout(resolve, 100));
  
  return {
    processed: true,
    eventType: webhook.event_type,
    timestamp: new Date().toISOString()
  };
}

// Health check endpoint
app.get('/health', (req, res) => {
  res.json({
    status: 'healthy',
    timestamp: new Date().toISOString(),
    version: process.env.npm_package_version || '1.0.0'
  });
});

// Error handling
app.use((error, req, res, next) => {
  console.error('Unhandled error:', error);
  res.status(500).json({
    error: 'Internal server error',
    timestamp: new Date().toISOString()
  });
});

const PORT = process.env.PORT || 3000;
app.listen(PORT, () => {
  console.log(`Secure webhook server running on port ${PORT}`);
});

module.exports = app;
```

## Security Best Practices Summary

### Authentication and authorization
- ✅ Always verify HMAC signatures
- ✅ Use timing-safe comparison for signature verification
- ✅ Support signature algorithm flexibility
- ✅ Implement timestamp validation to prevent replay attacks
- ✅ Support multiple secrets for rotation

### Input Validation & Sanitization
- ✅ Validate all input against strict schemas
- ✅ Sanitize string inputs to prevent injection attacks
- ✅ Implement payload size limits
- ✅ Check content types strictly
- ✅ Use allowlists for expected values

### Rate Limiting & DoS Protection
- ✅ Implement rate limiting per IP/signature
- ✅ Use progressive rate limiting strategies
- ✅ Monitor for anomalous request patterns
- ✅ Implement circuit breakers for external dependencies

### Infrastructure Security
- ✅ Use HTTPS with valid certificates
- ✅ Implement proper firewall rules
- ✅ Use security headers (helmet.js)
- ✅ Run with minimal privileges
- ✅ Keep dependencies updated

### Monitoring & Logging
- ✅ Log all security events
- ✅ Monitor for suspicious patterns
- ✅ Set up alerting for security incidents
- ✅ Implement request tracing
- ✅ Regular security audit logs review

### Error Handling
- ✅ Never expose internal system details
- ✅ Use consistent error response formats
- ✅ Log errors with sufficient context
- ✅ Implement graceful degradation
- ✅ Return appropriate HTTP status codes

Ready to secure your webhook endpoints? Start with signature verification and gradually implement the additional security layers outlined in this guide.