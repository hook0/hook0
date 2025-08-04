# Implementing Custom Retry Logic

This guide covers advanced retry strategies for webhook deliveries, from basic exponential backoff to sophisticated circuit breaker patterns and custom retry policies based on response codes and business logic.

## Understanding Retry Strategies

### When to Retry
**Temporary Failures** 🔄
- Network timeouts (ETIMEDOUT, ECONNRESET)
- DNS resolution failures
- 5xx server errors (500, 502, 503, 504)
- 408 Request Timeout
- 429 Too Many Requests

**Permanent Failures** ❌
- 4xx client errors (400, 401, 403, 404, etc.)
- SSL certificate errors
- Invalid URLs
- Payload too large (413)

### Retry Patterns
- **Exponential Backoff**: Increasing delays between retries
- **Linear Backoff**: Fixed intervals between retries
- **Circuit Breaker**: Stop retrying after threshold failures
- **Adaptive Retry**: Adjust based on response patterns

## Step 1: Basic Exponential Backoff

### Simple Exponential Backoff Implementation

```javascript
// exponential-backoff.js
class ExponentialBackoff {
  constructor(options = {}) {
    this.initialDelay = options.initialDelay || 1000; // 1 second
    this.maxDelay = options.maxDelay || 300000; // 5 minutes
    this.multiplier = options.multiplier || 2;
    this.jitter = options.jitter || true;
    this.maxAttempts = options.maxAttempts || 5;
  }
  
  calculateDelay(attempt) {
    // Calculate base delay: initialDelay * (multiplier ^ (attempt - 1))
    let delay = this.initialDelay * Math.pow(this.multiplier, attempt - 1);
    
    // Cap at maximum delay
    delay = Math.min(delay, this.maxDelay);
    
    // Add jitter to prevent thundering herd
    if (this.jitter) {
      delay = delay * (0.5 + Math.random() * 0.5);
    }
    
    return Math.floor(delay);
  }
  
  async executeWithRetry(fn, context = {}) {
    let lastError;
    
    for (let attempt = 1; attempt <= this.maxAttempts; attempt++) {
      try {
        const result = await fn(attempt, context);
        
        // Success - return result
        return {
          success: true,
          result,
          attempt,
          totalAttempts: attempt
        };
        
      } catch (error) {
        lastError = error;
        
        // Check if we should retry
        if (!this.shouldRetry(error, attempt)) {
          break;
        }
        
        // Don't delay after the last attempt
        if (attempt < this.maxAttempts) {
          const delay = this.calculateDelay(attempt);
          console.log(`Attempt ${attempt} failed, retrying in ${delay}ms:`, error.message);
          await this.sleep(delay);
        }
      }
    }
    
    // All retries exhausted
    return {
      success: false,
      error: lastError,
      totalAttempts: this.maxAttempts
    };
  }
  
  shouldRetry(error, attempt) {
    // Don't retry if we've exceeded max attempts
    if (attempt >= this.maxAttempts) {
      return false;
    }
    
    // Retry on network errors
    if (error.code && ['ENOTFOUND', 'ECONNRESET', 'ETIMEDOUT', 'ECONNREFUSED'].includes(error.code)) {
      return true;
    }
    
    // Retry on 5xx server errors and specific 4xx errors
    if (error.status) {
      return error.status >= 500 || 
             error.status === 408 || 
             error.status === 429;
    }
    
    // Default: don't retry unknown errors
    return false;
  }
  
  sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
  }
}

// Usage example
const backoff = new ExponentialBackoff({
  initialDelay: 1000,
  maxDelay: 60000,
  multiplier: 2,
  maxAttempts: 5,
  jitter: true
});

async function deliverWebhook(url, payload) {
  return backoff.executeWithRetry(async (attempt) => {
    console.log(`Delivery attempt ${attempt} to ${url}`);
    
    const response = await fetch(url, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload),
      timeout: 30000
    });
    
    if (!response.ok) {
      const error = new Error(`HTTP ${response.status}: ${response.statusText}`);
      error.status = response.status;
      throw error;
    }
    
    return {
      statusCode: response.status,
      body: await response.text()
    };
  });
}

module.exports = ExponentialBackoff;
```

## Step 2: Advanced Retry with Circuit Breaker

### Circuit Breaker Pattern Implementation

```javascript
// circuit-breaker-retry.js
class CircuitBreakerRetry {
  constructor(options = {}) {
    this.failureThreshold = options.failureThreshold || 5;
    this.recoveryTimeout = options.recoveryTimeout || 60000; // 1 minute
    this.monitoringWindow = options.monitoringWindow || 300000; // 5 minutes
    
    // Circuit breaker states: CLOSED, OPEN, HALF_OPEN
    this.state = 'CLOSED';
    this.failures = [];
    this.lastFailure = null;
    this.lastSuccess = null;
    
    // Retry configuration
    this.maxAttempts = options.maxAttempts || 3;
    this.baseDelay = options.baseDelay || 1000;
    this.maxDelay = options.maxDelay || 30000;
  }
  
  async executeWithCircuitBreaker(fn, context = {}) {
    // Check circuit breaker state
    if (this.state === 'OPEN') {
      if (Date.now() - this.lastFailure < this.recoveryTimeout) {
        throw new Error('Circuit breaker is OPEN - requests blocked');
      }
      
      // Try to recover
      this.state = 'HALF_OPEN';
      console.log('Circuit breaker moving to HALF_OPEN state');
    }
    
    try {
      const result = await this.executeWithRetry(fn, context);
      
      if (result.success) {
        this.onSuccess();
        return result;
      } else {
        this.onFailure();
        throw result.error;
      }
      
    } catch (error) {
      this.onFailure();
      throw error;
    }
  }
  
  async executeWithRetry(fn, context) {
    let lastError;
    
    for (let attempt = 1; attempt <= this.maxAttempts; attempt++) {
      try {
        const result = await fn(attempt, context);
        return { success: true, result, attempt };
        
      } catch (error) {
        lastError = error;
        
        // In HALF_OPEN state, fail fast on any error
        if (this.state === 'HALF_OPEN') {
          break;
        }
        
        if (!this.shouldRetry(error, attempt)) {
          break;
        }
        
        if (attempt < this.maxAttempts) {
          const delay = this.calculateDelay(attempt);
          await this.sleep(delay);
        }
      }
    }
    
    return { success: false, error: lastError, attempt: this.maxAttempts };
  }
  
  onSuccess() {
    this.lastSuccess = Date.now();
    
    if (this.state === 'HALF_OPEN') {
      console.log('Circuit breaker recovered - moving to CLOSED state');
      this.state = 'CLOSED';
      this.failures = [];
    }
  }
  
  onFailure() {
    const now = Date.now();
    this.lastFailure = now;
    
    // Add failure to monitoring window
    this.failures.push(now);
    
    // Remove old failures outside monitoring window
    this.failures = this.failures.filter(
      timestamp => now - timestamp < this.monitoringWindow
    );
    
    // Check if we should open the circuit
    if (this.failures.length >= this.failureThreshold) {
      if (this.state !== 'OPEN') {
        console.log(`Circuit breaker OPEN - ${this.failures.length} failures in ${this.monitoringWindow}ms`);
        this.state = 'OPEN';
      }
    }
  }
  
  calculateDelay(attempt) {
    const delay = this.baseDelay * Math.pow(2, attempt - 1);
    return Math.min(delay, this.maxDelay);
  }
  
  shouldRetry(error, attempt) {
    if (attempt >= this.maxAttempts) return false;
    
    // Network errors
    if (error.code && ['ENOTFOUND', 'ECONNRESET', 'ETIMEDOUT'].includes(error.code)) {
      return true;
    }
    
    // Server errors
    if (error.status >= 500) return true;
    
    // Rate limiting
    if (error.status === 429) return true;
    
    return false;
  }
  
  sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
  }
  
  getStatus() {
    return {
      state: this.state,
      failures: this.failures.length,
      lastFailure: this.lastFailure,
      lastSuccess: this.lastSuccess,
      nextRetryAllowed: this.state === 'OPEN' 
        ? this.lastFailure + this.recoveryTimeout 
        : Date.now()
    };
  }
}

module.exports = CircuitBreakerRetry;
```

## Step 3: Adaptive Retry Based on Response Patterns

### Response-Aware Retry Logic

```javascript
// adaptive-retry.js
class AdaptiveRetry {
  constructor(options = {}) {
    this.baseDelay = options.baseDelay || 1000;
    this.maxDelay = options.maxDelay || 300000;
    this.maxAttempts = options.maxAttempts || 5;
    
    // Response pattern tracking
    this.responsePatterns = new Map();
    this.successRates = new Map();
    
    // Adaptive parameters
    this.learningWindow = options.learningWindow || 100; // Track last 100 requests
    this.adaptiveMultiplier = options.adaptiveMultiplier || 1.5;
  }
  
  async executeWithAdaptiveRetry(fn, context = {}) {
    const endpoint = context.endpoint || 'default';
    let lastError;
    
    for (let attempt = 1; attempt <= this.maxAttempts; attempt++) {
      try {
        const startTime = Date.now();
        const result = await fn(attempt, context);
        const duration = Date.now() - startTime;
        
        // Record success
        this.recordResponse(endpoint, {
          success: true,
          statusCode: result.statusCode || 200,
          duration,
          attempt
        });
        
        return {
          success: true,
          result,
          attempt,
          adaptiveDelay: this.calculateAdaptiveDelay(endpoint, attempt + 1)
        };
        
      } catch (error) {
        lastError = error;
        const duration = Date.now() - startTime;
        
        // Record failure
        this.recordResponse(endpoint, {
          success: false,
          statusCode: error.status || 0,
          duration,
          attempt,
          error: error.message
        });
        
        if (!this.shouldRetryAdaptive(error, attempt, endpoint)) {
          break;
        }
        
        if (attempt < this.maxAttempts) {
          const delay = this.calculateAdaptiveDelay(endpoint, attempt);
          console.log(`Adaptive retry ${attempt} for ${endpoint} in ${delay}ms`);
          await this.sleep(delay);
        }
      }
    }
    
    return {
      success: false,
      error: lastError,
      totalAttempts: this.maxAttempts,
      endpointStats: this.getEndpointStats(endpoint)
    };
  }
  
  recordResponse(endpoint, response) {
    if (!this.responsePatterns.has(endpoint)) {
      this.responsePatterns.set(endpoint, []);
    }
    
    const patterns = this.responsePatterns.get(endpoint);
    patterns.push({
      ...response,
      timestamp: Date.now()
    });
    
    // Keep only recent patterns
    if (patterns.length > this.learningWindow) {
      patterns.shift();
    }
    
    // Update success rate
    const recentPatterns = patterns.slice(-20); // Last 20 requests
    const successCount = recentPatterns.filter(p => p.success).length;
    const successRate = successCount / recentPatterns.length;
    
    this.successRates.set(endpoint, successRate);
  }
  
  calculateAdaptiveDelay(endpoint, attempt) {
    const baseDelay = this.baseDelay * Math.pow(2, attempt - 1);
    const patterns = this.responsePatterns.get(endpoint) || [];
    
    if (patterns.length < 5) {
      // Not enough data, use standard exponential backoff
      return Math.min(baseDelay, this.maxDelay);
    }
    
    // Analyze recent patterns
    const recentFailures = patterns
      .slice(-10)
      .filter(p => !p.success);
    
    // Calculate adaptive multiplier based on failure patterns
    let adaptiveMultiplier = 1;
    
    // If many recent 5xx errors, increase delay significantly
    const serverErrors = recentFailures.filter(f => f.statusCode >= 500).length;
    if (serverErrors > 3) {
      adaptiveMultiplier *= 3;
    }
    
    // If many timeouts, increase delay moderately
    const timeouts = recentFailures.filter(f => f.statusCode === 0).length;
    if (timeouts > 2) {
      adaptiveMultiplier *= 2;
    }
    
    // If rate limited, use longer delays
    const rateLimited = recentFailures.filter(f => f.statusCode === 429).length;
    if (rateLimited > 1) {
      adaptiveMultiplier *= 4;
    }
    
    // Adjust based on success rate
    const successRate = this.successRates.get(endpoint) || 1;
    if (successRate < 0.5) {
      adaptiveMultiplier *= 2;
    } else if (successRate > 0.9) {
      adaptiveMultiplier *= 0.8;
    }
    
    const adaptiveDelay = baseDelay * adaptiveMultiplier;
    return Math.min(adaptiveDelay, this.maxDelay);
  }
  
  shouldRetryAdaptive(error, attempt, endpoint) {
    if (attempt >= this.maxAttempts) return false;
    
    // Standard retry logic
    if (this.isRetryableError(error)) {
      return true;
    }
    
    // Adaptive logic based on endpoint patterns
    const patterns = this.responsePatterns.get(endpoint) || [];
    const successRate = this.successRates.get(endpoint) || 0;
    
    // If endpoint has been consistently failing, reduce retry attempts
    if (successRate < 0.2 && attempt >= 2) {
      console.log(`Low success rate (${successRate}) for ${endpoint}, stopping retries early`);
      return false;
    }
    
    // If this specific error type has never succeeded, don't retry
    const sameErrorPatterns = patterns.filter(p => 
      !p.success && p.statusCode === error.status
    );
    
    if (sameErrorPatterns.length > 5 && 
        !patterns.some(p => p.success && p.statusCode === error.status)) {
      console.log(`Error ${error.status} never succeeded for ${endpoint}, not retrying`);
      return false;
    }
    
    return false;
  }
  
  isRetryableError(error) {
    // Network errors
    if (error.code && ['ENOTFOUND', 'ECONNRESET', 'ETIMEDOUT'].includes(error.code)) {
      return true;
    }
    
    // Server errors and specific client errors
    if (error.status) {
      return error.status >= 500 || 
             error.status === 408 || 
             error.status === 429;
    }
    
    return false;
  }
  
  getEndpointStats(endpoint) {
    const patterns = this.responsePatterns.get(endpoint) || [];
    if (patterns.length === 0) return null;
    
    const recent = patterns.slice(-20);
    const successCount = recent.filter(p => p.success).length;
    const avgDuration = recent.reduce((sum, p) => sum + p.duration, 0) / recent.length;
    
    const statusCodes = {};
    recent.forEach(p => {
      statusCodes[p.statusCode] = (statusCodes[p.statusCode] || 0) + 1;
    });
    
    return {
      totalRequests: patterns.length,
      recentRequests: recent.length,
      successRate: successCount / recent.length,
      averageDuration: Math.round(avgDuration),
      statusCodeDistribution: statusCodes,
      lastUpdated: patterns[patterns.length - 1].timestamp
    };
  }
  
  sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
  }
}

module.exports = AdaptiveRetry;
```

## Step 4: Custom Retry Policies

### Policy-Based Retry System

```javascript
// retry-policies.js
class RetryPolicy {
  constructor(name, config) {
    this.name = name;
    this.config = config;
  }
  
  shouldRetry(error, attempt, context = {}) {
    throw new Error('shouldRetry must be implemented by subclass');
  }
  
  calculateDelay(attempt, context = {}) {
    throw new Error('calculateDelay must be implemented by subclass');
  }
  
  getMaxAttempts(context = {}) {
    return this.config.maxAttempts || 3;
  }
}

class ExponentialBackoffPolicy extends RetryPolicy {
  shouldRetry(error, attempt, context = {}) {
    if (attempt >= this.getMaxAttempts(context)) return false;
    
    // Retry on server errors and network issues
    return (error.status >= 500) || 
           (error.code && ['ENOTFOUND', 'ECONNRESET', 'ETIMEDOUT'].includes(error.code));
  }
  
  calculateDelay(attempt, context = {}) {
    const base = this.config.baseDelay || 1000;
    const multiplier = this.config.multiplier || 2;
    const maxDelay = this.config.maxDelay || 30000;
    
    let delay = base * Math.pow(multiplier, attempt - 1);
    
    if (this.config.jitter) {
      delay *= (0.5 + Math.random() * 0.5);
    }
    
    return Math.min(delay, maxDelay);
  }
}

class LinearBackoffPolicy extends RetryPolicy {
  shouldRetry(error, attempt, context = {}) {
    if (attempt >= this.getMaxAttempts(context)) return false;
    return error.status >= 500 || error.status === 429;
  }
  
  calculateDelay(attempt, context = {}) {
    const interval = this.config.interval || 5000;
    const jitter = this.config.jitter ? Math.random() * 1000 : 0;
    return interval + jitter;
  }
}

class RateLimitAwarePolicy extends RetryPolicy {
  shouldRetry(error, attempt, context = {}) {
    if (attempt >= this.getMaxAttempts(context)) return false;
    
    // Always retry rate limit errors
    if (error.status === 429) return true;
    
    // Retry server errors
    return error.status >= 500;
  }
  
  calculateDelay(attempt, context = {}) {
    // Check for Retry-After header
    if (context.headers && context.headers['retry-after']) {
      const retryAfter = parseInt(context.headers['retry-after']);
      if (!isNaN(retryAfter)) {
        return retryAfter * 1000; // Convert to milliseconds
      }
    }
    
    // Rate limit specific backoff
    if (context.statusCode === 429) {
      return this.config.rateLimitDelay || 60000; // 1 minute default
    }
    
    // Standard exponential backoff for other errors
    const base = this.config.baseDelay || 2000;
    return base * Math.pow(2, attempt - 1);
  }
}

class BusinessLogicPolicy extends RetryPolicy {
  shouldRetry(error, attempt, context = {}) {
    if (attempt >= this.getMaxAttempts(context)) return false;
    
    // Custom business logic
    const { eventType, priority } = context;
    
    // High priority events get more retries
    if (priority === 'high' && attempt < 10) {
      return this.isRetryableError(error);
    }
    
    // Critical event types always retry
    if (eventType && this.config.criticalEventTypes?.includes(eventType)) {
      return this.isRetryableError(error);
    }
    
    // Standard retry logic
    return this.isRetryableError(error) && attempt < 3;
  }
  
  isRetryableError(error) {
    return error.status >= 500 || 
           error.status === 408 || 
           error.status === 429 ||
           (error.code && ['ENOTFOUND', 'ECONNRESET', 'ETIMEDOUT'].includes(error.code));
  }
  
  calculateDelay(attempt, context = {}) {
    const { priority, eventType } = context;
    let baseDelay = this.config.baseDelay || 1000;
    
    // Faster retries for high priority
    if (priority === 'high') {
      baseDelay = 500;
    }
    
    // Slower retries for low priority
    if (priority === 'low') {
      baseDelay = 5000;
    }
    
    return baseDelay * Math.pow(2, attempt - 1);
  }
  
  getMaxAttempts(context = {}) {
    const { priority, eventType } = context;
    
    if (priority === 'high') return 10;
    if (priority === 'low') return 2;
    if (eventType && this.config.criticalEventTypes?.includes(eventType)) return 8;
    
    return this.config.maxAttempts || 3;
  }
}

class PolicyBasedRetry {
  constructor() {
    this.policies = new Map();
    this.defaultPolicy = 'exponential';
  }
  
  registerPolicy(name, policy) {
    this.policies.set(name, policy);
  }
  
  setDefaultPolicy(name) {
    this.defaultPolicy = name;
  }
  
  async executeWithPolicy(fn, context = {}) {
    const policyName = context.retryPolicy || this.defaultPolicy;
    const policy = this.policies.get(policyName);
    
    if (!policy) {
      throw new Error(`Unknown retry policy: ${policyName}`);
    }
    
    let lastError;
    const maxAttempts = policy.getMaxAttempts(context);
    
    for (let attempt = 1; attempt <= maxAttempts; attempt++) {
      try {
        const result = await fn(attempt, context);
        return {
          success: true,
          result,
          attempt,
          policy: policyName
        };
        
      } catch (error) {
        lastError = error;
        
        // Add response headers to context for policy decisions
        if (error.response?.headers) {
          context.headers = error.response.headers;
          context.statusCode = error.status;
        }
        
        if (!policy.shouldRetry(error, attempt, context)) {
          break;
        }
        
        if (attempt < maxAttempts) {
          const delay = policy.calculateDelay(attempt, context);
          console.log(`Policy ${policyName}: retry ${attempt} in ${delay}ms`);
          await this.sleep(delay);
        }
      }
    }
    
    return {
      success: false,
      error: lastError,
      totalAttempts: maxAttempts,
      policy: policyName
    };
  }
  
  sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
  }
}

// Setup and usage
const retryManager = new PolicyBasedRetry();

// Register policies
retryManager.registerPolicy('exponential', new ExponentialBackoffPolicy('exponential', {
  baseDelay: 1000,
  multiplier: 2,
  maxDelay: 60000,
  maxAttempts: 5,
  jitter: true
}));

retryManager.registerPolicy('linear', new LinearBackoffPolicy('linear', {
  interval: 5000,
  maxAttempts: 3,
  jitter: true
}));

retryManager.registerPolicy('rate-limit-aware', new RateLimitAwarePolicy('rate-limit-aware', {
  baseDelay: 2000,
  rateLimitDelay: 60000,
  maxAttempts: 5
}));

retryManager.registerPolicy('business-logic', new BusinessLogicPolicy('business-logic', {
  baseDelay: 1000,
  maxAttempts: 3,
  criticalEventTypes: ['payment.processed', 'user.registered']
}));

module.exports = { 
  PolicyBasedRetry, 
  ExponentialBackoffPolicy, 
  LinearBackoffPolicy, 
  RateLimitAwarePolicy, 
  BusinessLogicPolicy 
};
```

## Step 5: Webhook-Specific Retry Implementation

### Complete Webhook Retry System

```javascript
// webhook-retry-system.js
const { PolicyBasedRetry, BusinessLogicPolicy } = require('./retry-policies');

class WebhookRetrySystem {
  constructor(options = {}) {
    this.retryManager = new PolicyBasedRetry();
    this.setupPolicies();
    
    this.defaultTimeout = options.timeout || 30000;
    this.trackingEnabled = options.tracking !== false;
    this.attemptHistory = new Map();
  }
  
  setupPolicies() {
    // Different policies for different scenarios
    this.retryManager.registerPolicy('standard', new BusinessLogicPolicy('standard', {
      baseDelay: 1000,
      maxAttempts: 3
    }));
    
    this.retryManager.registerPolicy('critical', new BusinessLogicPolicy('critical', {
      baseDelay: 500,
      maxAttempts: 8,
      criticalEventTypes: ['payment.completed', 'order.shipped', 'user.registered']
    }));
    
    this.retryManager.registerPolicy('bulk', new BusinessLogicPolicy('bulk', {
      baseDelay: 2000,
      maxAttempts: 2 // Fewer retries for bulk operations
    }));
  }
  
  async deliverWebhook(webhook, subscription, options = {}) {
    const context = {
      webhook,
      subscription,
      eventType: webhook.event_type,
      priority: this.determinePriority(webhook, subscription),
      retryPolicy: options.retryPolicy || this.selectPolicy(webhook, subscription),
      endpoint: subscription.target.url,
      ...options
    };
    
    const deliveryId = `${webhook.event_id}-${subscription.id}`;
    
    if (this.trackingEnabled) {
      this.attemptHistory.set(deliveryId, {
        startTime: Date.now(),
        attempts: []
      });
    }
    
    const result = await this.retryManager.executeWithPolicy(
      async (attempt, ctx) => {
        return this.performWebhookDelivery(webhook, subscription, attempt, ctx);
      },
      context
    );
    
    if (this.trackingEnabled) {
      const history = this.attemptHistory.get(deliveryId);
      history.endTime = Date.now();
      history.totalDuration = history.endTime - history.startTime;
      
      // Clean up old history
      this.cleanupHistory();
    }
    
    return {
      ...result,
      deliveryId,
      webhook: webhook.event_id,
      subscription: subscription.id
    };
  }
  
  async performWebhookDelivery(webhook, subscription, attempt, context) {
    const startTime = Date.now();
    const deliveryId = `${webhook.event_id}-${subscription.id}`;
    
    // Prepare webhook payload
    const payload = {
      event_id: webhook.event_id,
      event_type: webhook.event_type,
      payload: webhook.payload,
      labels: webhook.labels || {},
      timestamp: webhook.created_at || new Date().toISOString()
    };
    
    // Generate signature
    const signature = this.generateSignature(
      JSON.stringify(payload),
      subscription.secret
    );
    
    // Prepare headers
    const headers = {
      'Content-Type': 'application/json',
      'Hook0-Signature': signature,
      'Hook0-Event-Type': webhook.event_type,
      'Hook0-Event-Id': webhook.event_id,
      'Hook0-Attempt': attempt.toString(),
      'User-Agent': 'Hook0/1.0',
      ...(subscription.target.headers || {})
    };
    
    try {
      const response = await fetch(subscription.target.url, {
        method: subscription.target.method || 'POST',
        headers,
        body: JSON.stringify(payload),
        timeout: this.defaultTimeout
      });
      
      const responseBody = await this.safeReadResponse(response);
      const duration = Date.now() - startTime;
      
      // Track attempt
      if (this.trackingEnabled) {
        this.trackAttempt(deliveryId, {
          attempt,
          success: response.ok,
          statusCode: response.status,
          duration,
          responseBody: responseBody.substring(0, 500)
        });
      }
      
      if (!response.ok) {
        const error = new Error(`HTTP ${response.status}: ${response.statusText}`);
        error.status = response.status;
        error.response = { headers: response.headers };
        throw error;
      }
      
      return {
        statusCode: response.status,
        responseBody,
        duration,
        headers: Object.fromEntries(response.headers.entries())
      };
      
    } catch (error) {
      const duration = Date.now() - startTime;
      
      // Track failed attempt
      if (this.trackingEnabled) {
        this.trackAttempt(deliveryId, {
          attempt,
          success: false,
          error: error.message,
          duration,
          statusCode: error.status || 0
        });
      }
      
      throw error;
    }
  }
  
  determinePriority(webhook, subscription) {
    // Priority based on event type
    const highPriorityEvents = [
      'payment.completed',
      'payment.failed',
      'user.registered',
      'order.shipped'
    ];
    
    if (highPriorityEvents.includes(webhook.event_type)) {
      return 'high';
    }
    
    // Priority based on subscription metadata
    if (subscription.metadata?.priority) {
      return subscription.metadata.priority;
    }
    
    // Priority based on labels
    if (webhook.labels?.priority) {
      return webhook.labels.priority;
    }
    
    return 'normal';
  }
  
  selectPolicy(webhook, subscription) {
    const priority = this.determinePriority(webhook, subscription);
    
    if (priority === 'high') return 'critical';
    if (subscription.metadata?.bulk === true) return 'bulk';
    
    return 'standard';
  }
  
  generateSignature(payload, secret) {
    return 'sha256=' + require('crypto')
      .createHmac('sha256', secret)
      .update(payload)
      .digest('hex');
  }
  
  async safeReadResponse(response) {
    try {
      return await response.text();
    } catch (error) {
      return 'Failed to read response body';
    }
  }
  
  trackAttempt(deliveryId, attemptData) {
    const history = this.attemptHistory.get(deliveryId);
    if (history) {
      history.attempts.push({
        ...attemptData,
        timestamp: Date.now()
      });
    }
  }
  
  getDeliveryHistory(deliveryId) {
    return this.attemptHistory.get(deliveryId);
  }
  
  getDeliveryStats() {
    const histories = Array.from(this.attemptHistory.values());
    const stats = {
      totalDeliveries: histories.length,
      successful: 0,
      failed: 0,
      totalAttempts: 0,
      averageDuration: 0,
      retryRate: 0
    };
    
    let totalDuration = 0;
    let deliveriesWithRetries = 0;
    
    histories.forEach(history => {
      const lastAttempt = history.attempts[history.attempts.length - 1];
      if (lastAttempt?.success) {
        stats.successful++;
      } else {
        stats.failed++;
      }
      
      stats.totalAttempts += history.attempts.length;
      totalDuration += history.totalDuration || 0;
      
      if (history.attempts.length > 1) {
        deliveriesWithRetries++;
      }
    });
    
    stats.averageDuration = histories.length > 0 ? totalDuration / histories.length : 0;
    stats.retryRate = histories.length > 0 ? (deliveriesWithRetries / histories.length) * 100 : 0;
    
    return stats;
  }
  
  cleanupHistory() {
    const now = Date.now();
    const maxAge = 24 * 60 * 60 * 1000; // 24 hours
    
    for (const [deliveryId, history] of this.attemptHistory.entries()) {
      if (now - history.startTime > maxAge) {
        this.attemptHistory.delete(deliveryId);
      }
    }
  }
}

module.exports = WebhookRetrySystem;
```

## Step 6: Testing Retry Logic

### Retry Testing Framework

```javascript
// retry-test-framework.js
class RetryTestFramework {
  constructor() {
    this.scenarios = [];
    this.results = [];
  }
  
  addScenario(name, config) {
    this.scenarios.push({ name, config });
  }
  
  async runTests() {
    console.log(`Running ${this.scenarios.length} retry test scenarios...\n`);
    
    for (const scenario of this.scenarios) {
      console.log(`Testing: ${scenario.name}`);
      
      try {
        const result = await this.runScenario(scenario);
        this.results.push({ scenario: scenario.name, ...result });
        
        console.log(`✅ ${scenario.name}: ${result.totalAttempts} attempts, ${result.success ? 'SUCCESS' : 'FAILED'}`);
      } catch (error) {
        console.log(`❌ ${scenario.name}: Error - ${error.message}`);
        this.results.push({ 
          scenario: scenario.name, 
          success: false, 
          error: error.message 
        });
      }
      
      console.log('');
    }
    
    this.printSummary();
  }
  
  async runScenario(scenario) {
    const { config } = scenario;
    const retrySystem = new WebhookRetrySystem();
    
    // Mock webhook and subscription
    const webhook = {
      event_id: `test-${Date.now()}`,
      event_type: config.eventType || 'test.event',
      payload: { test: true },
      labels: config.labels || {}
    };
    
    const subscription = {
      id: 'test-subscription',
      target: {
        url: config.url || 'http://test.example.com/webhook',
        method: 'POST'
      },
      secret: 'test-secret',
      metadata: config.metadata || {}
    };
    
    // Mock fetch to simulate different responses
    const originalFetch = global.fetch;
    global.fetch = this.createMockFetch(config.responses || []);
    
    try {
      const result = await retrySystem.deliverWebhook(webhook, subscription, {
        retryPolicy: config.retryPolicy
      });
      
      return result;
    } finally {
      global.fetch = originalFetch;
    }
  }
  
  createMockFetch(responses) {
    let callCount = 0;
    
    return async (url, options) => {
      const response = responses[Math.min(callCount, responses.length - 1)];
      callCount++;
      
      // Simulate network delay
      if (response.delay) {
        await new Promise(resolve => setTimeout(resolve, response.delay));
      }
      
      // Simulate network error
      if (response.networkError) {
        const error = new Error(response.networkError);
        error.code = response.errorCode || 'ECONNRESET';
        throw error;
      }
      
      // Return mock response
      return {
        ok: response.status >= 200 && response.status < 300,
        status: response.status || 200,
        statusText: response.statusText || 'OK',
        headers: new Map(Object.entries(response.headers || {})),
        text: async () => response.body || 'OK'
      };
    };
  }
  
  printSummary() {
    console.log('=== Test Summary ===');
    
    const total = this.results.length;
    const passed = this.results.filter(r => r.success).length;
    const failed = total - passed;
    
    console.log(`Total scenarios: ${total}`);
    console.log(`Passed: ${passed}`);
    console.log(`Failed: ${failed}`);
    console.log(`Success rate: ${((passed / total) * 100).toFixed(2)}%`);
    
    if (failed > 0) {
      console.log('\nFailed scenarios:');
      this.results
        .filter(r => !r.success)
        .forEach(r => {
          console.log(`- ${r.scenario}: ${r.error || 'Unknown error'}`);
        });
    }
  }
}

// Test scenarios
async function runRetryTests() {
  const testFramework = new RetryTestFramework();
  
  // Test 1: Server error with eventual success
  testFramework.addScenario('Server Error Recovery', {
    responses: [
      { status: 500, body: 'Internal Server Error' },
      { status: 500, body: 'Still failing' },
      { status: 200, body: 'OK' }
    ],
    retryPolicy: 'standard'
  });
  
  // Test 2: Rate limiting with Retry-After
  testFramework.addScenario('Rate Limit Handling', {
    responses: [
      { status: 429, headers: { 'retry-after': '2' }, body: 'Rate limited' },
      { status: 200, body: 'OK' }
    ],
    retryPolicy: 'rate-limit-aware'
  });
  
  // Test 3: Network timeout recovery
  testFramework.addScenario('Network Timeout Recovery', {
    responses: [
      { networkError: 'ETIMEDOUT', errorCode: 'ETIMEDOUT' },
      { networkError: 'ECONNRESET', errorCode: 'ECONNRESET' },
      { status: 200, body: 'OK' }
    ],
    retryPolicy: 'standard'
  });
  
  // Test 4: Permanent failure (no retry)
  testFramework.addScenario('Permanent Failure', {
    responses: [
      { status: 404, body: 'Not Found' }
    ],
    retryPolicy: 'standard'
  });
  
  // Test 5: Critical event with more retries
  testFramework.addScenario('Critical Event Retries', {
    eventType: 'payment.completed',
    responses: [
      { status: 502, body: 'Bad Gateway' },
      { status: 503, body: 'Service Unavailable' },
      { status: 500, body: 'Internal Server Error' },
      { status: 200, body: 'OK' }
    ],
    retryPolicy: 'critical'
  });
  
  await testFramework.runTests();
}

if (require.main === module) {
  runRetryTests().catch(console.error);
}

module.exports = RetryTestFramework;
```

## Best Practices for Custom Retry Logic

### Configuration Guidelines
- ✅ Use exponential backoff with jitter
- ✅ Set reasonable maximum delays (< 5 minutes)
- ✅ Limit retry attempts (3-5 for most cases)
- ✅ Respect Retry-After headers
- ✅ Implement circuit breakers for chronic failures

### Error Handling
- ✅ Distinguish temporary from permanent failures
- ✅ Log retry attempts with context
- ✅ Track retry metrics and success rates
- ✅ Implement dead letter queues for final failures
- ✅ Alert on high retry rates

### Performance Considerations
- ✅ Use async/await for non-blocking retries
- ✅ Implement concurrency limits
- ✅ Pool HTTP connections
- ✅ Monitor resource usage during high retry volumes
- ✅ Consider batch retry processing

### Testing Strategy
- ✅ Test all retry scenarios
- ✅ Verify exponential backoff timing
- ✅ Test circuit breaker thresholds
- ✅ Validate policy selection logic
- ✅ Load test retry behavior

Ready to implement sophisticated retry logic? Start with the basic exponential backoff and gradually add advanced features like circuit breakers and adaptive policies based on your specific requirements.