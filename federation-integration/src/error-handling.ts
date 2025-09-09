/**
 * Error Handling and Resilience Patterns
 * Comprehensive error recovery, circuit breaker, and retry mechanisms for federation integration
 */

export interface RetryConfig {
  maxRetries: number;
  baseDelay: number;
  maxDelay: number;
  exponentialBase: number;
  jitter: boolean;
}

export interface CircuitBreakerConfig {
  failureThreshold: number;
  resetTimeout: number;
  monitoringPeriod: number;
}

export enum CircuitBreakerState {
  Closed = 'closed',
  Open = 'open',
  HalfOpen = 'half_open',
}

export class FederationError extends Error {
  constructor(
    message: string,
    public readonly code: string,
    public readonly retryable: boolean = true,
    public readonly cause?: Error
  ) {
    super(message);
    this.name = 'FederationError';
  }
}

export class RetryExhaustedError extends FederationError {
  constructor(
    attempts: number,
    lastError: Error
  ) {
    super(
      `Retry exhausted after ${attempts} attempts. Last error: ${lastError.message}`,
      'RETRY_EXHAUSTED',
      false,
      lastError
    );
  }
}

export class CircuitOpenError extends FederationError {
  constructor(service: string) {
    super(
      `Circuit breaker is open for service: ${service}`,
      'CIRCUIT_OPEN',
      false
    );
  }
}

/**
 * Exponential backoff retry mechanism with jitter
 */
export class RetryManager {
  private config: RetryConfig;

  constructor(config: Partial<RetryConfig> = {}) {
    this.config = {
      maxRetries: config.maxRetries || 3,
      baseDelay: config.baseDelay || 1000,
      maxDelay: config.maxDelay || 30000,
      exponentialBase: config.exponentialBase || 2,
      jitter: config.jitter ?? true,
    };
  }

  /**
   * Execute function with retry logic
   */
  async execute<T>(
    operation: () => Promise<T>,
    context: string = 'unknown'
  ): Promise<T> {
    let lastError: Error;
    
    for (let attempt = 0; attempt <= this.config.maxRetries; attempt++) {
      try {
        if (attempt > 0) {
          const delay = this.calculateDelay(attempt);
          console.log(`Retrying ${context} (attempt ${attempt}/${this.config.maxRetries}) after ${delay}ms`);
          await this.sleep(delay);
        }

        return await operation();
        
      } catch (error) {
        lastError = error as Error;
        
        // Don't retry if error is not retryable
        if (error instanceof FederationError && !error.retryable) {
          throw error;
        }
        
        // Don't retry on last attempt
        if (attempt === this.config.maxRetries) {
          break;
        }
        
        console.warn(`${context} failed on attempt ${attempt + 1}:`, lastError.message);
      }
    }

    throw new RetryExhaustedError(this.config.maxRetries + 1, lastError!);
  }

  /**
   * Calculate delay with exponential backoff and optional jitter
   */
  private calculateDelay(attempt: number): number {
    let delay = this.config.baseDelay * Math.pow(this.config.exponentialBase, attempt - 1);
    
    // Apply maximum delay limit
    delay = Math.min(delay, this.config.maxDelay);
    
    // Add jitter to prevent thundering herd
    if (this.config.jitter) {
      delay = delay * (0.5 + Math.random() * 0.5);
    }
    
    return Math.floor(delay);
  }

  /**
   * Sleep for specified milliseconds
   */
  private sleep(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }
}

/**
 * Circuit breaker pattern implementation
 */
export class CircuitBreaker {
  private state: CircuitBreakerState = CircuitBreakerState.Closed;
  private failureCount: number = 0;
  private lastFailureTime: number = 0;
  private config: CircuitBreakerConfig;
  private serviceName: string;

  constructor(serviceName: string, config: Partial<CircuitBreakerConfig> = {}) {
    this.serviceName = serviceName;
    this.config = {
      failureThreshold: config.failureThreshold || 5,
      resetTimeout: config.resetTimeout || 60000, // 1 minute
      monitoringPeriod: config.monitoringPeriod || 300000, // 5 minutes
    };
  }

  /**
   * Execute operation with circuit breaker protection
   */
  async execute<T>(operation: () => Promise<T>): Promise<T> {
    if (this.state === CircuitBreakerState.Open) {
      if (this.shouldAttemptReset()) {
        this.state = CircuitBreakerState.HalfOpen;
        console.log(`Circuit breaker for ${this.serviceName} moved to half-open state`);
      } else {
        throw new CircuitOpenError(this.serviceName);
      }
    }

    try {
      const result = await operation();
      this.onSuccess();
      return result;
      
    } catch (error) {
      this.onFailure();
      throw error;
    }
  }

  /**
   * Handle successful operation
   */
  private onSuccess(): void {
    this.failureCount = 0;
    if (this.state === CircuitBreakerState.HalfOpen) {
      this.state = CircuitBreakerState.Closed;
      console.log(`Circuit breaker for ${this.serviceName} reset to closed state`);
    }
  }

  /**
   * Handle failed operation
   */
  private onFailure(): void {
    this.failureCount++;
    this.lastFailureTime = Date.now();

    if (this.failureCount >= this.config.failureThreshold) {
      this.state = CircuitBreakerState.Open;
      console.error(
        `Circuit breaker for ${this.serviceName} opened after ${this.failureCount} failures`
      );
    }
  }

  /**
   * Check if circuit breaker should attempt reset
   */
  private shouldAttemptReset(): boolean {
    return Date.now() - this.lastFailureTime >= this.config.resetTimeout;
  }

  /**
   * Get current circuit breaker status
   */
  getStatus() {
    return {
      service: this.serviceName,
      state: this.state,
      failureCount: this.failureCount,
      lastFailureTime: this.lastFailureTime,
    };
  }
}

/**
 * Fallback strategy manager
 */
export class FallbackManager {
  private fallbacks: Map<string, () => Promise<any>> = new Map();

  /**
   * Register fallback function for operation
   */
  registerFallback<T>(operationName: string, fallbackFn: () => Promise<T>): void {
    this.fallbacks.set(operationName, fallbackFn);
  }

  /**
   * Execute operation with fallback if available
   */
  async executeWithFallback<T>(
    operationName: string,
    primaryOperation: () => Promise<T>
  ): Promise<T> {
    try {
      return await primaryOperation();
    } catch (error) {
      const fallback = this.fallbacks.get(operationName);
      
      if (fallback) {
        console.warn(`Primary operation ${operationName} failed, using fallback`);
        try {
          return await fallback();
        } catch (fallbackError) {
          console.error(`Fallback for ${operationName} also failed:`, fallbackError);
          throw error; // Throw original error
        }
      }
      
      throw error;
    }
  }
}

/**
 * Health monitor for federation services
 */
export class HealthMonitor {
  private healthChecks: Map<string, () => Promise<boolean>> = new Map();
  private lastHealthStatus: Map<string, boolean> = new Map();
  private checkInterval: number;
  private intervalId?: NodeJS.Timeout;

  constructor(checkIntervalMs: number = 30000) {
    this.checkInterval = checkIntervalMs;
  }

  /**
   * Register health check for service
   */
  registerHealthCheck(serviceName: string, healthCheckFn: () => Promise<boolean>): void {
    this.healthChecks.set(serviceName, healthCheckFn);
  }

  /**
   * Start periodic health monitoring
   */
  startMonitoring(): void {
    if (this.intervalId) {
      this.stopMonitoring();
    }

    this.intervalId = setInterval(async () => {
      await this.checkAllServices();
    }, this.checkInterval);

    console.log(`Health monitoring started (interval: ${this.checkInterval}ms)`);
  }

  /**
   * Stop health monitoring
   */
  stopMonitoring(): void {
    if (this.intervalId) {
      clearInterval(this.intervalId);
      this.intervalId = undefined;
      console.log('Health monitoring stopped');
    }
  }

  /**
   * Check health of all registered services
   */
  async checkAllServices(): Promise<Map<string, boolean>> {
    const results = new Map<string, boolean>();

    for (const [serviceName, healthCheck] of this.healthChecks) {
      try {
        const isHealthy = await healthCheck();
        results.set(serviceName, isHealthy);
        
        const wasHealthy = this.lastHealthStatus.get(serviceName);
        if (wasHealthy !== isHealthy) {
          console.log(`Service ${serviceName} health changed: ${wasHealthy} -> ${isHealthy}`);
        }
        
        this.lastHealthStatus.set(serviceName, isHealthy);
        
      } catch (error) {
        results.set(serviceName, false);
        this.lastHealthStatus.set(serviceName, false);
        console.error(`Health check failed for ${serviceName}:`, error);
      }
    }

    return results;
  }

  /**
   * Get current health status
   */
  getHealthStatus(): Map<string, boolean> {
    return new Map(this.lastHealthStatus);
  }

  /**
   * Check if specific service is healthy
   */
  isServiceHealthy(serviceName: string): boolean {
    return this.lastHealthStatus.get(serviceName) ?? false;
  }
}

/**
 * Comprehensive resilience manager combining all patterns
 */
export class ResilienceManager {
  private retryManager: RetryManager;
  private circuitBreakers: Map<string, CircuitBreaker> = new Map();
  private fallbackManager: FallbackManager;
  private healthMonitor: HealthMonitor;

  constructor(
    retryConfig?: Partial<RetryConfig>,
    circuitBreakerConfig?: Partial<CircuitBreakerConfig>
  ) {
    this.retryManager = new RetryManager(retryConfig);
    this.fallbackManager = new FallbackManager();
    this.healthMonitor = new HealthMonitor();
  }

  /**
   * Execute operation with full resilience patterns
   */
  async execute<T>(
    serviceName: string,
    operation: () => Promise<T>,
    operationName?: string
  ): Promise<T> {
    const circuitBreaker = this.getOrCreateCircuitBreaker(serviceName);
    const context = operationName || `${serviceName}_operation`;

    const resilientOperation = async (): Promise<T> => {
      return await circuitBreaker.execute(operation);
    };

    if (operationName) {
      return await this.fallbackManager.executeWithFallback(
        operationName,
        () => this.retryManager.execute(resilientOperation, context)
      );
    } else {
      return await this.retryManager.execute(resilientOperation, context);
    }
  }

  /**
   * Register fallback for operation
   */
  registerFallback<T>(operationName: string, fallbackFn: () => Promise<T>): void {
    this.fallbackManager.registerFallback(operationName, fallbackFn);
  }

  /**
   * Register health check for service
   */
  registerHealthCheck(serviceName: string, healthCheckFn: () => Promise<boolean>): void {
    this.healthMonitor.registerHealthCheck(serviceName, healthCheckFn);
  }

  /**
   * Start health monitoring
   */
  startHealthMonitoring(): void {
    this.healthMonitor.startMonitoring();
  }

  /**
   * Stop health monitoring
   */
  stopHealthMonitoring(): void {
    this.healthMonitor.stopMonitoring();
  }

  /**
   * Get comprehensive status of all resilience components
   */
  getStatus() {
    const circuitBreakerStatuses = Array.from(this.circuitBreakers.entries()).map(
      ([name, cb]) => ({ [name]: cb.getStatus() })
    );

    return {
      health: Object.fromEntries(this.healthMonitor.getHealthStatus()),
      circuitBreakers: Object.assign({}, ...circuitBreakerStatuses),
    };
  }

  /**
   * Get or create circuit breaker for service
   */
  private getOrCreateCircuitBreaker(serviceName: string): CircuitBreaker {
    if (!this.circuitBreakers.has(serviceName)) {
      this.circuitBreakers.set(serviceName, new CircuitBreaker(serviceName));
    }
    return this.circuitBreakers.get(serviceName)!;
  }
}

/**
 * Error classification utility
 */
export class ErrorClassifier {
  /**
   * Classify error and determine if it should be retried
   */
  static classifyError(error: Error): { retryable: boolean; code: string } {
    // Network errors - usually retryable
    if (error.message.includes('fetch') || error.message.includes('network')) {
      return { retryable: true, code: 'NETWORK_ERROR' };
    }

    // Timeout errors - retryable
    if (error.message.includes('timeout') || error.message.includes('aborted')) {
      return { retryable: true, code: 'TIMEOUT_ERROR' };
    }

    // HTTP status-based classification
    if (error.message.includes('HTTP')) {
      const statusMatch = error.message.match(/HTTP (\d+)/);
      if (statusMatch) {
        const status = parseInt(statusMatch[1]);
        
        // Server errors (5xx) - retryable
        if (status >= 500) {
          return { retryable: true, code: `HTTP_${status}` };
        }
        
        // Rate limiting (429) - retryable with delay
        if (status === 429) {
          return { retryable: true, code: 'RATE_LIMITED' };
        }
        
        // Client errors (4xx) - not retryable
        if (status >= 400) {
          return { retryable: false, code: `HTTP_${status}` };
        }
      }
    }

    // JSON parsing errors - not retryable
    if (error.message.includes('JSON') || error.message.includes('parse')) {
      return { retryable: false, code: 'PARSE_ERROR' };
    }

    // Default: not retryable
    return { retryable: false, code: 'UNKNOWN_ERROR' };
  }

  /**
   * Create FederationError from generic error
   */
  static createFederationError(error: Error, context?: string): FederationError {
    const { retryable, code } = this.classifyError(error);
    const message = context 
      ? `${context}: ${error.message}`
      : error.message;

    return new FederationError(message, code, retryable, error);
  }
}