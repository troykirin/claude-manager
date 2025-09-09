/**
 * Claude Session Federation Integration
 * Main entry point for federation client and multi-agent coordination
 */

// Core exports
export { FederationClient, type FederationConfig, type FederationMetrics } from './federation/client.js';
export { AgentSelector, type AgentSelectionConfig, type SelectionAnalysis } from './federation/agent_selector.js';
export { ThreadTransformer, type TransformationConfig, type ContextExtractionResult } from './federation/transformer.js';
export { TaskExporter, type LinearIssue, type MemchainEntity, type ExportMetrics } from './federation/export.js';

// Error handling and resilience
export {
  FederationError,
  RetryExhaustedError,
  CircuitOpenError,
  RetryManager,
  CircuitBreaker,
  FallbackManager,
  HealthMonitor,
  ResilienceManager,
  ErrorClassifier,
  type RetryConfig,
  type CircuitBreakerConfig,
  CircuitBreakerState,
} from './error-handling.js';

// Mock server for testing
export { MockFederationServer, type MockServerConfig, type MockServerStats } from './mock/server.js';

// Type definitions
export * from './types.js';

import { FederationClient, type FederationConfig } from './federation/client.js';
import { ResilienceManager } from './error-handling.js';
import type { Session, Marker, Insight, Task } from './types.js';

/**
 * High-level federation integration facade
 * Provides simplified API for common federation operations
 */
export class ClaudeSessionFederation {
  private client: FederationClient;
  private resilience: ResilienceManager;

  constructor(config: Partial<FederationConfig> = {}) {
    this.client = new FederationClient(config);
    this.resilience = new ResilienceManager();

    // Setup health monitoring for federation services
    this.setupHealthMonitoring();
    
    // Setup fallback strategies
    this.setupFallbacks();
  }

  /**
   * Process a Claude session through the federation
   */
  async processSession(session: Session): Promise<{
    threadId: string;
    agentAssigned: boolean;
    processingStarted: boolean;
  }> {
    try {
      const threadId = await this.resilience.execute(
        'federation',
        () => this.client.submitThread(session),
        'submit_thread'
      );

      return {
        threadId,
        agentAssigned: true,
        processingStarted: true,
      };

    } catch (error) {
      console.error('Failed to process session through federation:', error);
      throw error;
    }
  }

  /**
   * Export markers as actionable tasks
   */
  async exportActionItems(markers: Marker[], session: Session): Promise<{
    tasks: Task[];
    exported: number;
    failed: number;
  }> {
    try {
      const tasks = await this.resilience.execute(
        'linear',
        () => this.client.exportMarkers(markers, session),
        'export_markers'
      );

      return {
        tasks,
        exported: tasks.length,
        failed: markers.length - tasks.length,
      };

    } catch (error) {
      console.error('Failed to export action items:', error);
      return {
        tasks: [],
        exported: 0,
        failed: markers.length,
      };
    }
  }

  /**
   * Store insights in knowledge graph
   */
  async storeKnowledge(insights: Insight[]): Promise<{
    stored: number;
    failed: number;
  }> {
    try {
      await this.resilience.execute(
        'memchain',
        () => this.client.storeInsights(insights),
        'store_insights'
      );

      return {
        stored: insights.length,
        failed: 0,
      };

    } catch (error) {
      console.error('Failed to store knowledge:', error);
      return {
        stored: 0,
        failed: insights.length,
      };
    }
  }

  /**
   * Process session with full cognitive externalization pipeline
   */
  async cognitiveExternalization(
    session: Session,
    markers: Marker[],
    insights: Insight[]
  ): Promise<{
    threadId?: string;
    tasksCreated: number;
    insightsStored: number;
    errors: string[];
  }> {
    const result = {
      threadId: undefined as string | undefined,
      tasksCreated: 0,
      insightsStored: 0,
      errors: [] as string[],
    };

    // Process session through federation
    try {
      const sessionResult = await this.processSession(session);
      result.threadId = sessionResult.threadId;
    } catch (error) {
      result.errors.push(`Session processing failed: ${error instanceof Error ? error.message : String(error)}`);
    }

    // Export action items
    try {
      const exportResult = await this.exportActionItems(markers, session);
      result.tasksCreated = exportResult.exported;
      if (exportResult.failed > 0) {
        result.errors.push(`Failed to export ${exportResult.failed} markers`);
      }
    } catch (error) {
      result.errors.push(`Marker export failed: ${error instanceof Error ? error.message : String(error)}`);
    }

    // Store insights
    try {
      const storeResult = await this.storeKnowledge(insights);
      result.insightsStored = storeResult.stored;
      if (storeResult.failed > 0) {
        result.errors.push(`Failed to store ${storeResult.failed} insights`);
      }
    } catch (error) {
      result.errors.push(`Knowledge storage failed: ${error instanceof Error ? error.message : String(error)}`);
    }

    return result;
  }

  /**
   * Get comprehensive status and metrics
   */
  getStatus() {
    return {
      client: this.client.getMetrics(),
      resilience: this.resilience.getStatus(),
      health: this.checkHealth(),
    };
  }

  /**
   * Perform health check on all systems
   */
  async checkHealth(): Promise<{
    federation: boolean;
    linear: boolean;
    memchain: boolean;
    overall: boolean;
  }> {
    const health = {
      federation: await this.client.healthCheck(),
      linear: true, // Would implement actual Linear health check
      memchain: true, // Would implement actual Memchain health check
      overall: true,
    };

    health.overall = health.federation && health.linear && health.memchain;
    return health;
  }

  /**
   * Start background health monitoring
   */
  startHealthMonitoring(): void {
    this.resilience.startHealthMonitoring();
    console.log('Federation health monitoring started');
  }

  /**
   * Stop background health monitoring
   */
  stopHealthMonitoring(): void {
    this.resilience.stopHealthMonitoring();
    console.log('Federation health monitoring stopped');
  }

  /**
   * Private: Setup health monitoring for federation services
   */
  private setupHealthMonitoring(): void {
    // Federation health check
    this.resilience.registerHealthCheck('federation', async () => {
      return await this.client.healthCheck();
    });

    // Linear health check (mock implementation)
    this.resilience.registerHealthCheck('linear', async () => {
      try {
        // Would implement actual Linear API health check
        return true;
      } catch {
        return false;
      }
    });

    // Memchain health check (mock implementation)
    this.resilience.registerHealthCheck('memchain', async () => {
      try {
        // Would implement actual Memchain health check
        return true;
      } catch {
        return false;
      }
    });
  }

  /**
   * Private: Setup fallback strategies
   */
  private setupFallbacks(): void {
    // Fallback for thread submission - store locally
    this.resilience.registerFallback('submit_thread', async () => {
      console.warn('Using fallback: storing thread locally instead of federation');
      // Would implement local storage fallback
      return `local_thread_${Date.now()}`;
    });

    // Fallback for marker export - log to console
    this.resilience.registerFallback('export_markers', async () => {
      console.warn('Using fallback: logging markers instead of Linear export');
      // Would implement local logging fallback
      return [];
    });

    // Fallback for insight storage - local file
    this.resilience.registerFallback('store_insights', async () => {
      console.warn('Using fallback: storing insights locally instead of Memchain');
      // Would implement local file storage fallback
    });
  }
}

/**
 * Factory function for easy setup
 */
export function createFederationClient(config?: Partial<FederationConfig>): ClaudeSessionFederation {
  return new ClaudeSessionFederation(config);
}

/**
 * Utility function to start mock server for development
 */
export async function startMockServer(port: number = 8080): Promise<void> {
  const { MockFederationServer } = await import('./mock/server.js');
  const server = new MockFederationServer({ port, enableLogging: true });
  await server.start();
}

// Default export for convenience
export default ClaudeSessionFederation;