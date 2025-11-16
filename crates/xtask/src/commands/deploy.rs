use anyhow::{Context, Result};
use colored::Colorize;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Supported deployment environments
#[derive(Debug, Clone, Copy)]
pub enum Environment {
    Dev,
    Staging,
    Prod,
}

impl Environment {
    pub fn as_str(&self) -> &str {
        match self {
            Environment::Dev => "dev",
            Environment::Staging => "staging",
            Environment::Prod => "prod",
        }
    }

    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "dev" | "development" => Ok(Environment::Dev),
            "staging" | "stage" => Ok(Environment::Staging),
            "prod" | "production" => Ok(Environment::Prod),
            _ => anyhow::bail!("Unknown environment: {}. Valid options: dev, staging, prod", s),
        }
    }
}

/// Deploy application to specified environment
pub fn run(env: &str) -> Result<()> {
    let environment = Environment::from_str(env)?;

    println!("{} Preparing deployment to {} environment\n", "→".cyan(), environment.as_str());

    // Get workspace root
    let workspace_root = get_workspace_root()?;

    // Validate manifests exist
    validate_manifests(&workspace_root, &environment)?;

    // Check prerequisites
    check_prerequisites(&environment)?;

    // Display next steps
    display_next_steps(&environment);

    Ok(())
}

/// Validate that manifests exist for the target environment
fn validate_manifests(workspace_root: &Path, env: &Environment) -> Result<()> {
    let manifests_dir = workspace_root.join("infra").join("k8s").join(env.as_str());

    if !manifests_dir.exists() {
        anyhow::bail!(
            "Manifests directory not found: {}\n\
            \n\
            Create manifests for {} environment in this directory.",
            manifests_dir.display(),
            env.as_str()
        );
    }

    println!("{} Found manifests directory: {}", "✓".green(), manifests_dir.display());

    // Check for kustomization.yaml (for staging/prod) or raw manifests (for dev)
    let kustomization_file = manifests_dir.join("kustomization.yaml");
    if kustomization_file.exists() {
        println!("{} Found Kustomize overlay: kustomization.yaml", "✓".green());
    } else {
        // Check for expected base manifest files (dev environment)
        let expected_files = ["deployment.yaml", "service.yaml"];
        let mut found_files = Vec::new();
        let mut missing_files = Vec::new();

        for file in &expected_files {
            let file_path = manifests_dir.join(file);
            if file_path.exists() {
                found_files.push(*file);
            } else {
                missing_files.push(*file);
            }
        }

        if !found_files.is_empty() {
            println!("{} Found base manifest files:", "✓".green());
            for file in &found_files {
                println!("  • {}", file);
            }
        }

        if !missing_files.is_empty() {
            println!("{} Missing optional manifests:", "⚠".yellow());
            for file in &missing_files {
                println!("  • {}", file);
            }
        }
    }

    println!();
    Ok(())
}

/// Check deployment prerequisites
fn check_prerequisites(env: &Environment) -> Result<()> {
    println!("{} Checking prerequisites:", "→".cyan());

    // Check Docker
    let docker_available = check_command_available("docker", &["--version"]);
    if docker_available {
        println!("{} Docker is available", "✓".green());
    } else {
        println!("{} Docker not found (required for building images)", "✗".red());
    }

    // Check kubectl
    let kubectl_available = check_command_available("kubectl", &["version", "--client"]);
    if kubectl_available {
        println!("{} kubectl is available", "✓".green());

        // Try to get cluster info
        if check_command_available("kubectl", &["cluster-info"]) {
            println!("{} Connected to Kubernetes cluster", "✓".green());
        } else {
            println!("{} Not connected to a Kubernetes cluster", "⚠".yellow());
        }
    } else {
        println!("{} kubectl not found (required for deployment)", "✗".red());
    }

    // Environment-specific checks
    match env {
        Environment::Dev => {
            // Check if minikube or kind is available
            let minikube_available = check_command_available("minikube", &["status"]);
            let kind_available = check_command_available("kind", &["get", "clusters"]);

            if minikube_available {
                println!("{} minikube detected", "✓".green());
            } else if kind_available {
                println!("{} kind detected", "✓".green());
            } else {
                println!(
                    "{} No local cluster detected (minikube/kind). You may need to load images manually.",
                    "⚠".yellow()
                );
            }
        }
        Environment::Staging | Environment::Prod => {
            println!("{} Ensure you're connected to the correct cluster", "⚠".yellow());
        }
    }

    println!();
    Ok(())
}

/// Check if a command is available
fn check_command_available(cmd: &str, args: &[&str]) -> bool {
    Command::new(cmd).args(args).output().map(|output| output.status.success()).unwrap_or(false)
}

/// Display next steps for deployment
fn display_next_steps(env: &Environment) {
    println!("{}", "Next Steps:".bold());
    println!();

    match env {
        Environment::Dev => {
            println!("1. Build the Docker image:");
            println!(
                "   {}",
                "docker build -t app-http:latest -f crates/app-http/Dockerfile .".cyan()
            );
            println!();

            println!("2. Load image into your local cluster:");
            println!("   For minikube: {}", "minikube image load app-http:latest".cyan());
            println!("   For kind:     {}", "kind load docker-image app-http:latest".cyan());
            println!("   For Docker Desktop: (skip this step)");
            println!();

            println!("3. Apply Kubernetes manifests:");
            println!("   {}", format!("kubectl apply -k infra/k8s/{}/", env.as_str()).cyan());
            println!();

            println!("4. Verify deployment:");
            println!("   {}", "kubectl get pods -l app=app-http".cyan());
            println!("   {}", "kubectl get service app-http".cyan());
            println!();

            println!("5. Access the application:");
            println!("   {}", "kubectl port-forward service/app-http 8080:80".cyan());
            println!("   Then visit: {}", "http://localhost:8080/health".cyan());
        }
        Environment::Staging | Environment::Prod => {
            println!("1. Build and push Docker image to registry:");
            println!(
                "   {}",
                "docker build -t <registry>/app-http:<version> -f crates/app-http/Dockerfile ."
                    .cyan()
            );
            println!("   {}", "docker push <registry>/app-http:<version>".cyan());
            println!();

            println!("2. Update image tag in Kustomize overlay:");
            println!(
                "   {}",
                format!("# Edit infra/k8s/{}/kustomization.yaml", env.as_str()).cyan()
            );
            println!("   {}", "# Update images.newTag: <version>".cyan());
            println!("   Or use: {}", format!("cd infra/k8s/{} && kustomize edit set image app-http=<registry>/app-http:<version>", env.as_str()).cyan());
            println!();

            println!("3. Verify you're connected to the correct cluster:");
            println!("   {}", "kubectl config current-context".cyan());
            println!();

            println!("4. Preview changes (optional):");
            println!("   {}", format!("kubectl kustomize infra/k8s/{}/", env.as_str()).cyan());
            println!();

            println!("5. Apply Kubernetes manifests:");
            println!("   {}", format!("kubectl apply -k infra/k8s/{}/", env.as_str()).cyan());
            println!();

            println!("6. Monitor rollout:");
            println!("   {}", "kubectl rollout status deployment/app-http".cyan());
            println!("   {}", "kubectl get pods -l app=app-http -w".cyan());
        }
    }

    println!();
    println!("{}", "Additional Commands:".bold());
    println!("  Validate policies: {}", "cargo xtask policy-test".cyan());
    println!("  View logs:         {}", "kubectl logs -l app=app-http -f".cyan());
    println!("  Describe pod:      {}", "kubectl describe pod -l app=app-http".cyan());
    println!();

    println!("{}", "Documentation:".bold());
    println!("  See docs/how-to/deploy-dev.md for detailed deployment guide");
    println!();
}

/// Get workspace root directory
fn get_workspace_root() -> Result<PathBuf> {
    let current_dir = std::env::current_dir().context("Failed to get current directory")?;

    // Try to find Cargo.toml in workspace root
    let mut check_dir = current_dir.clone();
    for _ in 0..3 {
        let cargo_toml = check_dir.join("Cargo.toml");
        if cargo_toml.exists() {
            // Verify it's the workspace root by checking for infra directory
            if check_dir.join("infra").exists() {
                return Ok(check_dir);
            }
        }
        if let Some(parent) = check_dir.parent() {
            check_dir = parent.to_path_buf();
        } else {
            break;
        }
    }

    // Fallback: assume we're running from workspace root
    Ok(current_dir)
}
