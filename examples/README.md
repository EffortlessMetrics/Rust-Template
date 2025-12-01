# Examples

This directory contains example implementations and integrations for the Rust-as-Spec platform template.

**IMPORTANT**: All examples are marked as **EXAMPLE QUALITY** and are **NOT PRODUCTION READY**. They are provided as reference implementations to demonstrate integration patterns and best practices. Review, test, and harden before production use.

## Available Examples

### [backstage-plugin](./backstage-plugin/)

A minimal Backstage plugin demonstrating integration with the platform's governance APIs.

**What it shows**:
- React components for displaying governance health and documentation metrics
- TypeScript API client for `/platform/status` and `/platform/docs/index` endpoints
- Backstage plugin structure and configuration patterns
- Proxy setup for backend communication

**Use case**: Building internal developer portals with visibility into platform governance.

**Technologies**: TypeScript, React, Backstage, Material-UI

---

### [agent-pilot](./agent-pilot/)

Python-based agent pilot demonstrating autonomous workflow execution using platform APIs.

**What it shows**:
- How to query `/platform/agent/hints` for prioritized work
- Task selection and execution workflows
- State management across agent sessions
- Bundle-based context building

**Use case**: Building autonomous agents that can work within governed boundaries.

**Technologies**: Python

---

### [brownfield-demo](./brownfield-demo/)

Demonstration of adopting the Rust-as-Spec template in an existing (brownfield) codebase.

**What it shows**:
- Incremental adoption strategy
- Retrofitting governance to existing code
- Migration path from unstructured to governed development
- Handling legacy code and technical debt

**Use case**: Adopting the template in existing projects without rewriting everything.

**Technologies**: Rust

---

### [fork-customization](./fork-customization/)

Guide and examples for forking and customizing the template for your organization.

**What it shows**:
- How to modify the template for organizational needs
- Safe customization points vs. structural contracts
- Maintaining upstream compatibility
- Custom skill and workflow patterns

**Use case**: Creating organization-specific variants of the template.

**Technologies**: Documentation, Configuration

---

## Using These Examples

### General Guidelines

1. **Read the README first**: Each example has its own README with detailed setup instructions.

2. **Understand example quality**: These are not production-ready. They demonstrate patterns but skip:
   - Comprehensive error handling
   - Security hardening (auth, rate limiting, input validation)
   - Performance optimization
   - Production monitoring and observability
   - Comprehensive testing

3. **Copy, don't link**: Copy example code into your project and adapt it. Don't depend on these examples directly.

4. **Test thoroughly**: Examples demonstrate happy paths. Add error handling, edge case handling, and tests.

5. **Follow security best practices**:
   - Never commit secrets
   - Use HTTPS in production
   - Implement authentication and authorization
   - Validate and sanitize all inputs
   - Add rate limiting and monitoring

### Prerequisites

Each example may have different prerequisites:

- **backstage-plugin**: Node.js 18+, Backstage instance
- **agent-pilot**: Python 3.11+, platform service running
- **brownfield-demo**: Rust 1.75+, existing Rust project
- **fork-customization**: Git, basic understanding of the template

Check each example's README for specific requirements.

### Platform Service Required

Most examples require a running instance of the Rust-as-Spec platform service:

```bash
# In the template repository root
cargo run -p app-http

# Service runs on http://localhost:8080
# Platform APIs available at /platform/*
```

Verify it's running:

```bash
curl http://localhost:8080/platform/status
```

## Example Quality Statement

These examples are provided **as-is** for **reference and learning purposes**. They demonstrate:

✅ **Integration patterns** - How to connect to platform APIs
✅ **Component structure** - How to organize code
✅ **Type definitions** - Data models for API responses
✅ **Basic workflows** - Happy path implementations

They intentionally omit:

❌ **Production security** - Auth, secrets management, input validation
❌ **Error resilience** - Retry logic, circuit breakers, graceful degradation
❌ **Performance optimization** - Caching, connection pooling, request batching
❌ **Comprehensive testing** - Unit tests, integration tests, E2E tests
❌ **Observability** - Logging, metrics, tracing, alerting

### Before Production Use

1. **Security audit**: Review for security vulnerabilities
2. **Error handling**: Add comprehensive error handling and recovery
3. **Testing**: Add unit, integration, and E2E tests
4. **Documentation**: Document APIs, configuration, and deployment
5. **Monitoring**: Add logging, metrics, and alerting
6. **Performance**: Profile and optimize critical paths
7. **Review**: Have security and SRE teams review

## Contributing Examples

Want to add an example? Follow these guidelines:

1. **Create a descriptive directory name** (kebab-case)
2. **Include a comprehensive README.md** with:
   - What the example demonstrates
   - Prerequisites and dependencies
   - Setup and installation instructions
   - Usage examples
   - Production considerations
   - Links to relevant docs
3. **Mark as EXAMPLE QUALITY** prominently
4. **Include all necessary configuration files**
5. **Add to this top-level README**

### Example Template Structure

```
examples/
└── my-example/
    ├── README.md              # Comprehensive guide
    ├── .gitignore             # Ignore build artifacts
    ├── src/                   # Source code
    │   └── ...
    ├── config.example.yaml    # Example configuration
    └── tests/                 # Optional: example tests
        └── ...
```

## Additional Resources

- [Platform API Documentation](../docs/AGENT_GUIDE.md)
- [Governance Model](../docs/explanation/TEMPLATE-CONTRACTS.md)
- [Skills Development](../docs/SKILLS_GOVERNANCE.md)
- [Agent Development](../docs/AGENTS_GOVERNANCE.md)

## License

Examples follow the same license as the main template (Apache-2.0). See [LICENSE](../LICENSE) for details.

---

**Remember**: These are learning resources and reference implementations. Always review and adapt for your specific needs and security requirements.
