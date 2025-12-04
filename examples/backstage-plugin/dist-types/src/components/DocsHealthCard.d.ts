/**
 * Documentation Health Card Component
 *
 * EXAMPLE QUALITY - NOT PRODUCTION READY
 *
 * Displays documentation inventory metrics from /platform/docs/index endpoint.
 * Shows total docs, health status, and breakdown by document type.
 */
import React from 'react';
interface DocsHealthCardProps {
    /**
     * Optional custom base URL (defaults to Backstage proxy)
     */
    baseUrl?: string;
    /**
     * Refresh interval in milliseconds (default: 60000 = 60 seconds)
     * Set to 0 to disable auto-refresh
     */
    refreshInterval?: number;
}
/**
 * DocsHealthCard displays documentation inventory metrics
 *
 * Features:
 * - Total document count
 * - Health indicators (valid vs. with issues)
 * - Breakdown by document type (ADRs, design docs, how-tos, etc.)
 * - Auto-refresh capability
 *
 * Example usage:
 *   <DocsHealthCard />
 *   <DocsHealthCard baseUrl="http://localhost:8080" refreshInterval={120000} />
 */
export declare const DocsHealthCard: React.FC<DocsHealthCardProps>;
export {};
//# sourceMappingURL=DocsHealthCard.d.ts.map