# Architecture

This document describes the architecture of the Backstage integration with the Rust-as-Spec platform cell.

## Overview

The Backstage plugin provides a developer portal interface that integrates with the platform's governance and observability systems.

## Components

- **Catalog Entity** - Backstage component definition (`catalog-info.yaml`)
- **TechDocs** - Documentation built with MkDocs and served through Backstage
- **Platform APIs** - HTTP endpoints exposed at `/platform/*` for governance status

## Integration Points

The Backstage integration connects to:

1. Platform status endpoint (`/platform/status`)
2. Governance graph endpoint (`/platform/graph`)
3. AC coverage endpoint (`/platform/coverage`)

## Next Steps

Architecture details will be expanded as Backstage features are implemented.
