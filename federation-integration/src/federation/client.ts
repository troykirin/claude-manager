/**
 * Federation Client - Core integration layer for multi-agent coordination
 * Handles session transformation, thread submission, and agent coordination
 */

import { v4 as uuidv4 } from 'uuid';
import type {
  Session,
  Thread,
  ThreadMetadata,
  ThreadContext,
  ThreadAssignment,
  AgentRole,
  Priority,
  ComplexityLevel,
  AssignmentStatus,
  Task,
  Marker,
  Insight,
} from '../types.js';
import { AgentSelector } from './agent_selector.js';
import { ThreadTransformer } from './transformer.js';
import { TaskExporter } from './export.js';

export interface FederationConfig {
  endpoint: string;
  timeout: number;
  retryAttempts: number;
  batchSize: number;
  enableMocks: boolean;
}

export interface FederationMetrics {
  threadsSubmitted: number;
  tasksExported: number;
  insightsStored: number;
  averageResponseTime: number;
  errorRate: number;
}

export class FederationClient {
  private config: FederationConfig;
  private agentSelector: AgentSelector;
  private transformer: ThreadTransformer;
  private exporter: TaskExporter;
  private metrics: FederationMetrics;
  
  constructor(config: Partial<FederationConfig> = {}) {
    this.config = {
      endpoint: config.endpoint || 'http://localhost:8080/federation',
      timeout: config.timeout || 30000,
      retryAttempts: config.retryAttempts || 3,
      batchSize: config.batchSize || 10,
      enableMocks: config.enableMocks || true,
    };
    
    this.agentSelector = new AgentSelector();
    this.transformer = new ThreadTransformer();
    this.exporter = new TaskExporter(this.config);
    
    this.metrics = {
      threadsSubmitted: 0,
      tasksExported: 0,
      insightsStored: 0,
      averageResponseTime: 0,
      errorRate: 0,
    };
  }

  /**
   * Submit a session to the federation for multi-agent processing
   */
  async submitThread(session: Session): Promise<string> {
    const startTime = Date.now();
    
    try {
      // Transform session to federation thread format
      const thread = this.transformer.sessionToThread(session);
      
      // Select appropriate agent based on content analysis
      const primaryAgent = this.agentSelector.selectAgent(thread.blocks);
      
      // Create thread assignment
      const assignment: ThreadAssignment = {
        agent_id: `${primaryAgent}-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
        assigned_at: new Date().toISOString(),
        status: AssignmentStatus.Pending,
        progress: 0,
        deliverables: this.generateDeliverables(thread),
      };
      
      thread.assignments[primaryAgent] = assignment;
      
      // Submit to federation (with mock fallback)
      const threadId = await this.submitToFederation(thread);
      
      // Update metrics
      this.updateMetrics('submit', startTime);
      this.metrics.threadsSubmitted++;
      
      console.log(`Thread ${threadId} submitted to federation with agent ${primaryAgent}`);
      return threadId;
      
    } catch (error) {
      this.handleError('submitThread', error);
      throw error;
    }
  }

  /**
   * Export markers as actionable tasks to Linear
   */
  async exportMarkers(markers: Marker[], session: Session): Promise<Task[]> {
    const startTime = Date.now();
    
    try {
      // Transform markers to tasks
      const tasks = markers.map(marker => this.markerToTask(marker, session));
      
      // Export to Linear through federation
      const exportedTasks = await this.exporter.exportToLinear(tasks);
      
      // Update metrics
      this.updateMetrics('export', startTime);
      this.metrics.tasksExported += exportedTasks.length;
      
      console.log(`Exported ${exportedTasks.length} tasks to Linear`);
      return exportedTasks;
      
    } catch (error) {
      this.handleError('exportMarkers', error);
      throw error;
    }
  }

  /**
   * Store insights in the knowledge graph via memchain
   */
  async storeInsights(insights: Insight[]): Promise<void> {
    const startTime = Date.now();
    
    try {
      await this.exporter.exportToMemchain(insights);
      
      // Update metrics
      this.updateMetrics('store', startTime);
      this.metrics.insightsStored += insights.length;
      
      console.log(`Stored ${insights.length} insights in knowledge graph`);
      
    } catch (error) {
      this.handleError('storeInsights', error);
      throw error;
    }
  }

  /**
   * Get federation performance metrics
   */
  getMetrics(): FederationMetrics {
    return { ...this.metrics };
  }

  /**
   * Health check for federation connectivity
   */
  async healthCheck(): Promise<boolean> {
    try {
      if (this.config.enableMocks) {
        return true; // Mock server is always healthy
      }
      
      const response = await fetch(`${this.config.endpoint}/health`, {
        method: 'GET',
        headers: { 'Content-Type': 'application/json' },
        signal: AbortSignal.timeout(5000),
      });
      
      return response.ok;
    } catch {
      return false;
    }
  }

  /**
   * Private: Submit thread to federation service
   */
  private async submitToFederation(thread: Thread): Promise<string> {
    if (this.config.enableMocks) {
      return this.mockSubmitThread(thread);
    }
    
    const response = await this.fetchWithRetry(`${this.config.endpoint}/threads`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(thread),
    });
    
    const result = await response.json();
    return result.thread_id;
  }

  /**
   * Private: Mock thread submission for testing
   */
  private async mockSubmitThread(thread: Thread): Promise<string> {
    // Simulate federation processing delay
    await new Promise(resolve => setTimeout(resolve, Math.random() * 1000 + 500));
    
    console.log(`[MOCK] Submitted thread ${thread.id} with ${thread.blocks.length} blocks`);
    console.log(`[MOCK] Primary agent: ${Object.keys(thread.assignments)[0] || 'none'}`);
    console.log(`[MOCK] Context: ${thread.context.domain}, Technologies: ${thread.context.technologies.join(', ')}`);
    
    return thread.id;
  }

  /**
   * Private: Convert marker to task format
   */
  private markerToTask(marker: Marker, session: Session): Task {
    return {
      id: uuidv4(),
      title: this.generateTaskTitle(marker),
      description: marker.content,
      type: this.mapMarkerTypeToTaskType(marker.type),
      priority: this.calculateTaskPriority(marker),
      project: session.metadata.project_context?.project_name,
      labels: [...marker.tags, `marker:${marker.type}`],
      created_from: {
        session_id: session.id,
        block_id: marker.block_id,
        marker_type: marker.type,
      },
    };
  }

  /**
   * Private: Generate deliverables for thread assignment
   */
  private generateDeliverables(thread: Thread): string[] {
    const deliverables: string[] = [];
    
    // Analyze thread content to suggest deliverables
    const hasCodeBlocks = thread.blocks.some(b => b.content.code_blocks.length > 0);
    const hasQuestions = thread.blocks.some(b => b.metadata.intent === 'Question');
    const hasImplementation = thread.blocks.some(b => b.metadata.intent === 'Implementation');
    const hasDebugging = thread.blocks.some(b => b.metadata.intent === 'Debugging');
    
    if (hasCodeBlocks) {
      deliverables.push('Code review and optimization recommendations');
    }
    
    if (hasQuestions) {
      deliverables.push('Technical answers and clarifications');
    }
    
    if (hasImplementation) {
      deliverables.push('Implementation plan and code examples');
    }
    
    if (hasDebugging) {
      deliverables.push('Debugging report and resolution steps');
    }
    
    // Default deliverable
    if (deliverables.length === 0) {
      deliverables.push('Analysis report and recommendations');
    }
    
    return deliverables;
  }

  /**
   * Private: Generate task title from marker
   */
  private generateTaskTitle(marker: Marker): string {
    const words = marker.content.split(' ').slice(0, 8);
    const title = words.join(' ');
    return title.length > 50 ? `${title.substring(0, 47)}...` : title;
  }

  /**
   * Private: Map marker type to task type
   */
  private mapMarkerTypeToTaskType(markerType: string): any {
    const mapping: Record<string, string> = {
      'action_item': 'implementation',
      'question': 'research',
      'issue': 'bug',
      'todo': 'implementation',
      'fixme': 'bug',
      'note': 'documentation',
    };
    
    return mapping[markerType] || 'implementation';
  }

  /**
   * Private: Calculate task priority from marker
   */
  private calculateTaskPriority(marker: Marker): Priority {
    // High confidence markers get higher priority
    if (marker.confidence > 0.8) {
      return Priority.High;
    } else if (marker.confidence > 0.6) {
      return Priority.Medium;
    } else {
      return Priority.Low;
    }
  }

  /**
   * Private: Fetch with retry logic
   */
  private async fetchWithRetry(url: string, options: RequestInit): Promise<Response> {
    let lastError: Error;
    
    for (let attempt = 0; attempt < this.config.retryAttempts; attempt++) {
      try {
        const controller = new AbortController();
        const timeoutId = setTimeout(() => controller.abort(), this.config.timeout);
        
        const response = await fetch(url, {
          ...options,
          signal: controller.signal,
        });
        
        clearTimeout(timeoutId);
        
        if (!response.ok) {
          throw new Error(`HTTP ${response.status}: ${response.statusText}`);
        }
        
        return response;
        
      } catch (error) {
        lastError = error as Error;
        
        if (attempt < this.config.retryAttempts - 1) {
          const delay = Math.pow(2, attempt) * 1000; // Exponential backoff
          await new Promise(resolve => setTimeout(resolve, delay));
        }
      }
    }
    
    throw lastError!;
  }

  /**
   * Private: Update performance metrics
   */
  private updateMetrics(operation: string, startTime: number): void {
    const duration = Date.now() - startTime;
    
    // Update rolling average response time
    const totalOperations = this.metrics.threadsSubmitted + this.metrics.tasksExported + this.metrics.insightsStored;
    if (totalOperations > 0) {
      this.metrics.averageResponseTime = (
        (this.metrics.averageResponseTime * totalOperations + duration) / 
        (totalOperations + 1)
      );
    } else {
      this.metrics.averageResponseTime = duration;
    }
  }

  /**
   * Private: Handle and log errors
   */
  private handleError(operation: string, error: unknown): void {
    const errorMessage = error instanceof Error ? error.message : String(error);
    console.error(`Federation ${operation} failed:`, errorMessage);
    
    // Update error rate
    const totalOperations = this.metrics.threadsSubmitted + this.metrics.tasksExported + this.metrics.insightsStored;
    if (totalOperations > 0) {
      this.metrics.errorRate = (this.metrics.errorRate * totalOperations + 1) / (totalOperations + 1);
    } else {
      this.metrics.errorRate = 1;
    }
  }
}