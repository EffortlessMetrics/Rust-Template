# How-to: Change OpenAPI Safely

This guide explains how to evolve HTTP contracts while keeping CI green.

Key points:
- edit `specs/openapi/openapi.yaml`
- run OpenAPI lint locally (via `nix develop -c` and Redocly)
- understand and handle breaking-change reports from the `OpenAPI` workflow

