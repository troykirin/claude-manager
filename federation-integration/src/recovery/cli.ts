#!/usr/bin/env node
/**
 * CLI Entry Point for Recovery Audit Trail
 * Provides bash integration for recovery operations
 */

import { Command } from 'commander';
import { RecoveryAuditTrail, type RecoveryEvent, type AuditConfig } from './audit.js';
import { readFileSync, existsSync } from 'fs';
import { join } from 'path';

const program = new Command();

/**
 * Load configuration from file or environment
 */
function loadConfig(): AuditConfig {
  // Try to load from config file
  const configPath = process.env.RECOVERY_AUDIT_CONFIG || join(process.cwd(), 'recovery-audit.json');

  if (existsSync(configPath)) {
    try {
      const configFile = readFileSync(configPath, 'utf-8');
      return JSON.parse(configFile);
    } catch (error) {
      console.error('Failed to load config file, using defaults:', error);
    }
  }

  // Fall back to environment variables
  return {
    lokiUrl: process.env.LOKI_URL || 'http://localhost:3100',
    linearTeamId: process.env.LINEAR_TEAM_ID || '',
    enableLinearIssues: process.env.ENABLE_LINEAR_ISSUES === 'true',
    severityThreshold: parseInt(process.env.SEVERITY_THRESHOLD || '50', 10),
  };
}

/**
 * Shared audit trail instance
 */
let auditTrail: RecoveryAuditTrail | null = null;

function getAuditTrail(): RecoveryAuditTrail {
  if (!auditTrail) {
    const config = loadConfig();
    auditTrail = new RecoveryAuditTrail(config);
  }
  return auditTrail;
}

/**
 * Log recovery operation
 */
program
  .command('log')
  .description('Log a recovery operation')
  .requiredOption('--operation <type>', 'Operation type (diagnose|repair|verify|rollback)')
  .requiredOption('--session <id>', 'Session ID')
  .requiredOption('--outcome <result>', 'Operation outcome (success|failure|partial)')
  .option('--health-score <score>', 'Health score (0-100)', parseInt)
  .option('--corruption-patterns <patterns>', 'Comma-separated corruption patterns')
  .option('--backup-location <path>', 'Backup file location')
  .option('--duration <ms>', 'Operation duration in milliseconds', parseInt)
  .option('--error <message>', 'Error message if operation failed')
  .action(async (options) => {
    try {
      const event: RecoveryEvent = {
        sessionId: options.session,
        operation: options.operation as RecoveryEvent['operation'],
        timestamp: new Date().toISOString(),
        outcome: options.outcome as RecoveryEvent['outcome'],
        healthScore: options.healthScore,
        corruptionPatterns: options.corruptionPatterns?.split(','),
        backupLocation: options.backupLocation,
        durationMs: options.duration,
        errorMessage: options.error,
      };

      const audit = getAuditTrail();
      await audit.logOperation(event);

      console.log(`✓ Logged ${event.operation} operation for session ${event.sessionId.substring(0, 8)}`);
      process.exit(0);
    } catch (error) {
      console.error('Failed to log operation:', error);
      process.exit(1);
    }
  });

/**
 * Generate audit report
 */
program
  .command('report')
  .description('Generate audit report for a session')
  .requiredOption('--session <id>', 'Session ID')
  .option('--format <type>', 'Output format (json|text)', 'text')
  .action(async (options) => {
    try {
      const audit = getAuditTrail();
      const report = await audit.generateAuditReport(options.session);

      if (options.format === 'json') {
        console.log(JSON.stringify(report, null, 2));
      } else {
        console.log('\n=== Recovery Audit Report ===');
        console.log(`Session: ${report.sessionId}`);
        console.log(`\nSummary:`);
        console.log(`  Total Operations: ${report.summary.totalOperations}`);
        console.log(`  Successful: ${report.summary.successfulOperations}`);
        console.log(`  Failed: ${report.summary.failedOperations}`);
        console.log(`  Average Duration: ${report.summary.averageDuration.toFixed(2)}ms`);
        if (report.summary.finalHealthScore !== undefined) {
          console.log(`  Final Health Score: ${report.summary.finalHealthScore}%`);
        }

        console.log(`\nTimeline:`);
        console.log(report.timeline);

        console.log(`\nRecommendations:`);
        report.recommendations.forEach(rec => console.log(`  • ${rec}`));
      }

      process.exit(0);
    } catch (error) {
      console.error('Failed to generate report:', error);
      process.exit(1);
    }
  });

/**
 * Query recovery history
 */
program
  .command('query')
  .description('Query recovery history')
  .option('--session <id>', 'Filter by session ID')
  .option('--operation <type>', 'Filter by operation type')
  .option('--outcome <result>', 'Filter by outcome')
  .option('--start <time>', 'Start time (ISO 8601)')
  .option('--end <time>', 'End time (ISO 8601)')
  .option('--min-health <score>', 'Minimum health score', parseInt)
  .option('--max-health <score>', 'Maximum health score', parseInt)
  .option('--format <type>', 'Output format (json|table)', 'table')
  .action(async (options) => {
    try {
      const audit = getAuditTrail();
      const events = await audit.queryRecoveryHistory({
        sessionId: options.session,
        operation: options.operation,
        outcome: options.outcome,
        startTime: options.start,
        endTime: options.end,
        minHealthScore: options.minHealth,
        maxHealthScore: options.maxHealth,
      });

      if (options.format === 'json') {
        console.log(JSON.stringify(events, null, 2));
      } else {
        console.log(`\nFound ${events.length} event(s):\n`);
        events.forEach(event => {
          const time = new Date(event.timestamp).toISOString();
          const health = event.healthScore ? `[${event.healthScore}%]` : '';
          console.log(`${time} | ${event.operation.padEnd(10)} | ${event.outcome.padEnd(8)} | ${event.sessionId.substring(0, 8)} ${health}`);
        });
      }

      process.exit(0);
    } catch (error) {
      console.error('Failed to query history:', error);
      process.exit(1);
    }
  });

/**
 * Get metrics
 */
program
  .command('metrics')
  .description('Get recovery metrics')
  .option('--format <type>', 'Output format (json|text)', 'text')
  .action((options) => {
    try {
      const audit = getAuditTrail();
      const metrics = audit.getMetrics();

      if (options.format === 'json') {
        console.log(JSON.stringify(metrics, null, 2));
      } else {
        console.log('\n=== Recovery Metrics ===');
        console.log(`Total Events: ${metrics.totalEvents}`);
        console.log(`\nOperations:`);
        Object.entries(metrics.operationBreakdown).forEach(([op, count]) => {
          console.log(`  ${op}: ${count}`);
        });
        console.log(`\nOutcomes:`);
        Object.entries(metrics.outcomeBreakdown).forEach(([outcome, count]) => {
          console.log(`  ${outcome}: ${count}`);
        });
        console.log(`\nAverages:`);
        console.log(`  Health Score: ${metrics.averageHealthScore.toFixed(2)}%`);
        console.log(`  Duration: ${metrics.averageDuration.toFixed(2)}ms`);
        console.log(`\nCritical Failures: ${metrics.criticalFailures}`);
      }

      process.exit(0);
    } catch (error) {
      console.error('Failed to get metrics:', error);
      process.exit(1);
    }
  });

// Parse command line
program.parse();
