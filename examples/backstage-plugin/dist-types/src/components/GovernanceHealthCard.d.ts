/**
 * Governance Health Card Component
 *
 * EXAMPLE QUALITY - NOT PRODUCTION READY
 *
 * Displays governance health metrics from /platform/status endpoint.
 * Shows ledger counts, policy status, friction, and questions.
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
 * - Ledger counts (stories, requirements, ACs)
 * - Policy enforcement status
 * - Friction tracking
 * - Open questions
 * - Auto-refresh capability
 *
 * Example usage:
 *   <GovernanceHealthCard />
 *   <GovernanceHealthCard baseUrl="http://localhost:9090" refreshInterval={60000} />
 */
export declare const GovernanceHealthCard: React.FC<GovernanceHealthCardProps>;
export {};
//# sourceMappingURL=GovernanceHealthCard.d.ts.map