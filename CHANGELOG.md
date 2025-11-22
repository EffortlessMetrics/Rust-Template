# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

(empty)

## [3.2.0] - 2025-11-22

### Added

- **AC Coverage Tooling**: New `cargo xtask ac-coverage` command to analyze BDD coverage for acceptance criteria
- **AC Scenario Scaffolding**: New `cargo xtask ac-suggest-scenarios` to generate missing BDD scenarios from AC descriptions
- **Cross-Platform Support**: Windows as a Tier-2 platform with native tooling support (Linux/macOS + Nix remain Tier-1)
- **Release Bundler**: New `cargo xtask release-bundle` command to generate comprehensive release evidence files
- **Shared AC Parsing**: Common `ac_parsing` module for AC/feature/junit parsing across xtask commands
- **Skills Tooling**: Integrated `skills-fmt` and `skills-lint` commands for `.claude/skills/` validation
- **Platform Support Documentation**: Added comprehensive Tier-1/Tier-2 platform model documentation in `MISSING_MANUAL.md` and `README.md`
- **Git Hooks Cross-Platform**: POSIX shell hooks that work on all platforms via Git for Windows `sh.exe`
- **CI Matrix**: GitHub Actions now validates Linux, macOS (Tier-1), and Windows (Tier-2) on every push

### Changed

- **Selftest Summary UX**: Enhanced 7-step summary with "Next actions" hints and clearer output formatting
- **AC Coverage in Dashboard**: AC coverage metrics now displayed in `cargo xtask status` and `/ui` dashboard
- **Skills Integration**: Skills formatting/linting integrated into `docs-check` and pre-commit workflows
- **xtask Build Strategy**: On Windows, xtask excluded from workspace commands to avoid file-locking during self-rebuild
- **Null Device Handling**: Platform-aware null device constants (`/dev/null` on Unix, `nul` on Windows)

### Fixed

- **Windows Build Failures**: Fixed Unix-specific file permission APIs causing build failures on Windows
- **Git Hook Installation**: Simplified to single POSIX hook script that works on all platforms
- **Duplicate ADR Files**: Cleaned up duplicate `adr-*.md` files in docs/decisions/
- **Test Artifacts**: Removed accidentally committed JUnit/JSON test output files
- **AC Coverage Logic**: Fixed malformed JSON parsing issues; now uses proper junit/feature mapping
- **Clippy Warnings**: Fixed `double_ended_iterator_last` and `collapsible_if` warnings in acceptance tests

### Internal

- **Governance Wiring**: Full BDD → AC → Requirement traceability implemented and validated
- **Platform Model**: Established Tier-1 (Nix + hermetic) vs Tier-2 (native Windows with caveats) support model
- **Evidence Generation**: Release artifacts (`release_evidence/*.md`) auto-generated from governance data
- **Hook Generation**: Unified hook installation using POSIX scripts recognized by Git for Windows

---

## [3.1.0] - (Previous release - content not migrated)

(Previous versions would be listed here)
