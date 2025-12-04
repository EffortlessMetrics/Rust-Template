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
export declare class PlatformAPIError extends Error {
    readonly status?: number | undefined;
    readonly endpoint?: string | undefined;
    constructor(message: string, status?: number | undefined, endpoint?: string | undefined);
}
/**
 * Client for Rust-as-Spec Platform APIs
 */
export declare class PlatformClient {
    private readonly baseUrl;
    private readonly timeout;
    /**
     * Create a new platform client
     *
     * @param baseUrl - Base URL of the platform service (e.g., 'http://localhost:8080')
     * @param timeout - Request timeout in milliseconds (default: 5000)
     */
    constructor(baseUrl: string, timeout?: number);
    /**
     * Internal fetch wrapper with timeout and error handling
     */
    private fetch;
    /**
     * Get platform governance status
     *
     * Fetches from /platform/status endpoint
     *
     * @returns Platform status including governance health, AC coverage, and policy status
     * @throws PlatformAPIError on network errors, timeouts, or non-200 responses
     */
    getStatus(): Promise<PlatformStatus>;
    /**
     * Get documentation inventory
     *
     * Fetches from /platform/docs/index endpoint
     *
     * @returns Documentation index with summary metrics and document list
     * @throws PlatformAPIError on network errors, timeouts, or non-200 responses
     */
    getDocsIndex(): Promise<DocsIndex>;
    /**
     * Check if the platform is reachable
     *
     * @returns true if platform responds to /platform/status
     */
    isReachable(): Promise<boolean>;
}
/**
 * Convenience function to create a client configured for Backstage proxy
 *
 * Usage in Backstage:
 *   const client = createBackstageClient();
 *   const status = await client.getStatus();
 */
export declare function createBackstageClient(): PlatformClient;
//# sourceMappingURL=PlatformClient.d.ts.map