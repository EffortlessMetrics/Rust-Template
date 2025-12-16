# Terraform configuration for Rust-as-Spec Platform
# This configuration aligns with specs/config_schema.yaml

terraform {
  required_version = ">= 1.0"
}

# HTTP Configuration
variable "http_port" {
  type        = number
  default     = 8080
  description = "HTTP listen port"
}

# Telemetry Configuration
variable "telemetry_otlp_endpoint" {
  type        = string
  default     = ""
  description = "OTLP collector URL"
}

# Platform Configuration
variable "platform_auth_mode" {
  type        = string
  default     = "open"
  description = "Authentication mode for platform endpoints (open|basic)"
}

variable "platform_redact_secrets" {
  type        = bool
  default     = true
  description = "Whether to redact secrets from status/UI output"
}

# Database Configuration
variable "database_auto_migrate" {
  type        = bool
  default     = false
  description = "Whether to automatically run database migrations on startup"
}

# Secrets (should be provided via environment or secure backend)
variable "db_url" {
  type        = string
  sensitive   = true
  description = "Postgres connection string"
}

variable "auth_jwt_signing_key" {
  type        = string
  sensitive   = true
  description = "JWT signing key for authentication"
}

variable "platform_auth_token" {
  type        = string
  default     = ""
  sensitive   = true
  description = "Shared secret for platform basic auth mode"
}

variable "platform_jwt_secret" {
  type        = string
  default     = ""
  sensitive   = true
  description = "JWT secret key for platform JWT auth mode"
}
