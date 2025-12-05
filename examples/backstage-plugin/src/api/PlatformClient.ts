/**
 * API Client for Rust-as-Spec Platform
 *
 * EXAMPLE QUALITY - NOT PRODUCTION READY
 *
 * This client provides typed access to the platform's governance APIs.
 * Types are derived from actual API responses (validated 2025-12-04).
 *
 * In production, add:
 * - Authentication/authorization
 * - Retry logic and exponential backoff
 * - Request cancellation
 * - Response caching
 * - Rate limiting
 * - Comprehensive error handling
 */

// =============================================================================
// /platform/status types
// =============================================================================

/**
 * Service metadata
 *
 * Note: display_name and description are optional in the Rust kernel
 * (Option<String> with skip_serializing_if) so they may be absent from the JSON.
 */
export interface ServiceInfo {
  service_id: string;
  template_version: string;
  display_name?: string;
  description?: string;
  links: Record<string, string>;
  tags: string[];
}

/**
 * Governance ledger counts
 */
export interface LedgerCounts {
  stories: number;
  requirements: number;
  acs: number;
}

/**
 * DevEx command/flow counts
 */
export interface DevExCounts {
  commands: number;
  flows: number;
}

/**
 * Documentation summary in status
 */
export interface DocsCounts {
  total: number;
  design: number;
  doc_type_issues: number;
}

/**
 * Task counts
 */
export interface TasksCounts {
  total: number;
}

/**
 * Question summary entry
 */
export interface QuestionSummary {
  id: string;
  summary: string;
  flow: string;
}

/**
 * Questions tracking
 */
export interface QuestionsInfo {
  open: number;
  answered: number;
  resolved: number;
  total: number;
  top_open: QuestionSummary[];
}

/**
 * Friction log entry summary
 */
export interface FrictionSummary {
  id: string;
  date: string;
  severity: 'low' | 'medium' | 'high' | 'critical';
  summary: string;
  category: string;
}

/**
 * Friction tracking
 */
export interface FrictionInfo {
  total: number;
  open: number;
  by_severity: {
    low: number;
    medium: number;
    high: number;
    critical: number;
  };
  recent: FrictionSummary[];
}

/**
 * Fork tracking
 */
export interface ForksInfo {
  total: number;
  ids: string[];
}

/**
 * Policy enforcement status
 *
 * Note: status can be 'unknown' when policy-test hasn't run or failed to parse.
 */
export interface PoliciesInfo {
  status: 'pass' | 'fail' | 'unknown';
}

/**
 * Full governance health metrics from /platform/status
 */
export interface GovernanceInfo {
  ledger: LedgerCounts;
  devex: DevExCounts;
  docs: DocsCounts;
  tasks: TasksCounts;
  questions: QuestionsInfo;
  friction: FrictionInfo;
  forks: ForksInfo;
  policies: PoliciesInfo;
}

/**
 * Auth configuration
 */
export interface AuthConfig {
  mode: 'open' | 'token' | 'oidc';
  token_present: boolean;
}

/**
 * Runtime configuration (from /platform/status)
 *
 * Note: env is optional in the Rust kernel (Option<String>).
 */
export interface ConfigInfo {
  env?: string;
  http_port: number;
  settings: Record<string, unknown>;
  secrets_redacted: Record<string, string>;
  auth: AuthConfig;
}

/**
 * Platform status response from /platform/status
 *
 * Note: config is optional in the Rust kernel (Option<ConfigSummary> with skip_serializing_if).
 * It may be absent if no validated config was loaded.
 */
export interface PlatformStatus {
  service: ServiceInfo;
  governance: GovernanceInfo;
  config?: ConfigInfo;
}

// =============================================================================
// /platform/docs/index types
// =============================================================================

/**
 * Document type enumeration
 */
export type DocType =
  | 'adr'
  | 'design_doc'
  | 'impl_plan'
  | 'requirements_doc'
  | 'guide'
  | 'how-to'
  | 'how_to'
  | 'explanation'
  | 'reference'
  | 'ci_workflow'
  | 'status';

/**
 * Individual document entry from /platform/docs/index
 */
export interface DocumentEntry {
  id: string;
  file: string;
  doc_type: DocType;
  stories: string[];
  requirements: string[];
  acs: string[];
  adrs: string[];
  doc_type_valid: boolean;
}

/**
 * Documentation summary metrics
 */
export interface DocsSummary {
  total: number;
  valid: number;
  with_issues: number;
}

/**
 * Documentation index response from /platform/docs/index
 */
export interface DocsIndex {
  schema_version: string;
  template_version: string;
  docs: DocumentEntry[];
  summary: DocsSummary;
}

// =============================================================================
// Error handling
// =============================================================================

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

// =============================================================================
// Client implementation
// =============================================================================

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
          Accept: 'application/json',
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

      throw new PlatformAPIError('Unknown error occurred', undefined, endpoint);
    } finally {
      clearTimeout(timeoutId);
    }
  }

  /**
   * Get platform governance status
   *
   * Fetches from /platform/status endpoint
   *
   * @returns Platform status including service info, governance metrics, and config
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

  // =========================================================================
  // Convenience accessors (derived from actual API structure)
  // =========================================================================

  /**
   * Get AC count from status
   */
  async getACCount(): Promise<number> {
    const status = await this.getStatus();
    return status.governance.ledger.acs;
  }

  /**
   * Get policy pass/fail/unknown status
   */
  async getPolicyStatus(): Promise<'pass' | 'fail' | 'unknown'> {
    const status = await this.getStatus();
    return status.governance.policies.status;
  }

  /**
   * Get template version
   */
  async getTemplateVersion(): Promise<string> {
    const status = await this.getStatus();
    return status.service.template_version;
  }

  /**
   * Get open friction issues
   */
  async getOpenFrictionCount(): Promise<number> {
    const status = await this.getStatus();
    return status.governance.friction.open;
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
