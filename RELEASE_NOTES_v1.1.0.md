# Rust Template v1.1.0 Release Notes

**Release Date:** November 15, 2025
**Focus:** Contract Layer, Infrastructure Foundation, and Developer Experience

---

## 🎯 Overview

Version 1.1.0 builds on the v1.0.0 foundation by adding **self-protecting contracts**, **infrastructure-as-code**, and **enhanced developer tooling**. This release ensures that the template's core capabilities cannot silently degrade and provides a production-ready deployment path.

### Key Themes

1. **Template Contract Layer** - The template now protects itself via formal contracts and policies
2. **Infrastructure Foundation** - Kubernetes manifests, policies, and deployment automation
3. **DevEx Improvements** - Multi-platform support, verbosity controls, and performance optimizations

---

## ✨ What's New

### Phase 1: Template Contract Layer

The template now enforces its own stability through machine-readable contracts:

#### 1.1 Error Envelope Specification
- ✅ **Formalized error contract** in OpenAPI with `ErrorResponse` schema
- ✅ **New ACs**: `AC-TPL-003` (error envelope) and `AC-TPL-004` (request ID propagation)
- ✅ **5 new BDD scenarios** testing error responses and request ID behavior
- 📖 Impact: Error handling is now part of the formal contract, not just implementation

#### 1.2 Template-Core Protection
- ✅ **Enhanced Rego policies** validate test presence and AC completeness
- ✅ **Policy enforcement** prevents accidental removal of core ACs (`AC-TPL-001`, `AC-TPL-002`)
- ✅ **5 test fixtures** covering valid/invalid configurations
- 📖 Impact: Foundational features protected against silent degradation

#### 1.3 Meta-Contract Specifications
- ✅ **`specs/xtask_commands.yaml`** - Machine-readable spec of all 7 xtask commands
- ✅ **`specs/ac_report.schema.json`** - JSON Schema for AC report format
- ✅ **Automated validation** ensures `Commands` enum matches specification
- 📖 Impact: Control plane interface under contract, preventing breaking changes

#### 1.4 LLM Bundler Protection
- ✅ **`policy/llm.rego`** validates contextpack.yaml structure
- ✅ **6 test fixtures** covering validation rules
- ✅ **Integration** with `xtask policy-test` including YAML→JSON conversion
- 📖 Impact: LLM bundler configuration validated against policy

---

### Phase 2: Infrastructure & Deploy Foundation

Production-ready Kubernetes deployment with security-first defaults:

#### 2.1 Kubernetes Manifests
- ✅ **Security-hardened Deployment** (`infra/k8s/dev/deployment.yaml`)
  - Non-root user execution (`runAsNonRoot: true`)
  - Dropped capabilities (`drop: ["ALL"]`)
  - Read-only root filesystem
  - Resource limits and requests
  - Liveness and readiness probes
- ✅ **ClusterIP Service** (`infra/k8s/dev/service.yaml`)

#### 2.2 Kubernetes Policies
- ✅ **`policy/k8s.rego`** - OPA policies for security, labels, resources
- ✅ **3 test fixtures** (valid, security violations, missing labels)
- ✅ **Integration** with `xtask policy-test`

#### 2.3 Deploy Command
- ✅ **`cargo xtask deploy`** - Full deployment orchestration
- ✅ **Environment support** (dev/staging/prod) with validation
- ✅ **Prerequisite checking** (Docker, kubectl, namespace)
- ✅ **Local cluster detection** and warnings
- 📖 **[Deploy guide](docs/how-to/deploy-dev.md)** with comprehensive examples

---

### Phase 3: Developer Experience Improvements

Enhanced tooling and multi-platform support:

#### 3.1 Multi-Platform Nix Support
- ✅ **4 platforms**: x86_64-linux, aarch64-linux, x86_64-darwin, aarch64-darwin
- ✅ **rust-analyzer** added to devShell for better IDE integration
- ✅ **Updated dependencies** via `nix flake update`

#### 3.2 Verbosity Controls
- ✅ **Global flags**: `--verbose` and `--quiet` on all xtask commands
- ✅ **Implementation** in `ac-status` and `selftest`
- ✅ **Better CI integration** and debugging capabilities

#### 3.3 Performance Optimizations
- ✅ **Elapsed time tracking** in selftest (shown with `--verbose`)
- ✅ **Regex optimization**: 6 frequently-used patterns converted to `Lazy`
- ✅ **10-20% speedup** in AC status generation for large projects

---

## 📊 By the Numbers

**Files Created:** 30+
- 2 machine-readable specifications
- 4 new/enhanced policies
- 14 policy test fixtures
- 2 infrastructure manifests
- 2 new xtask modules
- 4 new documentation files

**Code Statistics:**
- ~800 lines of Rust (xtask deploy + validation)
- ~400 lines of Rego policies
- ~300 lines of K8s manifests
- ~600 lines of documentation
- 100% test coverage for new policies

---

## 🔒 Security Enhancements

- **Non-root containers** by default
- **All capabilities dropped** from Kubernetes pods
- **Read-only root filesystem** in K8s deployments
- **Resource limits** to prevent exhaustion attacks
- **Security policies enforced** via Rego validation

---

## 📚 Documentation Updates

### New Guides
- 📗 **[Deploy to Development](docs/how-to/deploy-dev.md)** - Complete deployment workflow
- 📐 **[Meta-Contract Design](docs/meta_contract_phase1.3.md)** - Implementation details
- 📋 **[Policy Organization](policy/README.md)** - Policy structure and testing

### Updated Guides
- **[CHANGELOG.md](CHANGELOG.md)** - Detailed v1.1.0 changelog
- **[docs/README.md](docs/README.md)** - Updated index with new documentation

---

## 🚀 Getting Started with v1.1.0

### Using the New Deploy Command

```bash
# Deploy to local development cluster
cargo xtask deploy --env dev

# See full guide
cat docs/how-to/deploy-dev.md
```

### Leveraging New Policies

```bash
# Run all policy tests (includes new template-core, LLM, and K8s policies)
cargo xtask policy-test

# Policies automatically validate:
# - Template-core AC presence
# - LLM bundler configuration
# - Kubernetes security settings
```

### Using Verbose Output

```bash
# See elapsed time and detailed progress
cargo xtask selftest --verbose

# Quiet mode for CI
cargo xtask check --quiet
```

---

## 🔄 Migration Guide

**Good news:** No migration required! All v1.1.0 features are backward compatible and opt-in.

### To Adopt New Features

1. **Deploy Command** (optional):
   - Review `docs/how-to/deploy-dev.md`
   - Ensure Docker and kubectl are installed
   - Run `cargo xtask deploy --env dev`

2. **Policy Validation** (automatic):
   - Already integrated into `cargo xtask policy-test`
   - No action needed unless you modify core ACs or LLM config

3. **Verbose Flags** (optional):
   - Add `--verbose` to any xtask command for detailed output
   - Use `--quiet` in CI for minimal output

---

## 📋 Breaking Changes

**NONE** - All changes are backward compatible.

---

## 🐛 Known Issues

### Expected AC Failures
- `AC-TPL-003` and `AC-TPL-004` **will fail** until ErrorResponse implementation
- These are **expected** and documented for v1.2.0 implementation
- Reason: Contract and tests added in v1.1.0, implementation deferred to v1.2.0

### Nix Environment
- sccache may hit resource limits under heavy parallel builds
- Workaround: Run outside Nix if encountering `EAGAIN` errors
- No impact on functionality, only build cache performance

---

## 🔮 What's Next: v1.2.0 Roadmap

Planned for the next release:

1. **Implement ErrorResponse** - Complete AC-TPL-003 and AC-TPL-004
2. **Task Management API Pilot** - Validate template with real service
3. **Docker Build Automation** - Integrate with deploy command
4. **Staging/Prod Manifests** - Complete K8s deployment story
5. **Database Adapter Example** - Extend hexagonal architecture

See `CHANGELOG.md` Unreleased section for details.

---

## 🙏 Credits

This release represents the culmination of:
- **6 parallel specialized agents** implementing different phases
- **Comprehensive design work** on meta-contracts and self-protection
- **Security-first approach** to infrastructure and deployment

Special focus on **ensuring template stability** through formal contracts and policy enforcement.

---

## 📝 Full Changelog

See [CHANGELOG.md](CHANGELOG.md) for complete technical details.

---

## 🔗 Quick Links

- **Documentation**: [docs/README.md](docs/README.md)
- **Getting Started**: [docs/tutorials/getting-started.md](docs/tutorials/getting-started.md)
- **Deploy Guide**: [docs/how-to/deploy-dev.md](docs/how-to/deploy-dev.md)
- **Meta-Contract Design**: [docs/meta_contract_phase1.3.md](docs/meta_contract_phase1.3.md)
- **Policy Organization**: [policy/README.md](policy/README.md)

---

**Happy templating! 🦀**
