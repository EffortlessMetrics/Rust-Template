/**
 * API Client for Rust-as-Spec Platform
 *
 * EXAMPLE QUALITY - NOT PRODUCTION READY
 *
 * This client provides typed access to the platform's governance APIs.
 * In production, add:
 * - Authentication/authorization
 * - Retry logic and exponential backoff
 * - Request cancellation
 * - Response caching
 * - Rate limiting
 * - Comprehensive error handling
 */

/**
 * AC Coverage metrics
 */
export interface ACCoverage {
  total: number;
  passing: number;
  failing: number;
  pending: number;
  percentage: number;
}

/**
 * Policy enforcement status
 */
export interface PolicyStatus {
  skills_valid: boolean;
  agents_valid: boolean;
  adrs_valid: boolean;
  specs_valid: boolean;
  bdd_valid: boolean;
}

/**
 * Governance health metrics
 */
export interface GovernanceHealth {
  ac_coverage: ACCoverage;
  policy_status: PolicyStatus;
  selftest_passing: boolean;
  last_validated: string;
}

/**
 * Platform status response from /platform/status
 */
export interface PlatformStatus {
  status: string;
  version: string;
  governance: GovernanceHealth;
  auth_mode: string;
  metadata: {
    template_version: string;
    schema_version: string;
    last_updated: string;
  };
}

/**
 * Individual document entry
 */
export interface DocumentEntry {
  id: string;
  type: 'adr' | 'design' | 'how-to' | 'explanation' | 'reference';
  title: string;
  path: string;
  status: 'valid' | 'has_issues';
  issues?: string[];
  linked_reqs?: string[];
  linked_acs?: string[];
}

/**
 * Documentation summary metrics
 */
export interface DocsSummary {
  total: number;
  valid: number;
  with_issues: number;
  by_type: {
    adr: number;
    design: number;
    'how-to': number;
    explanation: number;
    reference: number;
  };
}

/**
 * Documentation index response from /platform/docs/index
 */
export interface DocsIndex {
  summary: DocsSummary;
  documents: DocumentEntry[];
}

/**
 * API Error with context
 */
export class PlatformAPIError extends Error {
  constructor(
    message: string,
    public readonly status?: number,
    public readonly endpoint?: string,
  ) {
    super(message);
    this.name = 'PlatformAPIError';
  }
}

/**
 * Client for Rust-as-Spec Platform APIs
 */
export class PlatformClient {
  private readonly baseUrl: string;
  private readonly timeout: number;

  /**
   * Create a new platform client
   *
   * @param baseUrl - Base URL of the platform service (e.g., 'http://localhost:8080')
   * @param timeout - Request timeout in milliseconds (default: 5000)
   */
  constructor(baseUrl: string, timeout: number = 5000) {
    this.baseUrl = baseUrl.replace(/\/$/, ''); // Remove trailing slash
    this.timeout = timeout;
  }

  /**
   * Internal fetch wrapper with timeout and error handling
   */
  private async fetch<T>(endpoint: string): Promise<T> {
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), this.timeout);

    try {
      const url = `${this.baseUrl}${endpoint}`;
      const response = await fetch(url, {
        signal: controller.signal,
        headers: {
          'Accept': 'application/json',
        },
      });

      if (!response.ok) {
        throw new PlatformAPIError(
          `HTTP ${response.status}: ${response.statusText}`,
          response.status,
          endpoint,
        );
      }

      const data = await response.json();
      return data as T;
    } catch (error) {
      if (error instanceof PlatformAPIError) {
        throw error;
      }

      if (error instanceof Error) {
        if (error.name === 'AbortError') {
          throw new PlatformAPIError(
            `Request timeout after ${this.timeout}ms`,
            undefined,
            endpoint,
          );
        }
        throw new PlatformAPIError(
          `Network error: ${error.message}`,
          undefined,
          endpoint,
        );
      }

      throw new PlatformAPIError(
        'Unknown error occurred',
        undefined,
        endpoint,
      );
    } finally {
      clearTimeout(timeoutId);
    }
  }

  /**
   * Get platform governance status
   *
   * Fetches from /platform/status endpoint
   *
   * @returns Platform status including governance health, AC coverage, and policy status
   * @throws PlatformAPIError on network errors, timeouts, or non-200 responses
   */
  async getStatus(): Promise<PlatformStatus> {
    return this.fetch<PlatformStatus>('/platform/status');
  }

  /**
   * Get documentation inventory
   *
   * Fetches from /platform/docs/index endpoint
   *
   * @returns Documentation index with summary metrics and document list
   * @throws PlatformAPIError on network errors, timeouts, or non-200 responses
   */
  async getDocsIndex(): Promise<DocsIndex> {
    return this.fetch<DocsIndex>('/platform/docs/index');
  }

  /**
   * Check if the platform is reachable
   *
   * @returns true if platform responds to /platform/status
   */
  async isReachable(): Promise<boolean> {
    try {
      await this.getStatus();
      return true;
    } catch {
      return false;
    }
  }
}

/**
 * Convenience function to create a client configured for Backstage proxy
 *
 * Usage in Backstage:
 *   const client = createBackstageClient();
 *   const status = await client.getStatus();
 */
export function createBackstageClient(): PlatformClient {
  // Backstage proxy is typically at /api/proxy/<proxy-path>
  return new PlatformClient('/api/proxy/rust-spec-platform');
}
