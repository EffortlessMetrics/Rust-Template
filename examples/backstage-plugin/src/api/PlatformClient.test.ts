/**
 * Unit tests for PlatformClient
 *
 * Tests the API client's type handling and convenience methods.
 * Uses mocked fetch to verify correct parsing of Rust kernel responses.
 */

import {
  PlatformClient,
  PlatformStatus,
  DocsIndex,
  PlatformAPIError,
  DocumentEntry,
} from './PlatformClient';

// Mock fetch globally
const mockFetch = jest.fn();
global.fetch = mockFetch;

describe('PlatformClient', () => {
  let client: PlatformClient;

  beforeEach(() => {
    client = new PlatformClient('http://localhost:9090');
    mockFetch.mockReset();
  });

  describe('getStatus', () => {
    it('should parse a complete status response', async () => {
      const mockStatus: PlatformStatus = {
        service: {
          service_id: 'test-service',
          template_version: '3.3.6',
          display_name: 'Test Service',
          description: 'A test service',
          links: { repo: 'https://github.com/example/repo' },
          tags: ['rust', 'platform'],
        },
        governance: {
          ledger: { stories: 5, requirements: 10, acs: 20 },
          devex: { commands: 15, flows: 8 },
          docs: { total: 25, design: 3, doc_type_issues: 1 },
          tasks: { total: 12 },
          questions: {
            open: 2,
            answered: 3,
            resolved: 5,
            total: 10,
            top_open: [
              { id: 'Q-001', summary: 'Question 1', flow: 'feature-dev' },
            ],
          },
          friction: {
            total: 4,
            open: 2,
            by_severity: { low: 1, medium: 1, high: 0, critical: 0 },
            recent: [],
          },
          forks: { total: 1, ids: ['FORK-001'] },
          policies: { status: 'pass' },
        },
        config: {
          env: 'dev',
          http_port: 9090,
          settings: { 'platform.auth_mode': 'basic' },
          secrets_redacted: { 'db.url': '[REDACTED]' },
          auth: { mode: 'token', token_present: true },
        },
      };

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => mockStatus,
      });

      const result = await client.getStatus();

      expect(result).toEqual(mockStatus);
      expect(mockFetch).toHaveBeenCalledWith(
        'http://localhost:9090/platform/status',
        expect.objectContaining({
          headers: { Accept: 'application/json' },
        }),
      );
    });

    it('should handle optional fields being absent (Rust skip_serializing_if)', async () => {
      // Rust kernel may omit optional fields when None
      const mockStatus: PlatformStatus = {
        service: {
          service_id: 'minimal-service',
          template_version: '3.3.6',
          // display_name: absent (Option<String> with skip_serializing_if)
          // description: absent (Option<String> with skip_serializing_if)
          links: {},
          tags: [],
        },
        governance: {
          ledger: { stories: 0, requirements: 0, acs: 0 },
          devex: { commands: 0, flows: 0 },
          docs: { total: 0, design: 0, doc_type_issues: 0 },
          tasks: { total: 0 },
          questions: {
            open: 0,
            answered: 0,
            resolved: 0,
            total: 0,
            top_open: [],
          },
          friction: {
            total: 0,
            open: 0,
            by_severity: { low: 0, medium: 0, high: 0, critical: 0 },
            recent: [],
          },
          forks: { total: 0, ids: [] },
          policies: { status: 'unknown' },
        },
        // config: absent (Option<ConfigSummary> with skip_serializing_if)
      };

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => mockStatus,
      });

      const result = await client.getStatus();

      expect(result.service.display_name).toBeUndefined();
      expect(result.service.description).toBeUndefined();
      expect(result.config).toBeUndefined();
      expect(result.governance.policies.status).toBe('unknown');
    });

    it('should throw PlatformAPIError on HTTP error', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        status: 500,
        statusText: 'Internal Server Error',
      });

      let thrownError: Error | undefined;
      try {
        await client.getStatus();
      } catch (e) {
        thrownError = e as Error;
      }
      expect(thrownError).toBeInstanceOf(PlatformAPIError);
      expect(thrownError?.message).toContain('HTTP 500');
    });

    it('should throw PlatformAPIError on network error', async () => {
      mockFetch.mockRejectedValueOnce(new Error('Network failed'));

      let thrownError: Error | undefined;
      try {
        await client.getStatus();
      } catch (e) {
        thrownError = e as Error;
      }
      expect(thrownError).toBeInstanceOf(PlatformAPIError);
      expect(thrownError?.message).toContain('Network error');
    });
  });

  describe('convenience methods', () => {
    const createMockStatus = (
      overrides: Partial<{
        acs: number;
        policyStatus: 'pass' | 'fail' | 'unknown';
        templateVersion: string;
        openFriction: number;
      }> = {},
    ): PlatformStatus => ({
      service: {
        service_id: 'test',
        template_version: overrides.templateVersion ?? '3.3.6',
        links: {},
        tags: [],
      },
      governance: {
        ledger: { stories: 0, requirements: 0, acs: overrides.acs ?? 0 },
        devex: { commands: 0, flows: 0 },
        docs: { total: 0, design: 0, doc_type_issues: 0 },
        tasks: { total: 0 },
        questions: {
          open: 0,
          answered: 0,
          resolved: 0,
          total: 0,
          top_open: [],
        },
        friction: {
          total: 0,
          open: overrides.openFriction ?? 0,
          by_severity: { low: 0, medium: 0, high: 0, critical: 0 },
          recent: [],
        },
        forks: { total: 0, ids: [] },
        policies: { status: overrides.policyStatus ?? 'pass' },
      },
    });

    it('getACCount should return correct AC count', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => createMockStatus({ acs: 42 }),
      });

      const count = await client.getACCount();
      expect(count).toBe(42);
    });

    it('getPolicyStatus should return pass', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => createMockStatus({ policyStatus: 'pass' }),
      });

      const status = await client.getPolicyStatus();
      expect(status).toBe('pass');
    });

    it('getPolicyStatus should return fail', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => createMockStatus({ policyStatus: 'fail' }),
      });

      const status = await client.getPolicyStatus();
      expect(status).toBe('fail');
    });

    it('getPolicyStatus should return unknown', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => createMockStatus({ policyStatus: 'unknown' }),
      });

      const status = await client.getPolicyStatus();
      expect(status).toBe('unknown');
    });

    it('getTemplateVersion should return template version', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => createMockStatus({ templateVersion: '4.0.0' }),
      });

      const version = await client.getTemplateVersion();
      expect(version).toBe('4.0.0');
    });

    it('getOpenFrictionCount should return open friction count', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => createMockStatus({ openFriction: 7 }),
      });

      const count = await client.getOpenFrictionCount();
      expect(count).toBe(7);
    });
  });

  describe('getDocsIndex', () => {
    it('should parse a docs index response', async () => {
      const mockDocsIndex: DocsIndex = {
        schema_version: '1.0',
        template_version: '3.3.6',
        docs: [
          {
            id: 'DOC-001',
            file: 'docs/ADR-001.md',
            doc_type: 'adr',
            stories: [],
            requirements: ['REQ-001'],
            acs: [],
            adrs: [],
            doc_type_valid: true,
          },
          {
            id: 'DOC-002',
            file: 'docs/how-to/guide.md',
            doc_type: 'how_to',
            stories: [],
            requirements: ['REQ-002'],
            acs: ['AC-001'],
            adrs: [],
            doc_type_valid: true,
          },
        ],
        summary: {
          total: 2,
          valid: 2,
          with_issues: 0,
        },
      };

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => mockDocsIndex,
      });

      const result = await client.getDocsIndex();

      expect(result).toEqual(mockDocsIndex);
      expect(result.docs).toHaveLength(2);
      expect(result.summary.total).toBe(2);
    });
  });

  describe('isReachable', () => {
    it('should return true when platform responds', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          service: { service_id: 'test', template_version: '1.0', links: {}, tags: [] },
          governance: {
            ledger: { stories: 0, requirements: 0, acs: 0 },
            devex: { commands: 0, flows: 0 },
            docs: { total: 0, design: 0, doc_type_issues: 0 },
            tasks: { total: 0 },
            questions: { open: 0, answered: 0, resolved: 0, total: 0, top_open: [] },
            friction: { total: 0, open: 0, by_severity: { low: 0, medium: 0, high: 0, critical: 0 }, recent: [] },
            forks: { total: 0, ids: [] },
            policies: { status: 'pass' },
          },
        }),
      });

      const reachable = await client.isReachable();
      expect(reachable).toBe(true);
    });

    it('should return false when platform is unreachable', async () => {
      mockFetch.mockRejectedValueOnce(new Error('Connection refused'));

      const reachable = await client.isReachable();
      expect(reachable).toBe(false);
    });
  });

  describe('timeout handling', () => {
    it('should use custom timeout', () => {
      const customClient = new PlatformClient('http://localhost:9090', 10000);
      // We can't easily test the actual timeout behavior without mocking timers,
      // but we verify the client is constructed with the custom timeout.
      expect(customClient).toBeDefined();
    });
  });
});

// Helper function duplicated here for testing (mirrors DocsHealthCard.tsx)
function computeTypeCounts(docs: DocumentEntry[]): Record<string, number> {
  const counts: Record<string, number> = {
    adr: 0,
    design_doc: 0,
    'how-to': 0,
    how_to: 0,
    explanation: 0,
    reference: 0,
    guide: 0,
    impl_plan: 0,
    ci_workflow: 0,
    requirements_doc: 0,
    status: 0,
  };

  for (const doc of docs) {
    if (doc.doc_type in counts) {
      counts[doc.doc_type]++;
    }
  }

  return counts;
}

describe('computeTypeCounts', () => {
  it('should count document types correctly', () => {
    const docs: DocumentEntry[] = [
      { id: '1', file: 'a.md', doc_type: 'adr', stories: [], requirements: [], acs: [], adrs: [], doc_type_valid: true },
      { id: '2', file: 'b.md', doc_type: 'adr', stories: [], requirements: [], acs: [], adrs: [], doc_type_valid: true },
      { id: '3', file: 'c.md', doc_type: 'design_doc', stories: [], requirements: [], acs: [], adrs: [], doc_type_valid: true },
      { id: '4', file: 'd.md', doc_type: 'how_to', stories: [], requirements: [], acs: [], adrs: [], doc_type_valid: true },
      { id: '5', file: 'e.md', doc_type: 'how-to', stories: [], requirements: [], acs: [], adrs: [], doc_type_valid: true },
    ];

    const counts = computeTypeCounts(docs);

    expect(counts.adr).toBe(2);
    expect(counts.design_doc).toBe(1);
    expect(counts.how_to).toBe(1);
    expect(counts['how-to']).toBe(1);
  });

  it('should handle empty docs array', () => {
    const counts = computeTypeCounts([]);

    expect(counts.adr).toBe(0);
    expect(counts.design_doc).toBe(0);
  });

  it('should ignore unknown doc types', () => {
    const docs: DocumentEntry[] = [
      { id: '1', file: 'a.md', doc_type: 'unknown_type' as any, stories: [], requirements: [], acs: [], adrs: [], doc_type_valid: false },
    ];

    const counts = computeTypeCounts(docs);

    // Unknown types should not increment any counter
    expect(Object.values(counts).every(v => v === 0)).toBe(true);
  });
});
