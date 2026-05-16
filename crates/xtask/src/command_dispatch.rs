use anyhow::Result;

use crate::cli::Commands;
use crate::{Verbosity, commands};

pub(crate) fn run(command: Commands, verbosity: Verbosity) -> Result<()> {
    match command {
        Commands::AcStatus { summary, json, ac, check, require_coverage } => {
            commands::ac_status::run(commands::ac_status::AcStatusArgs {
                verbosity,
                summary,
                json,
                filter_ac: ac,
                check,
                require_coverage,
                ..Default::default()
            })
        }
        Commands::AcNew { ac_id, description, story, requirement } => {
            commands::ac_new::run(&ac_id, &description, &story, &requirement)
        }
        Commands::AcCoverage { todo, must_have } => {
            commands::ac_coverage::run(commands::ac_coverage::AcCoverageArgs {
                todo_only: todo,
                must_have_only: must_have,
                ..Default::default()
            })
        }
        Commands::AcSuggestScenarios { ac_id } => commands::ac_suggest_scenarios::run(
            commands::ac_suggest_scenarios::AcSuggestScenariosArgs { ac_id },
        ),
        Commands::AcTests { ac_id } => commands::ac_tests::run(&ac_id),
        Commands::TestAc { ac_id } => commands::test_ac::run(&ac_id),
        Commands::AdrCheck => commands::adr_check::run(commands::adr_check::AdrCheckArgs {
            verbosity,
            ..Default::default()
        }),
        Commands::AdrNew { title } => commands::adr_new::run(&title),
        Commands::Check => commands::check::run(),
        Commands::Precommit { mode, staged_only } => commands::precommit::run(&mode, staged_only),
        Commands::Bdd => commands::bdd::run(),
        Commands::AcReport { must_have, status, by_story, format } => {
            commands::ac_report::run(commands::ac_report::AcReportArgs {
                must_have,
                status,
                by_story,
                format,
            })
        }
        Commands::AcHistory { dir, format, must_have } => {
            commands::ac_history::run(commands::ac_history::AcHistoryArgs {
                dir: std::path::PathBuf::from(dir),
                format,
                must_have,
            })
        }
        Commands::AcSlo { dir, min_coverage, max_blockers, max_unknown, format } => {
            commands::ac_slo::run(commands::ac_slo::AcSloArgs {
                dir: std::path::PathBuf::from(dir),
                min_coverage,
                max_blockers,
                max_unknown,
                format,
            })
        }
        Commands::AcEnsureKernelMapped { strict } => commands::ac_ensure_kernel_mapped::run(
            commands::ac_ensure_kernel_mapped::AcEnsureKernelMappedArgs {
                verbose: verbosity.is_verbose(),
                strict,
            },
        ),
        Commands::AcLint { strict, check_files } => {
            commands::ac_lint::run(commands::ac_lint::AcLintArgs {
                verbose: verbosity.is_verbose(),
                strict,
                check_files,
            })
        }
        Commands::Bundle { task } => commands::bundle::run(&task),
        Commands::Audit => commands::audit::run(),
        Commands::Coverage => commands::coverage::run(),
        Commands::BuildTimeCapture => commands::build_time::run_capture(),
        Commands::BuildTimeCompare { baseline, current } => {
            commands::build_time::run_compare(&baseline, &current)
        }
        Commands::Clean => commands::clean::run(),
        Commands::CiLocal => commands::ci_local::run(),
        Commands::Deploy { env } => commands::deploy::run(&env),
        Commands::DesignNew { id, title, requirements, adrs, owner } => {
            commands::design_new::run(commands::design_new::DesignNewArgs {
                id,
                title,
                requirements,
                adrs,
                owner,
            })
        }
        Commands::Doctor => commands::doctor::run(),
        Commands::DocsCheck => commands::docs_check::run(),
        Commands::DocsFrontmatterSync { fix } => commands::docs_frontmatter_sync::run(fix),
        Commands::Spellcheck => commands::spellcheck::run_with_default_targets(),
        Commands::ContractsCheck => commands::contracts::check(),
        Commands::ContractsFmt => commands::contracts::fmt(),
        Commands::UiContractCheck => commands::ui_contract_check::run(),
        Commands::GraphExport { format, check, report_format } => {
            commands::graph_export::run(commands::graph_export::GraphExportArgs {
                format,
                check,
                report_format,
            })
        }
        Commands::TaskCreate { id, title, requirement, acs, owner, status, labels } => {
            commands::tasks::create_task(&id, &title, &requirement, &acs, owner, status, &labels)
        }
        Commands::TaskUpdate { id, title, owner, status } => {
            commands::tasks::update_task(&id, title, owner, status)
        }
        Commands::TasksList => commands::tasks_list::run(),
        Commands::ToolsChecksumUpdate => commands::tools_checksum_update::run(),
        Commands::ToolsChecksumVerify => commands::tools_checksum_verify::run(),
        Commands::FmtAll => commands::fmt_all::run(),
        Commands::Hakari => commands::hakari::run(),
        Commands::Migrate => commands::migrate::run(),
        Commands::PinActions => commands::pin_actions::run(),
        Commands::PolicyTest => commands::policy_test::run().map_err(|e| anyhow::anyhow!("{}", e)),
        Commands::Quickstart => commands::quickstart::run(),
        Commands::ReleasePrepare { version, dry_run } => {
            commands::release_prepare::run(&version, dry_run)
        }
        Commands::ReleaseBundle { version } => commands::release_bundle::run(&version),
        Commands::KernelPack { output_dir } => commands::kernel_pack::run_pack(&output_dir),
        Commands::KernelCheck { manifest } => commands::kernel_pack::run_check(manifest.as_deref()),
        Commands::ReleaseVerify => commands::release_verify::run(),
        Commands::SbomLocal => commands::sbom_local::run(),
        Commands::PublishCheck { crate_name, dry_run } => {
            commands::publish_check::run(commands::publish_check::PublishCheckArgs {
                crate_name,
                dry_run,
            })
        }
        Commands::PrCover { pr, run_dir, output, description } => {
            commands::pr_cover::run(commands::pr_cover::PrCoverArgs {
                pr,
                run_dir,
                output,
                description,
            })
        }
        Commands::PrUpdate { pr, run_dir, description, save_exhibit, dry_run } => {
            commands::pr_update::run(commands::pr_update::PrUpdateArgs {
                pr,
                run_dir,
                description,
                save_exhibit,
                dry_run,
            })
        }
        Commands::ReceiptsGate { pr, output_dir } => {
            commands::receipts::run_gate(commands::receipts::ReceiptsGateArgs {
                pr,
                output_dir,
                run_id: None,
            })
        }
        Commands::ReceiptsEconomics {
            pr,
            output_dir,
            author_minutes,
            author_confidence,
            review_minutes,
            review_confidence,
            interventions,
            compute_usd,
            compute_confidence,
            runs,
            failed_gates,
            fix_loops,
            uncertainty_reduced,
            rework_prevented,
            devlt_notes,
            compute_notes,
            iteration_notes,
        } => commands::receipts::run_economics(commands::receipts::ReceiptsEconomicsArgs {
            pr,
            output_dir,
            author_minutes,
            author_confidence,
            review_minutes,
            review_confidence,
            interventions,
            compute_usd,
            compute_confidence,
            runs,
            failed_gates,
            fix_loops,
            uncertainty_reduced,
            rework_prevented,
            devlt_notes,
            compute_notes,
            iteration_notes,
            run_id: None,
        }),
        Commands::ReceiptsValidate { dir, schema_dir } => {
            commands::receipts::run_validate(commands::receipts::ReceiptsValidateArgs {
                run_dir: dir,
                schema_dir,
            })
        }
        Commands::ReceiptsQuality {
            pr,
            output_dir,
            base_branch,
            boundary_rating,
            test_depth_rating,
            notes,
            llm,
            historian_output,
            historian_cmd,
        } => commands::receipts::run_quality(commands::receipts::ReceiptsQualityArgs {
            pr,
            output_dir,
            base_branch,
            boundary_rating,
            test_depth_rating,
            notes,
            run_id: None,
            llm,
            historian_output,
            historian_cmd,
        }),
        Commands::ReceiptsTelemetry { pr, output_dir, profile, base_branch } => {
            commands::receipts::run_telemetry(commands::receipts::ReceiptsTelemetryArgs {
                pr,
                output_dir,
                profile,
                base_branch,
                run_id: None,
            })
        }
        Commands::ReceiptsTimeline {
            pr,
            output_dir,
            base_branch,
            session_gap_minutes,
            exclude_prefixes,
            include_ephemeral,
        } => commands::receipts::run_timeline(commands::receipts::ReceiptsTimelineArgs {
            pr,
            output_dir,
            base_branch,
            session_gap_minutes,
            run_id: None,
            exclude_prefixes,
            include_ephemeral,
        }),
        Commands::ReceiptsForensic {
            pr,
            profile,
            base_branch,
            output_dir,
            exclude_prefixes,
            include_ephemeral,
            llm,
            historian_output,
            historian_cmd,
        } => commands::receipts::run_forensic(commands::receipts::ReceiptsForensicArgs {
            pr,
            output_dir,
            base_branch,
            profile,
            session_gap_minutes: 30,
            exclude_prefixes,
            include_ephemeral,
            llm,
            historian_output,
            historian_cmd,
        }),
        Commands::SuggestNext(args) => commands::suggest_next::run(args),
        Commands::Selftest => commands::selftest::run_with_verbosity(verbosity),
        Commands::KernelSmoke => commands::kernel_smoke::run(),
        Commands::KernelStatus => commands::kernel_status::run(),
        Commands::Status => commands::status::run(),
        Commands::FrictionList { status, severity, json } => {
            commands::friction::list_friction_entries(status.as_deref(), severity.as_deref(), json)
        }
        Commands::FrictionNew {
            category,
            severity,
            summary,
            description,
            flow,
            phase,
            discovered_by,
            refs,
        } => commands::friction::create_friction_entry(
            &category,
            &severity,
            &summary,
            description.as_deref(),
            flow.as_deref(),
            phase.as_deref(),
            discovered_by.as_deref(),
            &refs,
        ),
        Commands::FrictionResolve {
            id,
            resolved_by,
            fix_description,
            pr_links,
            verification,
            status,
        } => commands::friction::resolve_friction_entry(
            &id,
            &resolved_by,
            fix_description.as_deref(),
            &pr_links,
            verification.as_deref(),
            &status,
        ),
        Commands::FrictionGhCreate { friction_id, labels, dry_run, open } => {
            commands::friction::gh_create_issue(&friction_id, labels.as_deref(), dry_run, open)
        }
        Commands::FrictionGhLink { friction_id, issue_number } => {
            commands::friction::gh_link_issue(&friction_id, &issue_number)
        }
        Commands::ForkList { status, domain, json } => {
            commands::fork::list_fork_entries(status.as_deref(), domain.as_deref(), json)
        }
        Commands::ForkRegister {
            name,
            domain,
            kernel_version,
            url,
            maintainer_name,
            maintainer_contact,
            status,
            notes,
        } => commands::fork::create_fork_entry(commands::fork::CreateForkArgs {
            name: &name,
            domain: &domain,
            kernel_version: &kernel_version,
            url: url.as_deref(),
            maintainer_name: maintainer_name.as_deref(),
            maintainer_contact: maintainer_contact.as_deref(),
            status: status.as_deref(),
            notes: notes.as_deref(),
        }),
        Commands::QuestionsList { status, json } => {
            commands::questions::list_questions(status.as_deref(), json)
        }
        Commands::QuestionNew {
            category,
            summary,
            flow,
            phase,
            description,
            created_by,
            task_id,
            refs,
        } => commands::questions::create_question(
            &category,
            &summary,
            &flow,
            &phase,
            &description,
            &created_by,
            task_id.as_deref(),
            &refs,
        ),
        Commands::QuestionResolve { id, resolved_by, chosen_option, notes, status } => {
            commands::questions::resolve_question(
                &id,
                &resolved_by,
                chosen_option.as_deref(),
                notes.as_deref(),
                &status,
            )
        }
        Commands::IssuesSearch { query, type_filter, status, refs, json, limit } => {
            commands::issues_search::search_issues(
                &query,
                type_filter.as_deref(),
                status.as_deref(),
                refs.as_deref(),
                json,
                limit,
            )
        }
        Commands::ServiceDescriptor { format } => commands::service_descriptor::run(&format)
            .map_err(|e| anyhow::anyhow!("service-descriptor failed: {}", e)),
        Commands::ServiceInit { id, name, description, tags, register_fork } => {
            commands::service_init::run(commands::service_init::ServiceInitArgs {
                id,
                name,
                description,
                tags: if tags.is_empty() { None } else { Some(tags) },
                register_fork,
            })
        }
        Commands::ConfigValidate { env } => commands::config_validate::run(&env)
            .map_err(|e| anyhow::anyhow!("config-validate failed: {}", e)),
        Commands::HelpFlows => commands::help_flows::run(),
        Commands::InstallHooks => commands::install_hooks::run(),
        Commands::DevUp => commands::dev_up::run(),
        Commands::SkillsFmt => commands::skills::run_fmt(),
        Commands::SkillsLint => commands::skills::run_lint(),
        Commands::AgentsFmt => commands::agents::run_fmt(),
        Commands::AgentsLint => commands::agents::run_lint(),
        Commands::TestChanged { base, plan_only } => {
            commands::test_changed::run(commands::test_changed::TestChangedArgs { base, plan_only })
        }
        Commands::CheckApiDiff { adr } => {
            commands::check_api_diff::run(commands::check_api_diff::CheckApiDiffArgs { adr })
        }
        Commands::CheckOpenapiDiff => commands::check_openapi_diff::run(),
        Commands::CheckJsonSchemas { generate } => {
            commands::check_json_schemas::run(commands::check_json_schemas::CheckJsonSchemasArgs {
                generate,
            })
        }
        Commands::CheckLayering => commands::check_layering::run(),
        Commands::Version { json } => {
            commands::version::run(commands::version::VersionArgs { json })
        }
        Commands::VersionCheck { json } => {
            commands::version_check::run(commands::version_check::VersionCheckArgs { json })
        }
        Commands::EnvMode { json } => {
            commands::env_mode::run(commands::env_mode::EnvModeArgs { json })
        }
        Commands::IdpSnapshot { output, pretty } => {
            commands::idp_snapshot::run(commands::idp_snapshot::IdpSnapshotArgs { output, pretty })
        }
        Commands::IdpCheck => commands::idp_check::run(),
    }
}
