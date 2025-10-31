/**
 * Mock Federation Server - For testing and development
 * Simulates federation endpoints for thread submission, task export, and knowledge storage
 */

import type {
  Thread,
  Task,
  LinearIssue,
  MemchainEntity,
  AgentRole,
  AssignmentStatus,
} from '../types.js';

export interface MockServerConfig {
  port: number;
  delay: {
    min: number;
    max: number;
  };
  errorRate: number;
  enableLogging: boolean;
}

export interface MockServerStats {
  threadsSubmitted: number;
  tasksCreated: number;
  entitiesStored: number;
  requests: number;
  errors: number;
  uptime: number;
}

export class MockFederationServer {
  private config: MockServerConfig;
  private stats: MockServerStats;
  private startTime: number;
  private threads: Map<string, Thread> = new Map();
  private tasks: Map<string, Task> = new Map();
  private entities: Map<string, MemchainEntity> = new Map();

  constructor(config: Partial<MockServerConfig> = {}) {
    this.config = {
      port: config.port || 8080,
      delay: config.delay || { min: 200, max: 1500 },
      errorRate: config.errorRate || 0.05, // 5% error rate
      enableLogging: config.enableLogging ?? true,
    };

    this.stats = {
      threadsSubmitted: 0,
      tasksCreated: 0,
      entitiesStored: 0,
      requests: 0,
      errors: 0,
      uptime: 0,
    };

    this.startTime = Date.now();
  }

  /**
   * Start the mock federation server
   */
  async start(): Promise<void> {
    console.log(`🚀 Mock Federation Server starting on port ${this.config.port}`);

    const server = Bun.serve({
      port: this.config.port,
      fetch: this.handleRequest.bind(this),
    });

    if (this.config.enableLogging) {
      console.log(`📊 Mock server started at http://localhost:${this.config.port}`);
      console.log(`📈 Error rate: ${this.config.errorRate * 100}%`);
      console.log(`⏱️  Response delay: ${this.config.delay.min}-${this.config.delay.max}ms`);
    }

    return new Promise(() => {}); // Keep server running
  }

  /**
   * Handle incoming HTTP requests
   */
  private async handleRequest(request: Request): Promise<Response> {
    const url = new URL(request.url);
    const path = url.pathname;
    const method = request.method;

    this.stats.requests++;
    this.stats.uptime = Date.now() - this.startTime;

    if (this.config.enableLogging) {
      console.log(`📥 ${method} ${path}`);
    }

    // Simulate processing delay
    await this.simulateDelay();

    // Simulate random errors
    if (Math.random() < this.config.errorRate) {
      this.stats.errors++;
      return this.errorResponse('Simulated server error', 500);
    }

    try {
      switch (true) {
        case path === '/health':
          return this.handleHealth();

        case path === '/federation/threads' && method === 'POST':
          return await this.handleThreadSubmission(request);

        case path === '/federation/threads' && method === 'GET':
          return this.handleGetThreads(url);

        case path.startsWith('/federation/threads/') && method === 'GET':
          return this.handleGetThread(path);

        case path === '/linear/issues' && method === 'POST':
          return await this.handleLinearIssueCreation(request);

        case path === '/linear/teams' && method === 'GET':
          return this.handleGetLinearTeams();

        case path === '/memchain/entities/batch' && method === 'POST':
          return await this.handleMemchainBatchStorage(request);

        case path === '/memchain/entities' && method === 'GET':
          return this.handleGetMemchainEntities(url);

        case path === '/stats':
          return this.handleStats();

        default:
          return this.errorResponse('Not Found', 404);
      }
    } catch (error) {
      this.stats.errors++;
      const message = error instanceof Error ? error.message : 'Unknown error';
      return this.errorResponse(message, 500);
    }
  }

  /**
   * Health check endpoint
   */
  private handleHealth(): Response {
    return new Response(JSON.stringify({
      status: 'healthy',
      timestamp: new Date().toISOString(),
      uptime: this.stats.uptime,
    }), {
      headers: { 'Content-Type': 'application/json' },
    });
  }

  /**
   * Handle thread submission
   */
  private async handleThreadSubmission(request: Request): Response {
    const thread: Thread = await request.json();
    
    // Store thread
    this.threads.set(thread.id, thread);
    
    // Simulate agent assignment
    const primaryAgent = Object.keys(thread.assignments)[0] as AgentRole;
    if (primaryAgent && thread.assignments[primaryAgent]) {
      thread.assignments[primaryAgent].status = AssignmentStatus.InProgress;
    }

    this.stats.threadsSubmitted++;

    if (this.config.enableLogging) {
      console.log(`✅ Thread ${thread.id} submitted with ${thread.blocks.length} blocks`);
    }

    return new Response(JSON.stringify({
      thread_id: thread.id,
      status: 'submitted',
      agent_assigned: primaryAgent || 'none',
    }), {
      headers: { 'Content-Type': 'application/json' },
    });
  }

  /**
   * Get all threads
   */
  private handleGetThreads(url: URL): Response {
    const limit = parseInt(url.searchParams.get('limit') || '10');
    const threads = Array.from(this.threads.values()).slice(0, limit);

    return new Response(JSON.stringify({
      threads,
      total: this.threads.size,
      limit,
    }), {
      headers: { 'Content-Type': 'application/json' },
    });
  }

  /**
   * Get specific thread
   */
  private handleGetThread(path: string): Response {
    const threadId = path.split('/').pop();
    const thread = threadId ? this.threads.get(threadId) : null;

    if (!thread) {
      return this.errorResponse('Thread not found', 404);
    }

    return new Response(JSON.stringify(thread), {
      headers: { 'Content-Type': 'application/json' },
    });
  }

  /**
   * Handle Linear issue creation
   */
  private async handleLinearIssueCreation(request: Request): Response {
    const payload = await request.json();
    
    const issue: LinearIssue = {
      id: `mock-issue-${Date.now()}`,
      identifier: `MOCK-${this.stats.tasksCreated + 1}`,
      title: payload.title,
      description: payload.description,
      priority: payload.priority || 3,
      state: {
        name: 'Backlog',
        type: 'backlog',
      },
      team: {
        id: payload.teamId || 'mock-team-id',
        name: 'Mock Team',
      },
      assignee: payload.assigneeId ? {
        id: payload.assigneeId,
        name: 'Mock User',
      } : undefined,
      labels: (payload.labelIds || []).map((id: string, index: number) => ({
        id,
        name: `Label ${index + 1}`,
        color: '#' + Math.floor(Math.random()*16777215).toString(16),
      })),
      url: `https://mock-linear.com/issue/MOCK-${this.stats.tasksCreated + 1}`,
      createdAt: new Date().toISOString(),
    };

    this.stats.tasksCreated++;

    if (this.config.enableLogging) {
      console.log(`📝 Created Linear issue: ${issue.identifier} - ${issue.title}`);
    }

    return new Response(JSON.stringify(issue), {
      headers: { 'Content-Type': 'application/json' },
    });
  }

  /**
   * Get Linear teams (mock data)
   */
  private handleGetLinearTeams(): Response {
    const teams = [
      {
        id: 'mock-team-id',
        name: 'Mock Team',
        key: 'MOCK',
      },
      {
        id: 'dev-team-id',
        name: 'Development Team',
        key: 'DEV',
      },
    ];

    return new Response(JSON.stringify({ teams }), {
      headers: { 'Content-Type': 'application/json' },
    });
  }

  /**
   * Handle Memchain batch entity storage
   */
  private async handleMemchainBatchStorage(request: Request): Response {
    const payload = await request.json();
    const entities: MemchainEntity[] = payload.entities;

    // Store entities
    for (const entity of entities) {
      this.entities.set(entity.id, entity);
    }

    this.stats.entitiesStored += entities.length;

    if (this.config.enableLogging) {
      console.log(`🧠 Stored ${entities.length} entities in Memchain`);
    }

    return new Response(JSON.stringify({
      stored: entities.length,
      ids: entities.map(e => e.id),
    }), {
      headers: { 'Content-Type': 'application/json' },
    });
  }

  /**
   * Get Memchain entities
   */
  private handleGetMemchainEntities(url: URL): Response {
    const limit = parseInt(url.searchParams.get('limit') || '10');
    const category = url.searchParams.get('category');

    let entities = Array.from(this.entities.values());
    
    if (category) {
      entities = entities.filter(e => e.category === category);
    }

    entities = entities.slice(0, limit);

    return new Response(JSON.stringify({
      entities,
      total: this.entities.size,
      limit,
      category,
    }), {
      headers: { 'Content-Type': 'application/json' },
    });
  }

  /**
   * Get server statistics
   */
  private handleStats(): Response {
    return new Response(JSON.stringify({
      ...this.stats,
      threads: this.threads.size,
      tasks: this.tasks.size,
      entities: this.entities.size,
      uptime_formatted: this.formatUptime(this.stats.uptime),
      error_rate: this.stats.requests > 0 ? this.stats.errors / this.stats.requests : 0,
    }), {
      headers: { 'Content-Type': 'application/json' },
    });
  }

  /**
   * Create error response
   */
  private errorResponse(message: string, status: number): Response {
    return new Response(JSON.stringify({
      error: message,
      status,
      timestamp: new Date().toISOString(),
    }), {
      status,
      headers: { 'Content-Type': 'application/json' },
    });
  }

  /**
   * Simulate processing delay
   */
  private async simulateDelay(): Promise<void> {
    const delay = Math.random() * (this.config.delay.max - this.config.delay.min) + this.config.delay.min;
    await new Promise(resolve => setTimeout(resolve, delay));
  }

  /**
   * Format uptime for display
   */
  private formatUptime(ms: number): string {
    const seconds = Math.floor(ms / 1000);
    const minutes = Math.floor(seconds / 60);
    const hours = Math.floor(minutes / 60);
    
    if (hours > 0) {
      return `${hours}h ${minutes % 60}m ${seconds % 60}s`;
    } else if (minutes > 0) {
      return `${minutes}m ${seconds % 60}s`;
    } else {
      return `${seconds}s`;
    }
  }

  /**
   * Get current server statistics
   */
  getStats(): MockServerStats & { threads: number; entities: number } {
    return {
      ...this.stats,
      threads: this.threads.size,
      entities: this.entities.size,
      uptime: Date.now() - this.startTime,
    };
  }
}

/**
 * Start mock server if run directly
 */
if (import.meta.main) {
  const server = new MockFederationServer({
    port: parseInt(process.env.PORT || '8080'),
    enableLogging: true,
    errorRate: parseFloat(process.env.ERROR_RATE || '0.05'),
  });

  await server.start();
}