/**
 * Thread Transformation - Converts Claude sessions to federation thread format
 * Handles context extraction, priority assessment, and thread metadata generation
 */

import { v4 as uuidv4 } from 'uuid';
import type {
  Session,
  Thread,
  ThreadMetadata,
  ThreadContext,
  Block,
  Priority,
  ComplexityLevel,
  ConversationIntent,
  ProgrammingLanguage,
  AgentRole,
} from '../types.js';

export interface TransformationConfig {
  includeToolData: boolean;
  includeAttachments: boolean;
  maxBlocksPerThread: number;
  contextExtractionDepth: number;
}

export interface ContextExtractionResult {
  technologies: string[];
  domain: string;
  objectives: string[];
  constraints: string[];
  working_directory?: string;
}

export class ThreadTransformer {
  private config: TransformationConfig;

  constructor(config: Partial<TransformationConfig> = {}) {
    this.config = {
      includeToolData: config.includeToolData ?? true,
      includeAttachments: config.includeAttachments ?? true,
      maxBlocksPerThread: config.maxBlocksPerThread || 1000,
      contextExtractionDepth: config.contextExtractionDepth || 3,
    };
  }

  /**
   * Transform a Claude session into a federation thread
   */
  sessionToThread(session: Session): Thread {
    // Extract thread context from session
    const context = this.extractContext(session);
    
    // Generate thread metadata
    const metadata = this.generateThreadMetadata(session, context);
    
    // Filter and prepare blocks
    const blocks = this.prepareBlocks(session.blocks);
    
    // Create thread with empty assignments (filled by federation client)
    const thread: Thread = {
      id: this.generateThreadId(),
      blocks,
      metadata,
      context,
      assignments: {},
    };

    console.log(`Transformed session ${session.id} to thread ${thread.id}`, {
      blocks: thread.blocks.length,
      domain: context.domain,
      technologies: context.technologies,
      priority: metadata.priority,
    });

    return thread;
  }

  /**
   * Extract context information from session for federation routing
   */
  extractContext(session: Session): ThreadContext {
    const context: ContextExtractionResult = {
      technologies: [],
      domain: 'general',
      objectives: [],
      constraints: [],
      working_directory: session.metadata.project_context?.working_directory,
    };

    // Extract technologies from programming languages and frameworks
    if (session.metadata.project_context) {
      const projectContext = session.metadata.project_context;
      
      // Add programming languages
      context.technologies.push(
        ...projectContext.language_stack.map(lang => lang.toString())
      );
      
      // Add frameworks
      context.technologies.push(...projectContext.frameworks);
    }

    // Extract technologies from code blocks
    const codeLanguages = new Set<string>();
    for (const block of session.blocks) {
      for (const codeBlock of block.content.code_blocks) {
        if (codeBlock.language) {
          codeLanguages.add(codeBlock.language.toString());
        }
      }
    }
    context.technologies.push(...Array.from(codeLanguages));

    // Remove duplicates and clean up
    context.technologies = [...new Set(context.technologies.filter(Boolean))];

    // Determine primary domain from session content
    context.domain = this.determineDomain(session);

    // Extract objectives from conversation intents
    context.objectives = this.extractObjectives(session);

    // Extract constraints from session metadata and content
    context.constraints = this.extractConstraints(session);

    return context;
  }

  /**
   * Generate thread metadata with priority and complexity assessment
   */
  private generateThreadMetadata(session: Session, context: ThreadContext): ThreadMetadata {
    const priority = this.assessPriority(session);
    const complexity = this.assessComplexity(session);

    return {
      origin: 'claude-session-tui',
      project: session.metadata.project_context?.project_name || 'unnamed-project',
      timestamp: session.metadata.created_at,
      priority,
      estimated_complexity: complexity,
    };
  }

  /**
   * Prepare blocks for federation thread (filter, clean, optimize)
   */
  private prepareBlocks(blocks: Block[]): Block[] {
    let preparedBlocks = blocks;

    // Limit number of blocks if necessary
    if (preparedBlocks.length > this.config.maxBlocksPerThread) {
      console.warn(`Truncating thread: ${preparedBlocks.length} blocks -> ${this.config.maxBlocksPerThread}`);
      preparedBlocks = preparedBlocks.slice(0, this.config.maxBlocksPerThread);
    }

    // Filter out tool data if not included
    if (!this.config.includeToolData) {
      preparedBlocks = preparedBlocks.map(block => ({
        ...block,
        tools: [],
      }));
    }

    // Filter out attachments if not included
    if (!this.config.includeAttachments) {
      preparedBlocks = preparedBlocks.map(block => ({
        ...block,
        attachments: [],
      }));
    }

    return preparedBlocks;
  }

  /**
   * Generate unique thread ID
   */
  private generateThreadId(): string {
    const timestamp = Date.now().toString(36);
    const random = Math.random().toString(36).substr(2, 9);
    return `thread_${timestamp}_${random}`;
  }

  /**
   * Determine primary domain from session analysis
   */
  private determineDomain(session: Session): string {
    const domainScores: Record<string, number> = {
      'web-development': 0,
      'mobile-development': 0,
      'data-science': 0,
      'machine-learning': 0,
      'systems-programming': 0,
      'devops': 0,
      'database': 0,
      'security': 0,
      'testing': 0,
      'documentation': 0,
      'general': 0,
    };

    // Score based on project type
    if (session.metadata.project_context?.project_type) {
      const projectType = session.metadata.project_context.project_type;
      switch (projectType) {
        case 'WebApp':
          domainScores['web-development'] += 3;
          break;
        case 'MobileApp':
          domainScores['mobile-development'] += 3;
          break;
        case 'DataScience':
          domainScores['data-science'] += 3;
          break;
        case 'MachineLearning':
          domainScores['machine-learning'] += 3;
          break;
        case 'CLI':
          domainScores['systems-programming'] += 2;
          break;
        case 'Documentation':
          domainScores['documentation'] += 3;
          break;
        default:
          domainScores['general'] += 1;
      }
    }

    // Score based on programming languages
    const languages = session.metadata.project_context?.language_stack || [];
    for (const lang of languages) {
      switch (lang) {
        case ProgrammingLanguage.JavaScript:
        case ProgrammingLanguage.TypeScript:
        case ProgrammingLanguage.HTML:
        case ProgrammingLanguage.CSS:
          domainScores['web-development'] += 2;
          break;
        case ProgrammingLanguage.Swift:
        case ProgrammingLanguage.Kotlin:
        case ProgrammingLanguage.Dart:
          domainScores['mobile-development'] += 2;
          break;
        case ProgrammingLanguage.Python:
          domainScores['data-science'] += 1;
          domainScores['machine-learning'] += 1;
          break;
        case ProgrammingLanguage.Rust:
        case ProgrammingLanguage.Go:
        case ProgrammingLanguage.C:
        case ProgrammingLanguage.Cpp:
          domainScores['systems-programming'] += 2;
          break;
        case ProgrammingLanguage.SQL:
          domainScores['database'] += 2;
          break;
        case ProgrammingLanguage.Shell:
          domainScores['devops'] += 1;
          break;
      }
    }

    // Score based on frameworks and technologies
    const frameworks = session.metadata.project_context?.frameworks || [];
    for (const framework of frameworks) {
      const frameworkLower = framework.toLowerCase();
      if (['react', 'vue', 'angular', 'express', 'nextjs'].some(f => frameworkLower.includes(f))) {
        domainScores['web-development'] += 2;
      }
      if (['flutter', 'react-native', 'ionic'].some(f => frameworkLower.includes(f))) {
        domainScores['mobile-development'] += 2;
      }
      if (['tensorflow', 'pytorch', 'scikit-learn'].some(f => frameworkLower.includes(f))) {
        domainScores['machine-learning'] += 2;
      }
      if (['docker', 'kubernetes', 'terraform'].some(f => frameworkLower.includes(f))) {
        domainScores['devops'] += 2;
      }
    }

    // Score based on content keywords
    const allContent = session.blocks
      .map(block => block.content.raw_text.toLowerCase())
      .join(' ');

    const keywordMappings: Record<string, string[]> = {
      'web-development': ['frontend', 'backend', 'api', 'server', 'client', 'web', 'http', 'rest'],
      'mobile-development': ['mobile', 'app', 'ios', 'android', 'native', 'hybrid'],
      'data-science': ['data', 'analysis', 'visualization', 'dataset', 'analytics'],
      'machine-learning': ['ml', 'ai', 'model', 'training', 'neural', 'algorithm'],
      'systems-programming': ['performance', 'memory', 'concurrency', 'system', 'low-level'],
      'devops': ['deploy', 'pipeline', 'ci/cd', 'infrastructure', 'monitoring'],
      'database': ['database', 'query', 'table', 'schema', 'migration'],
      'security': ['security', 'authentication', 'encryption', 'vulnerability'],
      'testing': ['test', 'testing', 'unit', 'integration', 'qa'],
      'documentation': ['documentation', 'readme', 'guide', 'manual', 'docs'],
    };

    for (const [domain, keywords] of Object.entries(keywordMappings)) {
      for (const keyword of keywords) {
        const count = (allContent.match(new RegExp(keyword, 'g')) || []).length;
        domainScores[domain] += count * 0.5;
      }
    }

    // Find domain with highest score
    const topDomain = Object.entries(domainScores)
      .sort(([, a], [, b]) => b - a)[0];

    return topDomain[1] > 0 ? topDomain[0] : 'general';
  }

  /**
   * Extract objectives from conversation intents
   */
  private extractObjectives(session: Session): string[] {
    const objectives: string[] = [];
    const intentCounts = new Map<ConversationIntent, number>();

    // Count intents across blocks
    for (const block of session.blocks) {
      if (block.metadata.intent) {
        const current = intentCounts.get(block.metadata.intent) || 0;
        intentCounts.set(block.metadata.intent, current + 1);
      }
    }

    // Convert top intents to objectives
    const sortedIntents = Array.from(intentCounts.entries())
      .sort(([, a], [, b]) => b - a)
      .slice(0, 3);

    for (const [intent, count] of sortedIntents) {
      switch (intent) {
        case ConversationIntent.Implementation:
          objectives.push('Implement solution or feature');
          break;
        case ConversationIntent.Debugging:
          objectives.push('Debug and fix issues');
          break;
        case ConversationIntent.CodeReview:
          objectives.push('Review and improve code quality');
          break;
        case ConversationIntent.Planning:
          objectives.push('Plan and design approach');
          break;
        case ConversationIntent.Learning:
          objectives.push('Learn and understand concepts');
          break;
        case ConversationIntent.Documentation:
          objectives.push('Create or improve documentation');
          break;
        case ConversationIntent.Troubleshooting:
          objectives.push('Troubleshoot problems');
          break;
        default:
          if (count > 2) {
            objectives.push(`Address ${intent.toLowerCase()} requirements`);
          }
      }
    }

    return objectives.length > 0 ? objectives : ['General assistance and guidance'];
  }

  /**
   * Extract constraints from session context
   */
  private extractConstraints(session: Session): string[] {
    const constraints: string[] = [];

    // Time constraints from session duration
    const duration = session.statistics.session_duration;
    if (duration && duration < 1800000) { // Less than 30 minutes
      constraints.push('Time-sensitive task');
    }

    // Technology constraints from project context
    if (session.metadata.project_context?.language_stack.length === 1) {
      const language = session.metadata.project_context.language_stack[0];
      constraints.push(`Must use ${language}`);
    }

    // Complexity constraints
    const complexity = this.assessComplexity(session);
    if (complexity === ComplexityLevel.Expert) {
      constraints.push('High complexity - expert knowledge required');
    } else if (complexity === ComplexityLevel.Beginner) {
      constraints.push('Keep solution simple and beginner-friendly');
    }

    // Error constraints from session content
    const hasErrors = session.blocks.some(block => 
      block.content.raw_text.toLowerCase().includes('error') ||
      block.content.raw_text.toLowerCase().includes('exception')
    );
    if (hasErrors) {
      constraints.push('Must address existing errors');
    }

    return constraints;
  }

  /**
   * Assess thread priority based on session characteristics
   */
  private assessPriority(session: Session): Priority {
    let score = 0;

    // High priority indicators
    const urgentKeywords = ['urgent', 'asap', 'immediately', 'critical', 'emergency', 'broken', 'down'];
    const hasUrgentKeywords = session.blocks.some(block =>
      urgentKeywords.some(keyword =>
        block.content.raw_text.toLowerCase().includes(keyword)
      )
    );
    if (hasUrgentKeywords) score += 3;

    // Error indicators
    const hasErrors = session.blocks.some(block =>
      block.content.raw_text.toLowerCase().includes('error') ||
      block.content.raw_text.toLowerCase().includes('exception') ||
      block.content.raw_text.toLowerCase().includes('failed')
    );
    if (hasErrors) score += 2;

    // Session frequency (multiple blocks suggest ongoing work)
    if (session.blocks.length > 20) score += 1;

    // Tool usage frequency (active development)
    if (session.tool_usage.total_tool_calls > 10) score += 1;

    // Recent activity
    const now = new Date();
    const lastModified = new Date(session.metadata.last_modified);
    const hoursSinceModified = (now.getTime() - lastModified.getTime()) / (1000 * 60 * 60);
    if (hoursSinceModified < 2) score += 1;

    // Convert score to priority
    if (score >= 4) return Priority.Urgent;
    if (score >= 2) return Priority.High;
    if (score >= 1) return Priority.Medium;
    return Priority.Low;
  }

  /**
   * Assess thread complexity level
   */
  private assessComplexity(session: Session): ComplexityLevel {
    let score = 0;

    // Code complexity
    const totalCodeBlocks = session.blocks.reduce((sum, block) => 
      sum + block.content.code_blocks.length, 0
    );
    if (totalCodeBlocks > 20) score += 3;
    else if (totalCodeBlocks > 10) score += 2;
    else if (totalCodeBlocks > 5) score += 1;

    // Language diversity
    const languages = new Set<string>();
    for (const block of session.blocks) {
      for (const codeBlock of block.content.code_blocks) {
        if (codeBlock.language) {
          languages.add(codeBlock.language.toString());
        }
      }
    }
    if (languages.size > 3) score += 2;
    else if (languages.size > 1) score += 1;

    // Tool usage complexity
    if (session.tool_usage.total_tool_calls > 50) score += 2;
    else if (session.tool_usage.total_tool_calls > 20) score += 1;

    // Topic diversity
    const allTopics = new Set<string>();
    for (const block of session.blocks) {
      block.metadata.topics.forEach(topic => allTopics.add(topic));
    }
    if (allTopics.size > 15) score += 2;
    else if (allTopics.size > 10) score += 1;

    // Session length
    if (session.blocks.length > 50) score += 2;
    else if (session.blocks.length > 25) score += 1;

    // Convert score to complexity level
    if (score >= 8) return ComplexityLevel.Expert;
    if (score >= 5) return ComplexityLevel.Advanced;
    if (score >= 2) return ComplexityLevel.Intermediate;
    return ComplexityLevel.Beginner;
  }
}