// Metrics Collection Helper for Wave 4 Load Tests
// Tracks cache hits/misses, session performance, and response times

import { Rate, Trend, Counter, Gauge } from 'k6/metrics';

// Cache metrics
export const cacheHits = new Counter('cache_hits');
export const cacheMisses = new Counter('cache_misses');
export const cacheHitRate = new Gauge('cache_hit_rate');

// Session metrics
export const sessionCreated = new Counter('session_created');
export const sessionValidated = new Counter('session_validated');
export const sessionExtended = new Counter('session_extended');
export const sessionInvalidated = new Counter('session_invalidated');
export const sessionValidationFailed = new Counter('session_validation_failed');

// Response time metrics
export const apiLatency = new Trend('api_latency_ms');
export const redisLatency = new Trend('redis_latency_ms');
export const authLatency = new Trend('auth_latency_ms');

// Error metrics
export const errorRate = new Rate('errors');
export const httpErrors = new Counter('http_errors');
export const validationErrors = new Counter('validation_errors');

// Connection metrics
export const activeConnections = new Gauge('active_connections');
export const maxConnections = new Gauge('max_connections');
export const memoryUsage = new Gauge('memory_usage_mb');

// Throughput metrics
export const requestCount = new Counter('request_count');
export const successCount = new Counter('request_success');
export const failureCount = new Counter('request_failure');

// Custom metrics for Wave 4
export const concurrentSessions = new Gauge('concurrent_sessions');
export const sessionLookupTime = new Trend('session_lookup_ms');
export const cacheHitLatency = new Trend('cache_hit_latency_us');
export const cacheMissLatency = new Trend('cache_miss_latency_us');

/**
 * Record cache hit
 */
export function recordCacheHit() {
  cacheHits.add(1);
  updateCacheHitRate();
}

/**
 * Record cache miss
 */
export function recordCacheMiss() {
  cacheMisses.add(1);
  updateCacheHitRate();
}

/**
 * Update cache hit rate gauge
 */
function updateCacheHitRate() {
  const hits = cacheHits.value;
  const misses = cacheMisses.value;
  const total = hits + misses;

  if (total > 0) {
    cacheHitRate.set((hits / total) * 100);
  }
}

/**
 * Record successful request
 * @param {number} latency Response time in ms
 */
export function recordSuccess(latency = 0) {
  successCount.add(1);
  requestCount.add(1);
  if (latency > 0) {
    apiLatency.add(latency);
  }
}

/**
 * Record failed request
 * @param {string} errorType Error type
 */
export function recordError(errorType = 'unknown') {
  failureCount.add(1);
  requestCount.add(1);
  errorRate.add(1);
  httpErrors.add(1);
}

/**
 * Record session operation
 * @param {string} operation 'created' | 'validated' | 'extended' | 'invalidated'
 */
export function recordSessionOperation(operation) {
  switch (operation) {
    case 'created':
      sessionCreated.add(1);
      concurrentSessions.set(sessionCreated.value - sessionInvalidated.value);
      break;
    case 'validated':
      sessionValidated.add(1);
      break;
    case 'extended':
      sessionExtended.add(1);
      break;
    case 'invalidated':
      sessionInvalidated.add(1);
      concurrentSessions.set(sessionCreated.value - sessionInvalidated.value);
      break;
    case 'validation_failed':
      sessionValidationFailed.add(1);
      break;
  }
}

/**
 * Record Redis operation latency
 * @param {number} latencyMs Latency in milliseconds
 */
export function recordRedisLatency(latencyMs) {
  redisLatency.add(latencyMs);
}

/**
 * Record authentication latency
 * @param {number} latencyMs Latency in milliseconds
 */
export function recordAuthLatency(latencyMs) {
  authLatency.add(latencyMs);
}

/**
 * Record session lookup time
 * @param {number} latencyMs Latency in milliseconds
 */
export function recordSessionLookup(latencyMs) {
  sessionLookupTime.add(latencyMs);
}

/**
 * Record cache hit latency
 * @param {number} latencyUs Latency in microseconds
 */
export function recordCacheHitLatency(latencyUs) {
  cacheHitLatency.add(latencyUs);
}

/**
 * Record cache miss latency
 * @param {number} latencyUs Latency in microseconds
 */
export function recordCacheMissLatency(latencyUs) {
  cacheMissLatency.add(latencyUs);
}

/**
 * Set active connection count
 * @param {number} count Connection count
 */
export function setActiveConnections(count) {
  activeConnections.set(count);
}

/**
 * Set maximum connection count
 * @param {number} count Max connection count
 */
export function setMaxConnections(count) {
  maxConnections.set(count);
}

/**
 * Set memory usage
 * @param {number} mb Memory in MB
 */
export function setMemoryUsage(mb) {
  memoryUsage.set(mb);
}

/**
 * Get current stats
 * @returns {Object} Current metrics
 */
export function getStats() {
  const totalRequests = requestCount.value;
  const totalSuccess = successCount.value;
  const totalFailure = failureCount.value;
  const totalCacheHits = cacheHits.value;
  const totalCacheMisses = cacheMisses.value;
  const totalSessions = sessionCreated.value;

  return {
    totalRequests,
    totalSuccess,
    totalFailure,
    successRate: totalRequests > 0 ? (totalSuccess / totalRequests) * 100 : 0,
    totalCacheHits,
    totalCacheMisses,
    cacheHitRate: (totalCacheHits + totalCacheMisses) > 0
      ? (totalCacheHits / (totalCacheHits + totalCacheMisses)) * 100
      : 0,
    totalSessions,
    concurrentSessions: concurrentSessions.value,
  };
}
