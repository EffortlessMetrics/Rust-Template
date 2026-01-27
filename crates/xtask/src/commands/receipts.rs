//! CLI wiring for receipts commands.
//!
//! This module re-exports the xtask-receipts engine API.

pub use xtask_receipts::{
    ReceiptsEconomicsArgs, ReceiptsForensicArgs, ReceiptsGateArgs, ReceiptsQualityArgs,
    ReceiptsTelemetryArgs, ReceiptsTimelineArgs, ReceiptsValidateArgs, run_economics, run_forensic,
    run_gate, run_quality, run_telemetry, run_timeline, run_validate,
};
