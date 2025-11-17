#!/usr/bin/env bash
set -euo pipefail

# Migrate development database using sqlx
# Expects DATABASE_URL env var

SCRIPT_DIR="$(cd "$(dirname "${B0}")" && pwd)"
CRATE_DIR="$SCRIPT_DIR/../crates/adapters-db-sqlx"

echo "🚀 Running database migrations for dev environment..."

# Run sqlx migrations
echo "📦 Running sqlx migrate run..."
cd "$CRATE_DIR"
DATABASE_URL="$DATABASE_URL" cargo sqlx migrate run --source migrations

echo "✅ Migrations completed successfully!"
echo "   DATABASE_URL: $DATABASE_URL"

# Optional: Generate sqlx types (if sqlx CLI available)
if command -v cargo-sqlx &> /dev/null; then
    echo "🔄 Regenerating sqlx types..."
    cargo sqlx prepare -- --lib
fi