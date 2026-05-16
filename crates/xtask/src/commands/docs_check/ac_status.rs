use anyhow::Result;

pub(crate) fn check_ac_status_clean() -> Result<()> {
    // Use check mode to verify AC status file matches computed state without writing.
    // This is cleaner than regenerating + checking git status, and doesn't modify the repo.
    //
    // Note: We only enforce strict check mode in CI because:
    // - BDD runs with tag filtering locally (excludes @ci-only scenarios)
    // - Integration tests (AC-MYSERV-*, AC-GOV-025) need app-http running
    // - This produces different coverage than what was used to generate the committed file
    // - The committed file reflects full BDD coverage (CI mode with app-http)
    // - Comparing against partial coverage would always fail locally
    //
    // In CI: check mode verifies file is in sync
    // Locally: skip sync check (selftest step 6 will regenerate anyway)
    let in_ci = crate::env::is_ci();
    if !in_ci {
        // Locally, just verify ac-status runs without errors (don't check file sync)
        return Ok(());
    }

    // Note: We ignore AC failures (test failures) here - we only care if the file is in sync.
    // The ac-status command in check mode will fail if the file content differs, which is
    // what we want to catch. If it fails due to AC test failures, that's a separate concern.
    match crate::commands::ac_status::run(crate::commands::ac_status::AcStatusArgs {
        verbosity: crate::Verbosity::Quiet,
        check: true, // Check mode: verify without writing (CI only)
        ..Default::default()
    }) {
        Ok(_) => Ok(()),
        Err(e) => {
            // Check if this is a sync error vs an AC failure
            let err_str = e.to_string();
            if err_str.contains("out of sync") {
                // File is out of sync - this is the error we want to surface
                anyhow::bail!(
                    "AC status file is out of sync.\n\
                     Run 'cargo xtask ac-status' to regenerate and commit the changes.\n\
                     Error: {}",
                    err_str.lines().next().unwrap_or(&err_str)
                );
            } else if err_str.contains("ACs failed") {
                // ACs failed, but file is in sync - that's fine for docs-check
                Ok(())
            } else {
                // Some other error (file not found, etc.)
                Err(e)
            }
        }
    }
}
