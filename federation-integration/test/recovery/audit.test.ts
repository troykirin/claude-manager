import { describe, test, expect, beforeEach } from 'bun:test';
import { RecoveryAuditTrail, type RecoveryEvent, type AuditConfig } from '../../src/recovery/audit';

describe('RecoveryAuditTrail', () => {
  let config: AuditConfig;
  let audit: RecoveryAuditTrail;

  beforeEach(() => {
    config = {
      lokiUrl: 'http://localhost:3100',
      linearTeamId: 'test-team-id',
      enableLinearIssues: false, // Disable for tests
      severityThreshold: 50,
      enableMocks: true, // Enable mock mode for testing
    };
    audit = new RecoveryAuditTrail(config);
  });

  describe('Event Logging', () => {
    test('should log diagnostic event', async () => {
      const event: RecoveryEvent = {
        sessionId: 'test-session-123',
        operation: 'diagnose',
        timestamp: new Date().toISOString(),
        outcome: 'success',
        healthScore: 75,
        corruptionPatterns: ['missing-blocks', 'corrupted-metadata'],
      };

      await audit.logDiagnostic(event);
      const history = await audit.queryRecoveryHistory({ sessionId: 'test-session-123' });

      expect(history).toHaveLength(1);
      expect(history[0].operation).toBe('diagnose');
      expect(history[0].healthScore).toBe(75);
    });

    test('should log repair event', async () => {
      const event: RecoveryEvent = {
        sessionId: 'test-session-456',
        operation: 'repair',
        timestamp: new Date().toISOString(),
        outcome: 'success',
        healthScore: 85,
        backupLocation: '/tmp/backup.json',
        durationMs: 1250,
      };

      await audit.logRepair(event);
      const history = await audit.queryRecoveryHistory({ sessionId: 'test-session-456' });

      expect(history).toHaveLength(1);
      expect(history[0].operation).toBe('repair');
      expect(history[0].backupLocation).toBe('/tmp/backup.json');
      expect(history[0].durationMs).toBe(1250);
    });

    test('should log rollback event', async () => {
      const event: RecoveryEvent = {
        sessionId: 'test-session-789',
        operation: 'rollback',
        timestamp: new Date().toISOString(),
        outcome: 'failure',
        backupLocation: '/tmp/backup.json',
        errorMessage: 'Backup file corrupted',
      };

      await audit.logRollback(event);
      const history = await audit.queryRecoveryHistory({ sessionId: 'test-session-789' });

      expect(history).toHaveLength(1);
      expect(history[0].operation).toBe('rollback');
      expect(history[0].outcome).toBe('failure');
      expect(history[0].errorMessage).toBe('Backup file corrupted');
    });
  });

  describe('Query Filtering', () => {
    beforeEach(async () => {
      // Seed test data
      const events: RecoveryEvent[] = [
        {
          sessionId: 'session-1',
          operation: 'diagnose',
          timestamp: '2025-01-01T10:00:00Z',
          outcome: 'success',
          healthScore: 80,
        },
        {
          sessionId: 'session-1',
          operation: 'repair',
          timestamp: '2025-01-01T10:05:00Z',
          outcome: 'success',
          healthScore: 90,
        },
        {
          sessionId: 'session-2',
          operation: 'diagnose',
          timestamp: '2025-01-01T11:00:00Z',
          outcome: 'failure',
          healthScore: 30,
        },
        {
          sessionId: 'session-2',
          operation: 'rollback',
          timestamp: '2025-01-01T11:05:00Z',
          outcome: 'success',
          healthScore: 70,
        },
      ];

      for (const event of events) {
        await audit.logOperation(event);
      }
    });

    test('should filter by session ID', async () => {
      const history = await audit.queryRecoveryHistory({ sessionId: 'session-1' });

      expect(history).toHaveLength(2);
      expect(history.every(e => e.sessionId === 'session-1')).toBe(true);
    });

    test('should filter by operation', async () => {
      const history = await audit.queryRecoveryHistory({ operation: 'diagnose' });

      expect(history).toHaveLength(2);
      expect(history.every(e => e.operation === 'diagnose')).toBe(true);
    });

    test('should filter by outcome', async () => {
      const history = await audit.queryRecoveryHistory({ outcome: 'failure' });

      expect(history).toHaveLength(1);
      expect(history[0].sessionId).toBe('session-2');
      expect(history[0].operation).toBe('diagnose');
    });

    test('should filter by health score range', async () => {
      const history = await audit.queryRecoveryHistory({
        minHealthScore: 70,
        maxHealthScore: 90,
      });

      expect(history).toHaveLength(3);
      expect(history.every(e => e.healthScore && e.healthScore >= 70 && e.healthScore <= 90)).toBe(true);
    });

    test('should filter by time range', async () => {
      const history = await audit.queryRecoveryHistory({
        startTime: '2025-01-01T10:30:00Z',
        endTime: '2025-01-01T11:30:00Z',
      });

      expect(history).toHaveLength(2);
      expect(history.every(e => e.sessionId === 'session-2')).toBe(true);
    });
  });

  describe('Audit Report Generation', () => {
    beforeEach(async () => {
      const events: RecoveryEvent[] = [
        {
          sessionId: 'report-session',
          operation: 'diagnose',
          timestamp: '2025-01-01T10:00:00Z',
          outcome: 'success',
          healthScore: 50,
          durationMs: 100,
        },
        {
          sessionId: 'report-session',
          operation: 'repair',
          timestamp: '2025-01-01T10:05:00Z',
          outcome: 'success',
          healthScore: 85,
          durationMs: 2000,
        },
        {
          sessionId: 'report-session',
          operation: 'verify',
          timestamp: '2025-01-01T10:10:00Z',
          outcome: 'success',
          healthScore: 90,
          durationMs: 150,
        },
      ];

      for (const event of events) {
        await audit.logOperation(event);
      }
    });

    test('should generate complete audit report', async () => {
      const report = await audit.generateAuditReport('report-session');

      expect(report.sessionId).toBe('report-session');
      expect(report.events).toHaveLength(3);
      expect(report.summary.totalOperations).toBe(3);
      expect(report.summary.successfulOperations).toBe(3);
      expect(report.summary.failedOperations).toBe(0);
      expect(report.summary.finalHealthScore).toBe(90);
    });

    test('should calculate average duration', async () => {
      const report = await audit.generateAuditReport('report-session');

      // (100 + 2000 + 150) / 3 = 750
      expect(report.summary.averageDuration).toBe(750);
    });

    test('should generate timeline', async () => {
      const report = await audit.generateAuditReport('report-session');

      expect(report.timeline).toContain('diagnose');
      expect(report.timeline).toContain('repair');
      expect(report.timeline).toContain('verify');
      expect(report.timeline).toContain('✓');
    });

    test('should generate recommendations for healthy session', async () => {
      const report = await audit.generateAuditReport('report-session');

      expect(report.recommendations).toContain('All operations successful - no action needed');
    });

    test('should generate recommendations for failures', async () => {
      await audit.logOperation({
        sessionId: 'failed-session',
        operation: 'repair',
        timestamp: new Date().toISOString(),
        outcome: 'failure',
        errorMessage: 'Test failure',
      });

      const report = await audit.generateAuditReport('failed-session');

      expect(report.recommendations.some(r => r.includes('failed'))).toBe(true);
    });

    test('should throw error for non-existent session', async () => {
      expect(async () => {
        await audit.generateAuditReport('non-existent-session');
      }).toThrow();
    });
  });

  describe('Metrics Calculation', () => {
    beforeEach(async () => {
      const events: RecoveryEvent[] = [
        {
          sessionId: 's1',
          operation: 'diagnose',
          timestamp: new Date().toISOString(),
          outcome: 'success',
          healthScore: 80,
          durationMs: 100,
        },
        {
          sessionId: 's2',
          operation: 'diagnose',
          timestamp: new Date().toISOString(),
          outcome: 'failure',
          healthScore: 20,
          durationMs: 200,
        },
        {
          sessionId: 's3',
          operation: 'repair',
          timestamp: new Date().toISOString(),
          outcome: 'success',
          healthScore: 90,
          durationMs: 1500,
        },
        {
          sessionId: 's4',
          operation: 'rollback',
          timestamp: new Date().toISOString(),
          outcome: 'success',
          healthScore: 70,
          durationMs: 500,
        },
      ];

      for (const event of events) {
        await audit.logOperation(event);
      }
    });

    test('should calculate total events', () => {
      const metrics = audit.getMetrics();

      expect(metrics.totalEvents).toBe(4);
    });

    test('should calculate operation breakdown', () => {
      const metrics = audit.getMetrics();

      expect(metrics.operationBreakdown.diagnose).toBe(2);
      expect(metrics.operationBreakdown.repair).toBe(1);
      expect(metrics.operationBreakdown.rollback).toBe(1);
    });

    test('should calculate outcome breakdown', () => {
      const metrics = audit.getMetrics();

      expect(metrics.outcomeBreakdown.success).toBe(3);
      expect(metrics.outcomeBreakdown.failure).toBe(1);
    });

    test('should calculate average health score', () => {
      const metrics = audit.getMetrics();

      // (80 + 20 + 90 + 70) / 4 = 65
      expect(metrics.averageHealthScore).toBe(65);
    });

    test('should calculate average duration', () => {
      const metrics = audit.getMetrics();

      // (100 + 200 + 1500 + 500) / 4 = 575
      expect(metrics.averageDuration).toBe(575);
    });

    test('should count critical failures', () => {
      const metrics = audit.getMetrics();

      // One failure with health=20 (counts once, not twice)
      expect(metrics.criticalFailures).toBe(1);
    });
  });

  describe('Loki Payload Format', () => {
    test('should format event for Loki ingestion', async () => {
      // This test verifies the internal formatting logic
      // by checking that events can be logged without errors
      const event: RecoveryEvent = {
        sessionId: 'loki-test',
        operation: 'diagnose',
        timestamp: new Date().toISOString(),
        outcome: 'success',
        healthScore: 75,
        corruptionPatterns: ['test-pattern'],
      };

      // Should not throw
      await audit.logDiagnostic(event);
      const history = await audit.queryRecoveryHistory({ sessionId: 'loki-test' });
      expect(history).toHaveLength(1);
    });
  });

  describe('Linear Issue Creation', () => {
    test('should not create issue when disabled', async () => {
      const event: RecoveryEvent = {
        sessionId: 'linear-test',
        operation: 'repair',
        timestamp: new Date().toISOString(),
        outcome: 'failure',
        healthScore: 20,
      };

      // Should not throw even though it's a critical failure
      await audit.logRepair(event);
      const history = await audit.queryRecoveryHistory({ sessionId: 'linear-test' });
      expect(history).toHaveLength(1);
      expect(history[0].outcome).toBe('failure');
    });

    test('should handle Linear issue creation when enabled', async () => {
      const configWithLinear: AuditConfig = {
        ...config,
        enableLinearIssues: true,
      };
      const auditWithLinear = new RecoveryAuditTrail(configWithLinear);

      const event: RecoveryEvent = {
        sessionId: 'linear-enabled-test',
        operation: 'repair',
        timestamp: new Date().toISOString(),
        outcome: 'failure',
        healthScore: 20,
        errorMessage: 'Critical failure',
      };

      // Should not throw (mock implementation)
      await auditWithLinear.logRepair(event);
      const history = await auditWithLinear.queryRecoveryHistory({ sessionId: 'linear-enabled-test' });
      expect(history).toHaveLength(1);
    });
  });
});
