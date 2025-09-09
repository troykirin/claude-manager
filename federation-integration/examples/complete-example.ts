/**
 * Complete Federation Integration Example
 * Demonstrates full workflow from Claude session to federation coordination
 */

import {
  ClaudeSessionFederation,
  startMockServer,
  type Session,
  type Marker,
  type Insight,
  Role,
  ConversationIntent,
  MarkerType,
  InsightCategory,
  Priority,
  ProgrammingLanguage,
  ProjectType,
} from '../src/index.js';

/**
 * Create a realistic Claude session for demonstration
 */
function createExampleSession(): Session {
  return {
    id: 'session_example_001',
    metadata: {
      file_path: '/Users/developer/projects/web-app/claude_session.jsonl',
      created_at: new Date().toISOString(),
      last_modified: new Date().toISOString(),
      file_size_bytes: 15000,
      line_count: 120,
      claude_version: 'sonnet-4',
      client_info: {
        name: 'claude-session-tui',
        version: '1.0.0',
        platform: 'darwin',
      },
      conversation_id: 'conv_web_app_debug',
      project_context: {
        working_directory: '/Users/developer/projects/web-app',
        project_name: 'modern-web-app',
        project_type: ProjectType.WebApp,
        language_stack: [ProgrammingLanguage.TypeScript, ProgrammingLanguage.JavaScript],
        frameworks: ['React', 'Next.js', 'Tailwind CSS'],
        repository_url: 'https://github.com/dev/modern-web-app',
        git_branch: 'feature/user-authentication',
      },
    },
    blocks: [
      {
        id: 'block_001',
        sequence_number: 1,
        role: Role.User,
        timestamp: new Date().toISOString(),
        content: {
          raw_text: 'I\'m having issues with the user authentication flow in my React app. When users try to log in, they get a 401 error, but the credentials are correct.',
          formatted_text: 'I\'m having issues with the user authentication flow in my React app...',
          tokens: [
            { text: 'authentication', token_type: 'Keyword', position: 35, length: 14 },
            { text: 'React', token_type: 'Keyword', position: 72, length: 5 },
            { text: '401', token_type: 'Number', position: 120, length: 3 },
          ],
          code_blocks: [],
          links: [],
          mentions: [
            { text: 'React app', mention_type: 'Project', context: 'user authentication' },
          ],
          word_count: 28,
          character_count: 145,
        },
        metadata: {
          processing_time_ms: 250,
          confidence_score: 0.9,
          complexity_score: 0.6,
          topics: ['authentication', 'debugging', 'react', 'web-development'],
          intent: ConversationIntent.Debugging,
        },
        tools: [],
        attachments: [],
        context_references: [],
      },
      {
        id: 'block_002',
        sequence_number: 2,
        role: Role.Assistant,
        timestamp: new Date().toISOString(),
        content: {
          raw_text: 'Let\'s debug this authentication issue step by step. First, let\'s check your login endpoint implementation.',
          formatted_text: 'Let\'s debug this authentication issue step by step...',
          tokens: [
            { text: 'debug', token_type: 'Keyword', position: 6, length: 5 },
            { text: 'authentication', token_type: 'Keyword', position: 17, length: 14 },
            { text: 'endpoint', token_type: 'Keyword', position: 85, length: 8 },
          ],
          code_blocks: [
            {
              language: ProgrammingLanguage.JavaScript,
              content: `// Check your authentication endpoint
app.post('/api/auth/login', async (req, res) => {
  const { email, password } = req.body;
  
  try {
    const user = await User.findOne({ email });
    if (!user || !await user.comparePassword(password)) {
      return res.status(401).json({ error: 'Invalid credentials' });
    }
    
    const token = jwt.sign({ userId: user.id }, process.env.JWT_SECRET);
    res.json({ token, user });
  } catch (error) {
    res.status(500).json({ error: 'Server error' });
  }
});`,
              line_numbers: true,
              start_position: 95,
              end_position: 450,
            },
          ],
          links: [],
          mentions: [
            { text: 'login endpoint', mention_type: 'Function', context: 'authentication debugging' },
          ],
          word_count: 45,
          character_count: 520,
        },
        metadata: {
          processing_time_ms: 1200,
          confidence_score: 0.95,
          complexity_score: 0.8,
          topics: ['debugging', 'authentication', 'api', 'nodejs'],
          intent: ConversationIntent.Debugging,
        },
        tools: [],
        attachments: [],
        context_references: [
          {
            reference_type: 'Response',
            target_block_id: 'block_001',
            relevance_score: 0.95,
            description: 'Response to authentication debugging request',
          },
        ],
      },
    ],
    insights: {
      primary_topics: [
        {
          name: 'Authentication Debugging',
          relevance_score: 0.95,
          mentions: 3,
          subtopics: ['login', 'jwt', 'api'],
          related_tools: ['code-review', 'debugging'],
        },
      ],
      conversation_flow: {
        phases: [
          {
            phase_type: 'Debugging',
            start_block: 1,
            end_block: 2,
            duration: 300000, // 5 minutes
            primary_activity: 'Authentication troubleshooting',
          },
        ],
        transitions: [],
        complexity_evolution: [0.6, 0.8],
        focus_shifts: 1,
      },
      learning_outcomes: [],
      productivity_metrics: {
        tasks_completed: 0,
        problems_solved: 0,
        code_quality_score: 0.8,
        efficiency_rating: 0.7,
        collaboration_effectiveness: 0.9,
        time_to_resolution: [],
      },
      collaboration_patterns: {
        interaction_style: 'Debugging',
        question_types: { 'technical': 1, 'debugging': 1 },
        feedback_quality: 0.9,
        iterative_cycles: 1,
        knowledge_transfer: 0.8,
      },
    },
    statistics: {
      total_blocks: 2,
      user_blocks: 1,
      assistant_blocks: 1,
      tool_blocks: 0,
      total_words: 73,
      total_characters: 665,
      code_blocks: 1,
      files_referenced: 0,
      commands_executed: 0,
      errors_encountered: 1,
      session_duration: 300000, // 5 minutes
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
      files_mentioned: {
        '/api/auth/login': {
          path: '/api/auth/login',
          file_type: ProgrammingLanguage.JavaScript,
          mentions: 1,
          operations: ['Read'],
          last_accessed: new Date().toISOString(),
        },
      },
      commands_run: [],
      directories_accessed: ['/Users/developer/projects/web-app'],
      tools_used: [],
      error_patterns: [
        {
          error_type: 'Authentication Error',
          pattern: '401 Unauthorized',
          occurrences: 1,
          resolution_attempts: ['check credentials', 'debug endpoint'],
          resolved: false,
        },
      ],
      solution_patterns: [],
    },
  };
}

/**
 * Create example markers extracted from the session
 */
function createExampleMarkers(): Marker[] {
  return [
    {
      id: 'marker_001',
      block_id: 'block_001',
      type: MarkerType.Issue,
      content: 'Users getting 401 error during login with correct credentials',
      context: 'Authentication flow debugging in React app',
      confidence: 0.9,
      tags: ['authentication', 'bug', 'urgent'],
    },
    {
      id: 'marker_002',
      block_id: 'block_002',
      type: MarkerType.ActionItem,
      content: 'Check login endpoint implementation for credential validation logic',
      context: 'Debugging authentication 401 errors',
      confidence: 0.85,
      tags: ['debugging', 'api', 'backend'],
    },
    {
      id: 'marker_003',
      block_id: 'block_002',
      type: MarkerType.TODO,
      content: 'Verify JWT_SECRET environment variable is properly set',
      context: 'Authentication endpoint configuration',
      confidence: 0.75,
      tags: ['configuration', 'security', 'env'],
    },
  ];
}

/**
 * Create example insights generated from the session
 */
function createExampleInsights(): Insight[] {
  return [
    {
      id: 'insight_001',
      title: 'Authentication Pattern: JWT + Password Comparison',
      content: 'The session reveals a standard authentication pattern using JWT tokens with bcrypt password comparison. The user is experiencing 401 errors despite correct credentials, suggesting potential issues with password hashing, JWT secret configuration, or database connectivity.',
      category: InsightCategory.Pattern,
      confidence: 0.88,
      related_entities: ['authentication', 'jwt', 'password-hashing', 'api-security'],
      session_context: {
        session_id: 'session_example_001',
        blocks: ['block_001', 'block_002'],
        timestamp: new Date().toISOString(),
      },
    },
    {
      id: 'insight_002',
      title: 'Debugging Approach: Systematic Authentication Troubleshooting',
      content: 'The assistant employed a systematic debugging approach, starting with endpoint verification and providing concrete code examples. This demonstrates effective technical problem-solving methodology for authentication issues.',
      category: InsightCategory.Process,
      confidence: 0.82,
      related_entities: ['debugging', 'problem-solving', 'authentication'],
      session_context: {
        session_id: 'session_example_001',
        blocks: ['block_002'],
        timestamp: new Date().toISOString(),
      },
    },
    {
      id: 'insight_003',
      title: 'Technology Stack: React + Node.js Authentication',
      content: 'Session involves React frontend with Node.js backend authentication. Technologies include JWT for tokens, bcrypt for password hashing, and Express.js for API endpoints. This is a common modern web application stack.',
      category: InsightCategory.Technical,
      confidence: 0.91,
      related_entities: ['react', 'nodejs', 'jwt', 'express', 'web-development'],
      session_context: {
        session_id: 'session_example_001',
        blocks: ['block_001', 'block_002'],
        timestamp: new Date().toISOString(),
      },
    },
  ];
}

/**
 * Run the complete federation integration example
 */
async function runCompleteExample(): Promise<void> {
  console.log('🚀 Starting Federation Integration Example\n');

  // Step 1: Start mock federation server
  console.log('📡 Starting mock federation server...');
  const mockServerPromise = startMockServer(8080);
  
  // Wait a moment for server to start
  await new Promise(resolve => setTimeout(resolve, 2000));

  try {
    // Step 2: Create federation client
    console.log('🔧 Initializing federation client...');
    const federation = new ClaudeSessionFederation({
      endpoint: 'http://localhost:8080/federation',
      enableMocks: true,
      timeout: 10000,
      retryAttempts: 2,
    });

    // Step 3: Start health monitoring
    console.log('❤️  Starting health monitoring...');
    federation.startHealthMonitoring();

    // Step 4: Create example data
    const session = createExampleSession();
    const markers = createExampleMarkers();
    const insights = createExampleInsights();

    console.log(`📊 Created example data:
    - Session: ${session.blocks.length} blocks, ${session.statistics.total_words} words
    - Markers: ${markers.length} action items
    - Insights: ${insights.length} knowledge insights\n`);

    // Step 5: Check federation health
    console.log('🏥 Checking federation health...');
    const health = await federation.checkHealth();
    console.log(`Health Status:
    - Federation: ${health.federation ? '✅' : '❌'}
    - Linear: ${health.linear ? '✅' : '❌'}
    - Memchain: ${health.memchain ? '✅' : '❌'}
    - Overall: ${health.overall ? '✅' : '❌'}\n`);

    // Step 6: Process session through federation
    console.log('🧠 Processing session through federation...');
    const sessionResult = await federation.processSession(session);
    console.log(`Session Processing Result:
    - Thread ID: ${sessionResult.threadId}
    - Agent Assigned: ${sessionResult.agentAssigned ? '✅' : '❌'}
    - Processing Started: ${sessionResult.processingStarted ? '✅' : '❌'}\n`);

    // Step 7: Export markers as Linear tasks
    console.log('📝 Exporting markers to Linear...');
    const exportResult = await federation.exportActionItems(markers, session);
    console.log(`Marker Export Result:
    - Tasks Created: ${exportResult.exported}
    - Failed Exports: ${exportResult.failed}
    - Success Rate: ${((exportResult.exported / markers.length) * 100).toFixed(1)}%\n`);

    // Step 8: Store insights in knowledge graph
    console.log('🔍 Storing insights in knowledge graph...');
    const storeResult = await federation.storeKnowledge(insights);
    console.log(`Knowledge Storage Result:
    - Insights Stored: ${storeResult.stored}
    - Failed Storage: ${storeResult.failed}
    - Success Rate: ${((storeResult.stored / insights.length) * 100).toFixed(1)}%\n`);

    // Step 9: Demonstrate complete cognitive externalization
    console.log('🚀 Running complete cognitive externalization...');
    const cognitiveResult = await federation.cognitiveExternalization(
      session,
      markers,
      insights
    );

    console.log(`Cognitive Externalization Complete:
    - Thread ID: ${cognitiveResult.threadId}
    - Tasks Created: ${cognitiveResult.tasksCreated}
    - Insights Stored: ${cognitiveResult.insightsStored}
    - Errors: ${cognitiveResult.errors.length}${cognitiveResult.errors.length > 0 ? '\n      ' + cognitiveResult.errors.join('\n      ') : ''}\n`);

    // Step 10: Display comprehensive metrics
    console.log('📈 Federation metrics and status:');
    const status = federation.getStatus();
    console.log(`Client Metrics:
    - Threads Submitted: ${status.client.threadsSubmitted}
    - Tasks Exported: ${status.client.tasksExported}
    - Insights Stored: ${status.client.insightsStored}
    - Average Response Time: ${status.client.averageResponseTime.toFixed(0)}ms
    - Error Rate: ${(status.client.errorRate * 100).toFixed(1)}%\n`);

    // Step 11: Stop health monitoring
    federation.stopHealthMonitoring();

    console.log('✅ Federation integration example completed successfully!');
    console.log('\n🎯 Key Achievements:');
    console.log('   • Session successfully transformed and submitted to federation');
    console.log('   • Intelligent agent selection based on content analysis');
    console.log('   • Markers exported as actionable Linear tasks');
    console.log('   • Insights preserved in knowledge graph');
    console.log('   • Comprehensive error handling and resilience patterns');
    console.log('   • Health monitoring and performance metrics');

  } catch (error) {
    console.error('❌ Federation example failed:', error);
    process.exit(1);
  }
}

// Run example if called directly
if (import.meta.main) {
  await runCompleteExample();
  process.exit(0);
}