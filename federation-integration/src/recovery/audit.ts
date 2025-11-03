/**
 * Recovery Audit Trail - Governance Integration for Session Recovery
 * Logs diagnostic, repair, and rollback operations to Loki and Linear
 */

import type { FederationConfig } from '../federation/client.js';

// Type definitions
export interface RecoveryEvent {
  sessionId: string;
  operation: 'diagnose' | 'repair' | 'verify' | 'rollback';
  timestamp: string;
  healthScore?: number;
  corruptionPatterns?: string[];
  outcome: 'success' | 'failure' | 'partial';
  backupLocation?: string;
  durationMs?: number;
  errorMessage?: string;
}

export interface AuditConfig {
  lokiUrl: string;
  linearTeamId: string;
  enableLinearIssues: boolean;
  severityThreshold: number; // Health score below which to create Linear issue
  enableMocks?: boolean; // Enable mock mode for testing
}

export interface AuditReport {
  sessionId: string;
  events: RecoveryEvent[];
  summary: {
    totalOperations: number;
    successfulOperations: number;
    failedOperations: number;
    averageDuration: number;
    finalHealthScore?: number;
  };
  timeline: string;
  recommendations: string[];
}

export interface AuditFilters {
  sessionId?: string;
  operation?: RecoveryEvent['operation'];
  outcome?: RecoveryEvent['outcome'];
  startTime?: string;
  endTime?: string;
  minHealthScore?: number;
  maxHealthScore?: number;
}

interface LokiPayload {
  streams: Array<{
    stream: Record<string, string>;
    values: Array<[string, string]>;
  }>;
}

interface LinearIssueInput {
  title: string;
  description: string;
  priority: number;
  teamId: string;
  labels: string[];
}

interface Metrics {
  totalEvents: number;
  operationBreakdown: Record<string, number>;
  outcomeBreakdown: Record<string, number>;
  averageHealthScore: number;
  averageDuration: number;
  criticalFailures: number;
}

/**
 * Main audit trail class for recovery operations
 */
export class RecoveryAuditTrail {
  private config: AuditConfig;
  private events: RecoveryEvent[] = [];

  constructor(config: AuditConfig) {
    this.config = config;
  }

  /**
   * Log diagnostic operation
   */
  async logDiagnostic(event: RecoveryEvent): Promise<void> {
    const diagnosticEvent = { ...event, operation: 'diagnose' as const };
    await this.logOperation(diagnosticEvent);
  }

  /**
   * Log repair operation
   */
  async logRepair(event: RecoveryEvent): Promise<void> {
    const repairEvent = { ...event, operation: 'repair' as const };
    await this.logOperation(repairEvent);
  }

  /**
   * Log rollback operation
   */
  async logRollback(event: RecoveryEvent): Promise<void> {
    const rollbackEvent = { ...event, operation: 'rollback' as const };
    await this.logOperation(rollbackEvent);
  }

  /**
   * Internal operation logger
   */
  async logOperation(event: RecoveryEvent): Promise<void> {
    // Store event locally
    this.events.push(event);

    // Emit to Loki
    try {
      await this.emitToLoki(event);
    } catch (error) {
      console.error('Failed to emit to Loki:', error);
    }

    // Create Linear issue if critical
    if (this.shouldCreateLinearIssue(event)) {
      try {
        await this.createLinearIssue(event);
      } catch (error) {
        console.error('Failed to create Linear issue:', error);
      }
    }
  }

  /**
   * Emit recovery event to Loki
   */
  async emitToLoki(event: RecoveryEvent): Promise<void> {
    if (this.config.enableMocks) {
      console.log(`[MOCK] Emitting to Loki: ${event.operation} for ${event.sessionId}`);
      return;
    }

    const payload = formatLokiPayload(event);

    const response = await fetch(`${this.config.lokiUrl}/loki/api/v1/push`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(payload),
    });

    if (!response.ok) {
      throw new Error(`Loki push failed: ${response.status} ${response.statusText}`);
    }
  }

  /**
   * Create Linear issue for critical recovery events
   */
  async createLinearIssue(event: RecoveryEvent): Promise<string> {
    if (!this.config.enableLinearIssues) {
      return 'DISABLED';
    }

    const issueInput = formatLinearIssue(event, this.config.linearTeamId);

    // Mock implementation - replace with actual Linear API call
    console.log('[AUDIT] Creating Linear issue:', issueInput.title);

    // In production, use Linear GraphQL API
    // const response = await linearClient.createIssue(issueInput);
    // return response.issue.id;

    return `MOCK-ISSUE-${Date.now()}`;
  }

  /**
   * Generate audit report for a session
   */
  async generateAuditReport(sessionId: string): Promise<AuditReport> {
    const sessionEvents = this.events.filter(e => e.sessionId === sessionId);

    if (sessionEvents.length === 0) {
      throw new Error(`No events found for session: ${sessionId}`);
    }

    const summary = {
      totalOperations: sessionEvents.length,
      successfulOperations: sessionEvents.filter(e => e.outcome === 'success').length,
      failedOperations: sessionEvents.filter(e => e.outcome === 'failure').length,
      averageDuration: this.calculateAverageDuration(sessionEvents),
      finalHealthScore: sessionEvents[sessionEvents.length - 1]?.healthScore,
    };

    const timeline = this.generateTimeline(sessionEvents);
    const recommendations = this.generateRecommendations(sessionEvents);

    return {
      sessionId,
      events: sessionEvents,
      summary,
      timeline,
      recommendations,
    };
  }

  /**
   * Query recovery history with filters
   */
  async queryRecoveryHistory(filters: AuditFilters): Promise<RecoveryEvent[]> {
    let filtered = this.events;

    if (filters.sessionId) {
      filtered = filtered.filter(e => e.sessionId === filters.sessionId);
    }

    if (filters.operation) {
      filtered = filtered.filter(e => e.operation === filters.operation);
    }

    if (filters.outcome) {
      filtered = filtered.filter(e => e.outcome === filters.outcome);
    }

    if (filters.startTime) {
      filtered = filtered.filter(e => e.timestamp >= filters.startTime!);
    }

    if (filters.endTime) {
      filtered = filtered.filter(e => e.timestamp <= filters.endTime!);
    }

    if (filters.minHealthScore !== undefined) {
      filtered = filtered.filter(e => e.healthScore !== undefined && e.healthScore >= filters.minHealthScore!);
    }

    if (filters.maxHealthScore !== undefined) {
      filtered = filtered.filter(e => e.healthScore !== undefined && e.healthScore <= filters.maxHealthScore!);
    }

    return filtered;
  }

  /**
   * Calculate recovery metrics
   */
  getMetrics(): Metrics {
    return calculateRecoveryMetrics(this.events);
  }

  /**
   * Private: Determine if Linear issue should be created
   */
  private shouldCreateLinearIssue(event: RecoveryEvent): boolean {
    if (!this.config.enableLinearIssues) {
      return false;
    }

    // Create issue for failures
    if (event.outcome === 'failure') {
      return true;
    }

    // Create issue for low health scores
    if (event.healthScore !== undefined && event.healthScore < this.config.severityThreshold) {
      return true;
    }

    return false;
  }

  /**
   * Private: Calculate average duration
   */
  private calculateAverageDuration(events: RecoveryEvent[]): number {
    const durations = events.filter(e => e.durationMs !== undefined).map(e => e.durationMs!);
    if (durations.length === 0) return 0;
    return durations.reduce((sum, d) => sum + d, 0) / durations.length;
  }

  /**
   * Private: Generate event timeline
   */
  private generateTimeline(events: RecoveryEvent[]): string {
    const lines = events.map(e => {
      const time = new Date(e.timestamp).toISOString();
      const status = e.outcome === 'success' ? '✓' : e.outcome === 'failure' ? '✗' : '○';
      const health = e.healthScore ? `[${e.healthScore}%]` : '';
      return `${time} ${status} ${e.operation} ${health}`;
    });
    return lines.join('\n');
  }

  /**
   * Private: Generate recommendations
   */
  private generateRecommendations(events: RecoveryEvent[]): string[] {
    const recommendations: string[] = [];
    const failures = events.filter(e => e.outcome === 'failure');
    const lowHealth = events.filter(e => e.healthScore !== undefined && e.healthScore < 50);

    if (failures.length > 0) {
      recommendations.push(`${failures.length} operation(s) failed - review error logs`);
    }

    if (lowHealth.length > 0) {
      recommendations.push(`${lowHealth.length} event(s) with health <50% - consider manual intervention`);
    }

    const rollbacks = events.filter(e => e.operation === 'rollback');
    if (rollbacks.length > 0) {
      recommendations.push(`${rollbacks.length} rollback(s) performed - verify data integrity`);
    }

    if (recommendations.length === 0) {
      recommendations.push('All operations successful - no action needed');
    }

    return recommendations;
  }
}

/**
 * Format event for Loki ingestion
 */
function formatLokiPayload(event: RecoveryEvent): LokiPayload {
  const labels: Record<string, string> = {
    job: 'session-recovery',
    operation: event.operation,
    outcome: event.outcome,
    session_id: event.sessionId,
  };

  if (event.healthScore !== undefined) {
    labels.health_score = event.healthScore.toString();
  }

  const logLine = JSON.stringify({
    sessionId: event.sessionId,
    operation: event.operation,
    outcome: event.outcome,
    healthScore: event.healthScore,
    corruptionPatterns: event.corruptionPatterns,
    backupLocation: event.backupLocation,
    durationMs: event.durationMs,
    errorMessage: event.errorMessage,
  });

  // Loki expects timestamp in nanoseconds
  const timestampNs = (new Date(event.timestamp).getTime() * 1_000_000).toString();

  return {
    streams: [
      {
        stream: labels,
        values: [[timestampNs, logLine]],
      },
    ],
  };
}

/**
 * Format event as Linear issue
 */
function formatLinearIssue(event: RecoveryEvent, teamId: string): LinearIssueInput {
  const title = `Session Recovery ${event.outcome}: ${event.sessionId.substring(0, 8)}`;

  const description = [
    `**Operation**: ${event.operation}`,
    `**Outcome**: ${event.outcome}`,
    `**Session ID**: ${event.sessionId}`,
    event.healthScore !== undefined ? `**Health Score**: ${event.healthScore}%` : null,
    event.durationMs !== undefined ? `**Duration**: ${event.durationMs}ms` : null,
    event.backupLocation ? `**Backup**: ${event.backupLocation}` : null,
    '',
    event.corruptionPatterns && event.corruptionPatterns.length > 0
      ? `**Corruption Patterns**:\n${event.corruptionPatterns.map(p => `- ${p}`).join('\n')}`
      : null,
    '',
    event.errorMessage ? `**Error**:\n\`\`\`\n${event.errorMessage}\n\`\`\`` : null,
  ]
    .filter(Boolean)
    .join('\n');

  const priority = event.outcome === 'failure' ? 1 : event.healthScore && event.healthScore < 30 ? 2 : 3;

  return {
    title,
    description,
    priority,
    teamId,
    labels: ['session-recovery', event.operation, event.outcome],
  };
}

/**
 * Calculate recovery metrics from events
 */
function calculateRecoveryMetrics(events: RecoveryEvent[]): Metrics {
  const operationBreakdown: Record<string, number> = {};
  const outcomeBreakdown: Record<string, number> = {};
  let totalHealthScore = 0;
  let healthScoreCount = 0;
  let totalDuration = 0;
  let durationCount = 0;
  let criticalFailures = 0;

  for (const event of events) {
    // Operation breakdown
    operationBreakdown[event.operation] = (operationBreakdown[event.operation] || 0) + 1;

    // Outcome breakdown
    outcomeBreakdown[event.outcome] = (outcomeBreakdown[event.outcome] || 0) + 1;

    // Health score
    if (event.healthScore !== undefined) {
      totalHealthScore += event.healthScore;
      healthScoreCount++;
    }

    // Duration
    if (event.durationMs !== undefined) {
      totalDuration += event.durationMs;
      durationCount++;
    }

    // Critical failures
    if (event.outcome === 'failure' || (event.healthScore !== undefined && event.healthScore < 30)) {
      criticalFailures++;
    }
  }

  return {
    totalEvents: events.length,
    operationBreakdown,
    outcomeBreakdown,
    averageHealthScore: healthScoreCount > 0 ? totalHealthScore / healthScoreCount : 0,
    averageDuration: durationCount > 0 ? totalDuration / durationCount : 0,
    criticalFailures,
  };
}
