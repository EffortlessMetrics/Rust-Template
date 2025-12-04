/**
 * Governance Health Card Component
 *
 * EXAMPLE QUALITY - NOT PRODUCTION READY
 *
 * Displays governance health metrics from /platform/status endpoint.
 * Shows AC coverage, policy status, and selftest gate health.
 */
import React from 'react';
interface GovernanceHealthCardProps {
    /**
     * Optional custom base URL (defaults to Backstage proxy)
     */
    baseUrl?: string;
    /**
     * Refresh interval in milliseconds (default: 30000 = 30 seconds)
     * Set to 0 to disable auto-refresh
     */
    refreshInterval?: number;
}
/**
 * GovernanceHealthCard displays real-time governance metrics
 *
 * Features:
 * - AC coverage with progress bar
 * - Policy enforcement status badges
 * - Selftest gate indicator
 * - Auto-refresh capability
 *
 * Example usage:
 *   <GovernanceHealthCard />
 *   <GovernanceHealthCard baseUrl="http://localhost:8080" refreshInterval={60000} />
 */
export declare const GovernanceHealthCard: React.FC<GovernanceHealthCardProps>;
export {};
//# sourceMappingURL=GovernanceHealthCard.d.ts.map