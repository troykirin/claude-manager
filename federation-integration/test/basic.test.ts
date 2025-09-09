/**
 * Basic tests for federation integration components
 */

import { describe, it, expect, beforeAll, afterAll } from 'bun:test';
import {
  FederationClient,
  AgentSelector,
  ThreadTransformer,
  TaskExporter,
  MockFederationServer,
  ClaudeSessionFederation,
  type Session,
  type Block,
  Role,
  ConversationIntent,
  AgentRole,
  ProgrammingLanguage,
} from '../src/index.js';

// Test data
const mockSession: Session = {
  id: 'test_session_001',
  metadata: {
    file_path: '/test/session.jsonl',
    created_at: new Date().toISOString(),
    last_modified: new Date().toISOString(),
    file_size_bytes: 1000,
    line_count: 10,
    project_context: {
      project_name: 'test-project',
      language_stack: [ProgrammingLanguage.TypeScript],
      frameworks: ['React'],
    },
  },
  blocks: [
    {
      id: 'block_001',
      sequence_number: 1,
      role: Role.User,
      timestamp: new Date().toISOString(),
      content: {
        raw_text: 'I need help debugging my React component',
        formatted_text: 'I need help debugging my React component',
        tokens: [],
        code_blocks: [],
        links: [],
        mentions: [],
        word_count: 8,
        character_count: 40,
      },
      metadata: {
        topics: ['debugging', 'react'],
        intent: ConversationIntent.Debugging,
      },
      tools: [],
      attachments: [],
      context_references: [],
    },
  ],
  insights: {
    primary_topics: [],
    conversation_flow: {
      phases: [],
      transitions: [],
      complexity_evolution: [],
      focus_shifts: 0,
    },
    learning_outcomes: [],
    productivity_metrics: {
      tasks_completed: 0,
      problems_solved: 0,
      code_quality_score: 0,
      efficiency_rating: 0,
      collaboration_effectiveness: 0,
      time_to_resolution: [],
    },
    collaboration_patterns: {
      interaction_style: 'Debugging',
      question_types: {},
      feedback_quality: 0,
      iterative_cycles: 0,
      knowledge_transfer: 0,
    },
  },
  statistics: {
    total_blocks: 1,
    user_blocks: 1,
    assistant_blocks: 0,
    tool_blocks: 0,
    total_words: 8,
    total_characters: 40,
    code_blocks: 0,
    files_referenced: 0,
    commands_executed: 0,
    errors_encountered: 0,
  },
  tool_usage: {
    tools_by_frequency: {},
    total_tool_calls: 0,
    successful_calls: 0,
    failed_calls: 0,
    average_execution_time: 0,
    tool_efficiency: {},
  },
  working_context: {
    files_mentioned: {},
    commands_run: [],
    directories_accessed: [],
    tools_used: [],
    error_patterns: [],
    solution_patterns: [],
  },
};

describe('Federation Integration', () => {
  let mockServer: MockFederationServer;

  beforeAll(async () => {
    // Start mock server for testing
    mockServer = new MockFederationServer({
      port: 8081,
      enableLogging: false,
      errorRate: 0, // No errors for tests
    });

    // Give server time to start
    setTimeout(() => mockServer.start(), 100);
    await new Promise(resolve => setTimeout(resolve, 500));
  });

  afterAll(() => {
    // Clean up if needed
  });

  describe('AgentSelector', () => {
    it('should select debugger agent for debugging content', () => {
      const selector = new AgentSelector();
      const agent = selector.selectAgent(mockSession.blocks);
      
      expect(agent).toBe(AgentRole.Debugger);
    });

    it('should provide detailed analysis', () => {
      const selector = new AgentSelector();
      const analysis = selector.getDetailedAnalysis(mockSession.blocks);
      
      expect(analysis.primary_agent).toBe(AgentRole.Debugger);
      expect(analysis.confidence).toBeGreaterThan(0);
      expect(analysis.reasoning).toBeInstanceOf(Array);
      expect(analysis.reasoning.length).toBeGreaterThan(0);
    });
  });

  describe('ThreadTransformer', () => {
    it('should transform session to thread', () => {
      const transformer = new ThreadTransformer();
      const thread = transformer.sessionToThread(mockSession);
      
      expect(thread.id).toMatch(/^thread_/);
      expect(thread.blocks).toHaveLength(1);
      expect(thread.metadata.origin).toBe('claude-session-tui');
      expect(thread.context.technologies).toContain('TypeScript');
    });

    it('should extract context correctly', () => {
      const transformer = new ThreadTransformer();
      const context = transformer.extractContext(mockSession);
      
      expect(context.technologies).toContain('TypeScript');
      expect(context.technologies).toContain('React');
      expect(context.domain).toBe('web-development');
    });
  });

  describe('FederationClient', () => {
    it('should create client with default config', () => {
      const client = new FederationClient();
      const metrics = client.getMetrics();
      
      expect(metrics.threadsSubmitted).toBe(0);
      expect(metrics.tasksExported).toBe(0);
      expect(metrics.insightsStored).toBe(0);
    });

    it('should perform health check', async () => {
      const client = new FederationClient({
        endpoint: 'http://localhost:8081/federation',
        enableMocks: true,
      });
      
      const isHealthy = await client.healthCheck();
      expect(isHealthy).toBe(true);
    });

    it('should submit thread successfully', async () => {
      const client = new FederationClient({
        endpoint: 'http://localhost:8081/federation',
        enableMocks: true,
      });
      
      const threadId = await client.submitThread(mockSession);
      expect(threadId).toMatch(/^thread_/);
      
      const metrics = client.getMetrics();
      expect(metrics.threadsSubmitted).toBe(1);
    });
  });

  describe('ClaudeSessionFederation', () => {
    it('should create federation instance', () => {
      const federation = new ClaudeSessionFederation({
        endpoint: 'http://localhost:8081/federation',
        enableMocks: true,
      });
      
      expect(federation).toBeInstanceOf(ClaudeSessionFederation);
    });

    it('should process session successfully', async () => {
      const federation = new ClaudeSessionFederation({
        endpoint: 'http://localhost:8081/federation',
        enableMocks: true,
      });
      
      const result = await federation.processSession(mockSession);
      
      expect(result.threadId).toMatch(/^thread_/);
      expect(result.agentAssigned).toBe(true);
      expect(result.processingStarted).toBe(true);
    });

    it('should check health status', async () => {
      const federation = new ClaudeSessionFederation({
        endpoint: 'http://localhost:8081/federation',
        enableMocks: true,
      });
      
      const health = await federation.checkHealth();
      
      expect(health.federation).toBe(true);
      expect(health.overall).toBe(true);
    });
  });

  describe('MockServer', () => {
    it('should provide server stats', () => {
      const stats = mockServer.getStats();
      
      expect(stats.requests).toBeGreaterThanOrEqual(0);
      expect(stats.uptime).toBeGreaterThan(0);
      expect(stats.threads).toBeGreaterThanOrEqual(0);
    });
  });
});

describe('Error Handling', () => {
  it('should handle network errors gracefully', async () => {
    const client = new FederationClient({
      endpoint: 'http://localhost:9999/federation', // Non-existent server
      enableMocks: false,
      retryAttempts: 1,
      timeout: 1000,
    });
    
    await expect(client.submitThread(mockSession)).rejects.toThrow();
  });

  it('should fallback to mock when configured', async () => {
    const client = new FederationClient({
      endpoint: 'http://localhost:9999/federation',
      enableMocks: true, // Should use mock even with bad endpoint
    });
    
    const threadId = await client.submitThread(mockSession);
    expect(threadId).toMatch(/^thread_/);
  });
});