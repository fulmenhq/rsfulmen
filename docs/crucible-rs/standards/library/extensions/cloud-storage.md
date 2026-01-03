---
title: "Cloud Storage Extension"
description: "Optional helper module for interacting with common object storage providers"
author: "Schema Cartographer"
date: "2025-10-09"
last_updated: "2025-10-09"
status: "draft"
tags: ["standards", "library", "extensions", "storage", "2025.10.2"]
---

# Cloud Storage Extension

## Scope

Provide convenience wrappers for interacting with cloud object storage (S3, GCS, Azure Blob) tailored for
Fulmen tooling use cases (artifact uploads, schema archives, pipeline outputs).

## Capabilities

- Credential resolution (environment variables, shared config files, workload identities).
- Simplified helpers for common operations: `UploadObject`, `DownloadObject`, `ListPrefix`, `DeleteObject`.
- Configurable retry/backoff strategies aligned with Fulmen reliability standards.
- Optional client-side encryption (KMS integration TBD).

## Implementation Notes

- **Go**: wrappers around AWS SDK v2, Google Cloud Storage client, Azure SDK. Provide interface for testability.
- **Python**: rely on `boto3`, `google-cloud-storage`, `azure-storage-blob` with thin abstraction layer.
- **TypeScript**: target Node runtime; use official SDKs (`@aws-sdk/client-s3`, `@google-cloud/storage`,
  `@azure/storage-blob`).

## Configuration

Expose unified configuration struct/object capturing provider, region, bucket, prefix, and credentials source.
Validate via JSON Schema before constructing clients.

## Testing

- Mock-based unit tests for credential resolution and error handling.
- Integration tests using local emulators where available (e.g., `localstack`, `fake-gcs-server`).

## Status

- Optional; implement when consumer demand justifies footprint. Track adoption in module manifest overrides.
