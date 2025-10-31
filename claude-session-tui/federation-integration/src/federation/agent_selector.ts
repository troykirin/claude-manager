/**
 * Agent Selection Logic - Intelligent agent assignment based on content analysis
 * Analyzes conversation patterns to determine the most appropriate agent role
 */

import type {
  Block,
} from '../types.js';

import {
  AgentRole,
  ConversationIntent,
  PhaseType,
  ProgrammingLanguage,
  TokenType,
  ComplexityLevel,
} from '../types.js';

export interface AgentSelectionConfig {
  preferredAgents: AgentRole[];
  complexityThreshold: number;
  confidenceThreshold: number;
  enableMultiAgentAssignment: boolean;
}

export interface SelectionAnalysis {
  primary_agent: AgentRole;
  secondary_agents: AgentRole[];
  confidence: number;
  reasoning: string[];
  complexity_score: number;
  domain_indicators: string[];
}

export class AgentSelector {
  private config: AgentSelectionConfig;
  
  constructor(config: Partial<AgentSelectionConfig> = {}) {
    this.config = {
      preferredAgents: config.preferredAgents || [
        AgentRole.Worker,
        AgentRole.Analyst,
        AgentRole.Architect,
        AgentRole.Debugger,
        AgentRole.Implementer,
      ],
      complexityThreshold: config.complexityThreshold || 0.7,
      confidenceThreshold: config.confidenceThreshold || 0.6,
      enableMultiAgentAssignment: config.enableMultiAgentAssignment || true,
    };
  }

  /**
   * Select the most appropriate agent for a thread based on content analysis
   */
  selectAgent(blocks: Block[]): AgentRole {
    const analysis = this.analyzeContent(blocks);
    
    console.log(`Agent selection analysis:`, {
      primary: analysis.primary_agent,
      confidence: analysis.confidence,
      complexity: analysis.complexity_score,
      reasoning: analysis.reasoning,
    });
    
    return analysis.primary_agent;
  }

  /**
   * Get detailed analysis for multi-agent coordination
   */
  getDetailedAnalysis(blocks: Block[]): SelectionAnalysis {
    return this.analyzeContent(blocks);
  }

  /**
   * Private: Comprehensive content analysis for agent selection
   */
  private analyzeContent(blocks: Block[]): SelectionAnalysis {
    const indicators = this.extractIndicators(blocks);
    const patterns = this.analyzePatterns(blocks);
    const complexity = this.calculateComplexity(blocks);
    
    // Agent scoring based on content analysis
    const scores = {
      [AgentRole.Architect]: this.scoreArchitect(indicators, patterns, complexity),
      [AgentRole.Implementer]: this.scoreImplementer(indicators, patterns, complexity),
      [AgentRole.Debugger]: this.scoreDebugger(indicators, patterns, complexity),
      [AgentRole.Analyst]: this.scoreAnalyst(indicators, patterns, complexity),
      [AgentRole.Worker]: this.scoreWorker(indicators, patterns, complexity),
      [AgentRole.Orchestrator]: this.scoreOrchestrator(indicators, patterns, complexity),
    };
    
    // Find primary agent with highest score
    const sortedAgents = Object.entries(scores)
      .sort(([, a], [, b]) => b.score - a.score)
      .filter(([agent]) => this.config.preferredAgents.includes(agent as AgentRole));
    
    const [primaryAgent, primaryScore] = sortedAgents[0];
    const confidence = primaryScore.score;
    
    // Select secondary agents if multi-agent is enabled
    const secondaryAgents: AgentRole[] = [];
    if (this.config.enableMultiAgentAssignment && confidence > this.config.confidenceThreshold) {
      secondaryAgents.push(
        ...sortedAgents
          .slice(1, 3)
          .filter(([, score]) => score.score > this.config.confidenceThreshold * 0.7)
          .map(([agent]) => agent as AgentRole)
      );
    }
    
    return {
      primary_agent: primaryAgent as AgentRole,
      secondary_agents: secondaryAgents,
      confidence,
      reasoning: primaryScore.reasons,
      complexity_score: complexity,
      domain_indicators: indicators.domains,
    };
  }

  /**
   * Private: Extract content indicators
   */
  private extractIndicators(blocks: Block[]) {
    const indicators = {
      technologies: new Set<string>(),
      domains: new Set<string>(),
      intents: new Map<ConversationIntent, number>(),
      languages: new Set<ProgrammingLanguage>(),
      keywords: new Set<string>(),
      errors: new Set<string>(),
      patterns: new Set<string>(),
    };

    for (const block of blocks) {
      // Extract programming languages
      block.content.code_blocks.forEach(cb => {
        if (cb.language) {
          indicators.languages.add(cb.language);
        }
      });

      // Count conversation intents
      if (block.metadata.intent) {
        const current = indicators.intents.get(block.metadata.intent) || 0;
        indicators.intents.set(block.metadata.intent, current + 1);
      }

      // Extract keywords from tokens
      block.content.tokens.forEach(token => {
        if (token.token_type === TokenType.Keyword || token.token_type === TokenType.Function) {
          indicators.keywords.add(token.text.toLowerCase());
        }
      });

      // Extract domain indicators from topics
      block.metadata.topics.forEach(topic => {
        indicators.domains.add(this.categorizeTopic(topic));
      });

      // Extract error patterns
      if (block.content.raw_text.toLowerCase().includes('error') ||
          block.content.raw_text.toLowerCase().includes('exception') ||
          block.content.raw_text.toLowerCase().includes('failed')) {
        indicators.errors.add('error_handling');
      }
    }

    return {
      technologies: Array.from(indicators.technologies),
      domains: Array.from(indicators.domains),
      intents: indicators.intents,
      languages: Array.from(indicators.languages),
      keywords: Array.from(indicators.keywords),
      errors: Array.from(indicators.errors),
      patterns: Array.from(indicators.patterns),
    };
  }

  /**
   * Private: Analyze conversation patterns
   */
  private analyzePatterns(blocks: Block[]) {
    return {
      hasArchitectureDiscussion: this.hasArchitectureDiscussion(blocks),
      hasDebuggingPattern: this.hasDebuggingPattern(blocks),
      hasImplementationTask: this.hasImplementationTask(blocks),
      hasAnalysisRequest: this.hasAnalysisRequest(blocks),
      hasComplexPlanning: this.hasComplexPlanning(blocks),
      hasMultiStepProcess: this.hasMultiStepProcess(blocks),
    };
  }

  /**
   * Private: Calculate content complexity
   */
  private calculateComplexity(blocks: Block[]): number {
    let complexity = 0;
    const factors = {
      codeBlockCount: 0,
      uniqueLanguages: new Set<ProgrammingLanguage>(),
      topicCount: new Set<string>(),
      toolUsage: new Set<string>(),
      averageBlockLength: 0,
    };

    let totalWords = 0;
    for (const block of blocks) {
      factors.codeBlockCount += block.content.code_blocks.length;
      totalWords += block.content.word_count;
      
      block.content.code_blocks.forEach(cb => {
        if (cb.language) factors.uniqueLanguages.add(cb.language);
      });
      
      block.metadata.topics.forEach(topic => factors.topicCount.add(topic));
      block.tools.forEach(tool => factors.toolUsage.add(tool.tool_name));
    }

    factors.averageBlockLength = blocks.length > 0 ? totalWords / blocks.length : 0;

    // Calculate complexity score
    complexity += Math.min(factors.codeBlockCount * 0.1, 0.3);
    complexity += Math.min(factors.uniqueLanguages.size * 0.1, 0.2);
    complexity += Math.min(factors.topicCount.size * 0.05, 0.2);
    complexity += Math.min(factors.toolUsage.size * 0.05, 0.15);
    complexity += Math.min(factors.averageBlockLength / 1000, 0.15);

    return Math.min(complexity, 1.0);
  }

  /**
   * Private: Score architect suitability
   */
  private scoreArchitect(indicators: any, patterns: any, complexity: number) {
    let score = 0;
    const reasons: string[] = [];

    if (patterns.hasArchitectureDiscussion) {
      score += 0.4;
      reasons.push('Architecture discussion detected');
    }

    if (patterns.hasComplexPlanning) {
      score += 0.3;
      reasons.push('Complex planning requirements');
    }

    if (indicators.domains.includes('system_design')) {
      score += 0.2;
      reasons.push('System design domain');
    }

    if (complexity > this.config.complexityThreshold) {
      score += 0.1;
      reasons.push('High complexity content');
    }

    return { score: Math.min(score, 1.0), reasons };
  }

  /**
   * Private: Score implementer suitability
   */
  private scoreImplementer(indicators: any, patterns: any, complexity: number) {
    let score = 0;
    const reasons: string[] = [];

    if (patterns.hasImplementationTask) {
      score += 0.5;
      reasons.push('Implementation tasks identified');
    }

    if (indicators.languages.length > 0) {
      score += 0.2;
      reasons.push(`Code in ${indicators.languages.length} language(s)`);
    }

    if (indicators.keywords.some((k: string) => ['function', 'class', 'method', 'implement'].includes(k))) {
      score += 0.2;
      reasons.push('Implementation keywords detected');
    }

    if (patterns.hasMultiStepProcess) {
      score += 0.1;
      reasons.push('Multi-step process detected');
    }

    return { score: Math.min(score, 1.0), reasons };
  }

  /**
   * Private: Score debugger suitability
   */
  private scoreDebugger(indicators: any, patterns: any, complexity: number) {
    let score = 0;
    const reasons: string[] = [];

    if (patterns.hasDebuggingPattern) {
      score += 0.6;
      reasons.push('Debugging patterns detected');
    }

    if (indicators.errors.length > 0) {
      score += 0.3;
      reasons.push('Error handling content');
    }

    const debugKeywords = ['debug', 'error', 'exception', 'bug', 'fix', 'troubleshoot'];
    if (indicators.keywords.some((k: string) => debugKeywords.includes(k))) {
      score += 0.1;
      reasons.push('Debugging keywords detected');
    }

    return { score: Math.min(score, 1.0), reasons };
  }

  /**
   * Private: Score analyst suitability
   */
  private scoreAnalyst(indicators: any, patterns: any, complexity: number) {
    let score = 0;
    const reasons: string[] = [];

    if (patterns.hasAnalysisRequest) {
      score += 0.4;
      reasons.push('Analysis request detected');
    }

    if (indicators.domains.includes('data_analysis') || indicators.domains.includes('research')) {
      score += 0.3;
      reasons.push('Analysis/research domain');
    }

    if (complexity > 0.5) {
      score += 0.2;
      reasons.push('Complex content requiring analysis');
    }

    const analyticsKeywords = ['analyze', 'compare', 'evaluate', 'research', 'investigate'];
    if (indicators.keywords.some((k: string) => analyticsKeywords.includes(k))) {
      score += 0.1;
      reasons.push('Analysis keywords detected');
    }

    return { score: Math.min(score, 1.0), reasons };
  }

  /**
   * Private: Score worker suitability (default/fallback)
   */
  private scoreWorker(indicators: any, patterns: any, complexity: number) {
    let score = 0.3; // Base score as fallback
    const reasons: string[] = ['Default worker assignment'];

    if (!patterns.hasArchitectureDiscussion && 
        !patterns.hasDebuggingPattern && 
        !patterns.hasAnalysisRequest) {
      score += 0.2;
      reasons.push('General task suitable for worker');
    }

    if (complexity < 0.5) {
      score += 0.1;
      reasons.push('Moderate complexity');
    }

    return { score: Math.min(score, 1.0), reasons };
  }

  /**
   * Private: Score orchestrator suitability
   */
  private scoreOrchestrator(indicators: any, patterns: any, complexity: number) {
    let score = 0;
    const reasons: string[] = [];

    if (patterns.hasMultiStepProcess && complexity > 0.8) {
      score += 0.5;
      reasons.push('Complex multi-step coordination needed');
    }

    if (indicators.domains.length > 3) {
      score += 0.3;
      reasons.push('Multi-domain coordination');
    }

    if (indicators.languages.length > 2) {
      score += 0.2;
      reasons.push('Multi-language coordination');
    }

    return { score: Math.min(score, 1.0), reasons };
  }

  /**
   * Private: Pattern detection methods
   */
  private hasArchitectureDiscussion(blocks: Block[]): boolean {
    const architectureKeywords = ['architecture', 'design', 'structure', 'pattern', 'framework', 'system'];
    return blocks.some(block => 
      architectureKeywords.some(keyword => 
        block.content.raw_text.toLowerCase().includes(keyword)
      )
    );
  }

  private hasDebuggingPattern(blocks: Block[]): boolean {
    const debugKeywords = ['debug', 'error', 'exception', 'bug', 'fix', 'troubleshoot', 'issue'];
    return blocks.some(block => 
      debugKeywords.some(keyword => 
        block.content.raw_text.toLowerCase().includes(keyword)
      ) || block.metadata.intent === ConversationIntent.Debugging
    );
  }

  private hasImplementationTask(blocks: Block[]): boolean {
    const implementKeywords = ['implement', 'create', 'build', 'develop', 'code', 'write'];
    return blocks.some(block => 
      implementKeywords.some(keyword => 
        block.content.raw_text.toLowerCase().includes(keyword)
      ) || 
      block.metadata.intent === ConversationIntent.Implementation ||
      block.content.code_blocks.length > 0
    );
  }

  private hasAnalysisRequest(blocks: Block[]): boolean {
    const analysisKeywords = ['analyze', 'compare', 'evaluate', 'review', 'assess', 'study'];
    return blocks.some(block => 
      analysisKeywords.some(keyword => 
        block.content.raw_text.toLowerCase().includes(keyword)
      ) || block.metadata.intent === ConversationIntent.CodeReview
    );
  }

  private hasComplexPlanning(blocks: Block[]): boolean {
    const planningKeywords = ['plan', 'strategy', 'roadmap', 'phase', 'milestone', 'approach'];
    return blocks.some(block => 
      planningKeywords.some(keyword => 
        block.content.raw_text.toLowerCase().includes(keyword)
      ) || block.metadata.intent === ConversationIntent.Planning
    );
  }

  private hasMultiStepProcess(blocks: Block[]): boolean {
    const stepKeywords = ['step', 'phase', 'stage', 'first', 'second', 'then', 'next', 'finally'];
    return blocks.some(block => 
      stepKeywords.filter(keyword => 
        block.content.raw_text.toLowerCase().includes(keyword)
      ).length > 2
    );
  }

  /**
   * Private: Categorize topics into domains
   */
  private categorizeTopic(topic: string): string {
    const topicLower = topic.toLowerCase();
    
    if (['architecture', 'design', 'pattern', 'system'].some(k => topicLower.includes(k))) {
      return 'system_design';
    }
    if (['data', 'analysis', 'analytics', 'research'].some(k => topicLower.includes(k))) {
      return 'data_analysis';
    }
    if (['debug', 'error', 'bug', 'issue'].some(k => topicLower.includes(k))) {
      return 'debugging';
    }
    if (['implement', 'code', 'develop', 'build'].some(k => topicLower.includes(k))) {
      return 'implementation';
    }
    if (['test', 'qa', 'quality', 'validation'].some(k => topicLower.includes(k))) {
      return 'testing';
    }
    
    return 'general';
  }
}