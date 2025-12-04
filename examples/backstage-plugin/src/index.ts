/**
 * Rust-as-Spec Platform Plugin for Backstage
 *
 * EXAMPLE QUALITY - NOT PRODUCTION READY
 *
 * Main entry point for the plugin
 */

export { rustSpecPlatformPlugin } from './plugin';
export { GovernanceHealthCard } from './components/GovernanceHealthCard';
export { DocsHealthCard } from './components/DocsHealthCard';
export { PlatformClient, createBackstageClient } from './api/PlatformClient';
export type {
  PlatformStatus,
  ServiceInfo,
  GovernanceInfo,
  ConfigInfo,
  LedgerCounts,
  PoliciesInfo,
  FrictionInfo,
  QuestionsInfo,
  DocsIndex,
  DocsSummary,
  DocumentEntry,
  DocType,
} from './api/PlatformClient';
