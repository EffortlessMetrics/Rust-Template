# testing

Shared testing utilities for safe process-global state manipulation.

## What It Is

`testing` provides RAII (Resource Acquisition Is Initialization) guards for environment variable and working directory manipulation in tests. These guards:

- Serialize access via a global lock (preventing concurrent mutation)
- Snapshot state before modification
- Restore state on Drop (panic-safe)

### Why This Exists

In Rust 2024 edition, [`std::env::set_var`] and [`std::env::remove_var`] are `unsafe` because environment variables are process-global state. Concurrent reads during mutation cause undefined behavior.

This crate centralizes the unsafe operations behind safe interfaces with proper synchronization.

### What It Owns

| Module | Responsibility |
|--------|----------------|
| `process` | RAII guards for environment variables and working directory |
| `fakes` | Scanner-safe fake secrets for test code |

### What It Is Not

- **Not a test framework**: Use `cucumber` or standard `#[test]` for test execution
- **Not a mocking library**: Use `mockall` or similar for mocking
- **Not production code**: This crate is for tests only (`publish = false`)

## Process Guards

### EnvVarGuard

RAII guard for safe environment variable manipulation:

```rust
use testing::process::EnvVarGuard;

#[test]
fn test_with_custom_env() {
    let guard = EnvVarGuard::new(&["MY_VAR", "OTHER_VAR"]);

    guard.set("MY_VAR", "test-value");
    guard.set("OTHER_VAR", "another-value");

    // Test code here...

} // MY_VAR and OTHER_VAR automatically restored to original values
```

#### Methods

| Method | Description |
|---------|-------------|
| `new(&[&str])` | Create guard for specified environment variables |
| `set(&str, &str)` | Set environment variable value |
| `remove(&str)` | Remove environment variable |

#### Thread Safety

The guard uses a global lock (`parking_lot::Mutex`) to serialize access, preventing race conditions between concurrent test execution.

### CwdGuard

RAII guard for safe working directory manipulation:

```rust
use testing::process::CwdGuard;
use std::path::Path;

#[test]
fn test_in_custom_dir() {
    let original = std::env::current_dir().unwrap();
    let new_dir = Path::new("/tmp/test-dir");

    let _guard = CwdGuard::new(&new_dir);

    // Current directory is now /tmp/test-dir
    assert_eq!(std::env::current_dir().unwrap(), new_dir);

} // Working directory automatically restored
```

## Fake Secrets

The `fakes` module provides deterministic, low-entropy fake secrets that won't trigger secret scanners (GitHub, gitleaks, etc.).

### Design Principles

- **Low entropy**: Human-obvious, no random UUIDs/hashes/base64
- **No vendor patterns**: Never use `sk_live_`, `AKIA`, `ghp_`, `xoxb-`, etc.
- **Deterministic**: Same input produces same output (no randomness)
- **Clearly fake**: `EXAMPLE_*_DO_NOT_USE` pattern is unambiguous

### Example Secret

```rust
use testing::fakes::example_secret;

let secret = example_secret("database_password");
assert_eq!(secret, "EXAMPLE_DATABASE_PASSWORD_DO_NOT_USE");
```

### Example Token

```rust
use testing::fakes::example_token;

let token = example_token("platform-auth");
assert_eq!(token, "EXAMPLE_TOKEN_PLATFORM_AUTH_0000");
```

### Example Secret with ID

```rust
use testing::fakes::example_secret_for;

let secret1 = example_secret_for("api-key", "test-1");
let secret2 = example_secret_for("api-key", "test-2");

assert_eq!(secret1, "EXAMPLE_API_KEY_TEST_1_DO_NOT_USE");
assert_eq!(secret2, "EXAMPLE_API_KEY_TEST_2_DO_NOT_USE");
```

### Example API Key

```rust
use testing::fakes::example_api_key;

let key = example_api_key("stripe");
assert_eq!(key, "EXAMPLE_APIKEY_STRIPE_00000000");
```

### Example Database URL

```rust
use testing::fakes::example_database_url;

let url = example_database_url("postgres");
assert_eq!(url, "postgresql://EXAMPLE_USER:EXAMPLE_PASS@example.internal:5432/EXAMPLE_DB");
```

## Usage Examples

### Testing with Environment Variables

```rust
use testing::process::EnvVarGuard;

#[test]
fn test_config_loading() {
    let guard = EnvVarGuard::new(&["APP_CONFIG", "APP_ENV"]);

    guard.set("APP_CONFIG", "/path/to/config.yaml");
    guard.set("APP_ENV", "test");

    // Load config with test values
    let config = load_config().unwrap();
    assert_eq!(config.path, "/path/to/config.yaml");

    // Variables restored on drop
}
```

### Testing with Multiple Secrets

```rust
use testing::fakes::{example_secret, example_token, example_api_key};

#[test]
fn test_authentication() {
    let jwt_secret = example_secret("jwt");
    let api_token = example_token("platform");
    let stripe_key = example_api_key("stripe");

    // Use fakes in test
    let result = authenticate(&jwt_secret, &api_token, &stripe_key);
    assert!(result.is_ok());

    // No scanner warnings for these values
}
```

### Parallel Test Safety

The global lock ensures parallel tests don't interfere:

```rust
#[test]
fn test_parallel_safe_1() {
    let guard = EnvVarGuard::new(&["SHARED_VAR"]);
    guard.set("SHARED_VAR", "value1");
    // Safe even if test_parallel_safe_2 runs concurrently
}

#[test]
fn test_parallel_safe_2() {
    let guard = EnvVarGuard::new(&["SHARED_VAR"]);
    guard.set("SHARED_VAR", "value2");
    // Lock ensures serialization
}
```

## Banned Patterns

The fake generators never produce these vendor-specific patterns:

| Pattern | Vendor | Why Banned |
|----------|---------|-------------|
| `sk_live_` | Stripe | Production secret prefix |
| `sk_test_` | Stripe | Production secret prefix |
| `AKIA` | AWS | Access key ID prefix |
| `ghp_` | GitHub | Personal access token prefix |
| `gho_` | GitHub | OAuth token prefix |
| `github_pat_` | GitHub | Personal access token prefix |
| `xoxb-` | Slack | Bot token prefix |
| `xoxp-` | Slack | User token prefix |

## Consumers

This crate is used by:

| Consumer | Usage |
|----------|-------|
| `telemetry` | Environment variable tests |
| `acceptance` | Environment and working directory guards |
| `xtask` | Test utilities for xtask commands |

## See Also

- [`telemetry/README.md`](../telemetry/README.md) - Uses this crate for env var tests
- [`acceptance/README.md`](../acceptance/README.md) - Uses guards for test isolation
