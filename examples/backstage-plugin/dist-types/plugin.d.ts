/**
 * Rust-as-Spec Platform Plugin for Backstage
 *
 * EXAMPLE QUALITY - NOT PRODUCTION READY
 *
 * This plugin provides visibility into Rust-as-Spec platform governance
 * through Backstage UI components.
 */
/**
 * Plugin ID - must be unique across your Backstage instance
 */
export declare const rustSpecPlatformPlugin: import("@backstage/core-plugin-api").BackstagePlugin<{}, {}, {}>;
/**
 * Export components for use in Backstage app
 */
export { GovernanceHealthCard } from './components/GovernanceHealthCard';
export { DocsHealthCard } from './components/DocsHealthCard';
/**
 * Export API client for custom integrations
 */
export { PlatformClient } from './api/PlatformClient';
export type { PlatformStatus, GovernanceHealth, PolicyStatus, ACCoverage, DocsIndex, DocsSummary, DocumentEntry, } from './api/PlatformClient';
//# sourceMappingURL=plugin.d.ts.map
