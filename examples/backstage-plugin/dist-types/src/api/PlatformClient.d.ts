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
/**
 * Service metadata
 */
export interface ServiceInfo {
    service_id: string;
    template_version: string;
    display_name: string;
    description: string;
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
 */
export interface PoliciesInfo {
    status: 'pass' | 'fail';
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
 */
export interface ConfigInfo {
    env: string;
    http_port: number;
    settings: Record<string, unknown>;
    secrets_redacted: Record<string, string>;
    auth: AuthConfig;
}
/**
 * Platform status response from /platform/status
 */
export interface PlatformStatus {
    service: ServiceInfo;
    governance: GovernanceInfo;
    config: ConfigInfo;
}
/**
 * Document type enumeration
 */
export type DocType = 'adr' | 'design_doc' | 'impl_plan' | 'requirements_doc' | 'guide' | 'how-to' | 'how_to' | 'explanation' | 'reference' | 'ci_workflow' | 'status';
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
     * @returns Platform status including service info, governance metrics, and config
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
    /**
     * Get AC count from status
     */
    getACCount(): Promise<number>;
    /**
     * Get policy pass/fail status
     */
    getPolicyStatus(): Promise<'pass' | 'fail'>;
    /**
     * Get template version
     */
    getTemplateVersion(): Promise<string>;
    /**
     * Get open friction issues
     */
    getOpenFrictionCount(): Promise<number>;
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