/**
 * Task Export Pipeline - Integration with Linear and Memchain
 * Handles task creation in Linear and knowledge storage in Memchain through federation
 */

import type {
  Task,
  Insight,
  Priority,
  TaskType,
  InsightCategory,
} from '../types.js';
import type { FederationConfig } from './client.js';

export interface LinearIssue {
  id: string;
  identifier: string;
  title: string;
  description?: string;
  priority: number;
  state: {
    name: string;
    type: string;
  };
  team: {
    id: string;
    name: string;
  };
  assignee?: {
    id: string;
    name: string;
  };
  labels: Array<{
    id: string;
    name: string;
    color: string;
  }>;
  url: string;
  createdAt: string;
}

export interface MemchainEntity {
  id: string;
  title: string;
  content: string;
  category: string;
  metadata: Record<string, any>;
  relationships: Array<{
    type: string;
    target: string;
    weight: number;
  }>;
  created_at: string;
}

export interface ExportMetrics {
  tasksExported: number;
  insightsStored: number;
  linearErrors: number;
  memchainErrors: number;
  averageExportTime: number;
}

export class TaskExporter {
  private config: FederationConfig;
  private metrics: ExportMetrics;

  constructor(config: FederationConfig) {
    this.config = config;
    this.metrics = {
      tasksExported: 0,
      insightsStored: 0,
      linearErrors: 0,
      memchainErrors: 0,
      averageExportTime: 0,
    };
  }

  /**
   * Export tasks to Linear through federation
   */
  async exportToLinear(tasks: Task[]): Promise<Task[]> {
    const startTime = Date.now();
    const exportedTasks: Task[] = [];

    try {
      // Process tasks in batches
      const batches = this.createBatches(tasks, this.config.batchSize);
      
      for (const batch of batches) {
        const batchResults = await this.processBatch(batch, 'linear');
        exportedTasks.push(...batchResults);
      }

      // Update metrics
      this.updateExportMetrics('linear', tasks.length, Date.now() - startTime);
      
      console.log(`Exported ${exportedTasks.length}/${tasks.length} tasks to Linear`);
      return exportedTasks;

    } catch (error) {
      this.handleExportError('linear', error);
      throw error;
    }
  }

  /**
   * Store insights in Memchain knowledge graph through federation
   */
  async exportToMemchain(insights: Insight[]): Promise<void> {
    const startTime = Date.now();

    try {
      // Transform insights to Memchain entities
      const entities = insights.map(insight => this.insightToMemchainEntity(insight));

      // Process entities in batches
      const batches = this.createBatches(entities, this.config.batchSize);
      
      for (const batch of batches) {
        await this.storeMemchainBatch(batch);
      }

      // Update metrics
      this.updateExportMetrics('memchain', insights.length, Date.now() - startTime);
      
      console.log(`Stored ${insights.length} insights in Memchain knowledge graph`);

    } catch (error) {
      this.handleExportError('memchain', error);
      throw error;
    }
  }

  /**
   * Bulk export both tasks and insights
   */
  async bulkExport(tasks: Task[], insights: Insight[]): Promise<{
    exportedTasks: Task[],
    storedInsights: number
  }> {
    console.log(`Starting bulk export: ${tasks.length} tasks, ${insights.length} insights`);

    try {
      // Export concurrently for better performance
      const [exportedTasks] = await Promise.allSettled([
        this.exportToLinear(tasks),
        this.exportToMemchain(insights),
      ]);

      const successfulTasks = exportedTasks.status === 'fulfilled' ? exportedTasks.value : [];
      const storedInsights = insights.length; // Assume success for now

      return {
        exportedTasks: successfulTasks,
        storedInsights,
      };

    } catch (error) {
      console.error('Bulk export failed:', error);
      throw error;
    }
  }

  /**
   * Get export performance metrics
   */
  getMetrics(): ExportMetrics {
    return { ...this.metrics };
  }

  /**
   * Private: Process task batch for Linear export
   */
  private async processBatch(tasks: Task[], target: 'linear'): Promise<Task[]> {
    if (this.config.enableMocks) {
      return this.mockLinearExport(tasks);
    }

    const results: Task[] = [];
    
    for (const task of tasks) {
      try {
        const linearIssue = await this.createLinearIssue(task);
        
        // Update task with Linear information
        const exportedTask: Task = {
          ...task,
          // Add Linear-specific metadata
        };
        
        results.push(exportedTask);
        
      } catch (error) {
        console.error(`Failed to export task ${task.id} to Linear:`, error);
        this.metrics.linearErrors++;
      }
    }

    return results;
  }

  /**
   * Private: Create Linear issue through federation API
   */
  private async createLinearIssue(task: Task): Promise<LinearIssue> {
    const payload = {
      title: task.title,
      description: task.description,
      priority: this.mapPriorityToLinear(task.priority),
      teamId: await this.getLinearTeamId(task.project),
      labelIds: await this.getOrCreateLinearLabels(task.labels),
      assigneeId: task.assignee ? await this.getLinearUserId(task.assignee) : undefined,
    };

    const response = await fetch(`${this.config.endpoint}/linear/issues`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload),
      signal: AbortSignal.timeout(this.config.timeout),
    });

    if (!response.ok) {
      throw new Error(`Linear API error: ${response.status} ${response.statusText}`);
    }

    return await response.json();
  }

  /**
   * Private: Store Memchain entity batch
   */
  private async storeMemchainBatch(entities: MemchainEntity[]): Promise<void> {
    if (this.config.enableMocks) {
      return this.mockMemchainStorage(entities);
    }

    const payload = {
      entities,
      batch_size: entities.length,
    };

    const response = await fetch(`${this.config.endpoint}/memchain/entities/batch`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload),
      signal: AbortSignal.timeout(this.config.timeout),
    });

    if (!response.ok) {
      throw new Error(`Memchain API error: ${response.status} ${response.statusText}`);
    }
  }

  /**
   * Private: Transform insight to Memchain entity
   */
  private insightToMemchainEntity(insight: Insight): MemchainEntity {
    return {
      id: insight.id,
      title: insight.title,
      content: insight.content,
      category: this.mapInsightCategoryToMemchain(insight.category),
      metadata: {
        confidence: insight.confidence,
        session_id: insight.session_context.session_id,
        blocks: insight.session_context.blocks,
        timestamp: insight.session_context.timestamp,
        related_entities: insight.related_entities,
      },
      relationships: insight.related_entities.map(entity => ({
        type: 'related_to',
        target: entity,
        weight: insight.confidence,
      })),
      created_at: new Date().toISOString(),
    };
  }

  /**
   * Private: Create batches for processing
   */
  private createBatches<T>(items: T[], batchSize: number): T[][] {
    const batches: T[][] = [];
    for (let i = 0; i < items.length; i += batchSize) {
      batches.push(items.slice(i, i + batchSize));
    }
    return batches;
  }

  /**
   * Private: Mock Linear export for testing
   */
  private async mockLinearExport(tasks: Task[]): Promise<Task[]> {
    console.log(`[MOCK] Exporting ${tasks.length} tasks to Linear`);
    
    // Simulate API delay
    await new Promise(resolve => setTimeout(resolve, Math.random() * 1000 + 500));

    for (const task of tasks) {
      console.log(`[MOCK] Created Linear issue: ${task.title} (${task.type}, ${task.priority})`);
    }

    return tasks;
  }

  /**
   * Private: Mock Memchain storage for testing
   */
  private async mockMemchainStorage(entities: MemchainEntity[]): Promise<void> {
    console.log(`[MOCK] Storing ${entities.length} entities in Memchain`);
    
    // Simulate storage delay
    await new Promise(resolve => setTimeout(resolve, Math.random() * 800 + 300));

    for (const entity of entities) {
      console.log(`[MOCK] Stored entity: ${entity.title} (${entity.category})`);
    }
  }

  /**
   * Private: Map priority to Linear priority scale
   */
  private mapPriorityToLinear(priority: Priority): number {
    const mapping = {
      [Priority.Low]: 4,
      [Priority.Medium]: 3,
      [Priority.High]: 2,
      [Priority.Urgent]: 1,
    };
    return mapping[priority] || 3;
  }

  /**
   * Private: Map insight category to Memchain category
   */
  private mapInsightCategoryToMemchain(category: InsightCategory): string {
    const mapping = {
      [InsightCategory.Technical]: 'technical_knowledge',
      [InsightCategory.Process]: 'process_knowledge',
      [InsightCategory.Learning]: 'learning_outcome',
      [InsightCategory.Pattern]: 'pattern_recognition',
      [InsightCategory.Decision]: 'decision_record',
      [InsightCategory.Risk]: 'risk_assessment',
    };
    return mapping[category] || 'general_knowledge';
  }

  /**
   * Private: Get Linear team ID (mock implementation)
   */
  private async getLinearTeamId(project?: string): Promise<string> {
    if (this.config.enableMocks) {
      return 'mock-team-id';
    }
    
    // Implementation would query Linear API for team based on project name
    const defaultTeamId = 'default-team-id';
    return defaultTeamId;
  }

  /**
   * Private: Get or create Linear labels (mock implementation)
   */
  private async getOrCreateLinearLabels(labels: string[]): Promise<string[]> {
    if (this.config.enableMocks) {
      return labels.map((_, index) => `mock-label-id-${index}`);
    }
    
    // Implementation would query/create Linear labels
    return [];
  }

  /**
   * Private: Get Linear user ID (mock implementation)
   */
  private async getLinearUserId(username: string): Promise<string> {
    if (this.config.enableMocks) {
      return 'mock-user-id';
    }
    
    // Implementation would query Linear API for user
    return 'default-user-id';
  }

  /**
   * Private: Update export metrics
   */
  private updateExportMetrics(target: 'linear' | 'memchain', count: number, duration: number): void {
    if (target === 'linear') {
      this.metrics.tasksExported += count;
    } else {
      this.metrics.insightsStored += count;
    }

    // Update average export time
    const totalExports = this.metrics.tasksExported + this.metrics.insightsStored;
    if (totalExports > 0) {
      this.metrics.averageExportTime = (
        (this.metrics.averageExportTime * (totalExports - count) + duration) / 
        totalExports
      );
    } else {
      this.metrics.averageExportTime = duration;
    }
  }

  /**
   * Private: Handle export errors
   */
  private handleExportError(target: 'linear' | 'memchain', error: unknown): void {
    const errorMessage = error instanceof Error ? error.message : String(error);
    console.error(`${target} export failed:`, errorMessage);

    if (target === 'linear') {
      this.metrics.linearErrors++;
    } else {
      this.metrics.memchainErrors++;
    }
  }
}