# DevSecOps Project Secrets Standard

**Status:** Stable (v1.0.0 - Updated 2025-11-17)
**Schema:** `schemas/devsecops/secrets/v1.0.0/secrets.schema.json`
**Config:** `config/devsecops/secrets/v1.0.0/defaults.yaml`
**Last Reviewed:** 2025-11-17

---

## Overview

The DevSecOps Project Secrets standard provides a canonical schema and workflow for managing environment-specific secrets across projects. It solves the "how do I safely store and load environment variables" problem with:

1. **Schema-validated YAML/JSON files** conforming to Crucible standards
2. **Dual-mode support:** Plaintext (development) and Encrypted (production)
3. **Project scoping:** Multiple projects in a single file with slug-based filtering
4. **Encryption metadata:** Track encryption method, keys, and timestamps
5. **Policy enforcement:** `allow_plain_secrets: false` for compliance workflows

**Primary Tool:** [`fulmen-secrets` (fulsecrets)](https://github.com/fulmenhq/fulmen-secrets) - CLI tool for encrypt/decrypt/exec workflows

**Use Cases:**

- Development: Local plaintext secrets for rapid iteration
- Staging/Production: Encrypted secrets with GPG/age/passphrase
- CI/CD: Subprocess wrapping for secure environment variable injection
- Compliance: Policy-enforced encryption ("FIPS mode")

---

## Schema Structure

### Core Fields

| Field            | Type   | Required      | Description                                             |
| ---------------- | ------ | ------------- | ------------------------------------------------------- |
| `schema_version` | string | Yes           | Schema version (`v1.0.0`)                               |
| `env_prefix`     | string | No            | Global env var prefix (e.g., `MYAPP_`) for all projects |
| `description`    | string | No            | Multi-line description of the secrets file              |
| `projects`       | array  | Conditional\* | Array of project configs (plaintext mode only)          |
| `encryption`     | object | Conditional\* | Encryption metadata (encrypted mode only)               |
| `ciphertext`     | string | Conditional\* | Encrypted payload (encrypted mode only)                 |
| `policies`       | object | No            | Policy enforcement configuration                        |

**Conditional Requirements:**

- **Plaintext mode:** `projects` required, `encryption` and `ciphertext` must NOT be present
- **Encrypted mode:** `encryption` and `ciphertext` required, `projects` must NOT be present

### Project Configuration

Each project in the `projects` array has:

| Field          | Type   | Required | Description                                                                                                                               |
| -------------- | ------ | -------- | ----------------------------------------------------------------------------------------------------------------------------------------- |
| `project_slug` | string | Yes      | Slugified identifier (lowercase, hyphens/underscores, alphanumeric). Examples: `myapp-prod`, `api_staging`, `worker-dev`, `my_service-v2` |
| `description`  | string | No       | Multi-line description of the project                                                                                                     |
| `credentials`  | object | Yes      | Map of credential names to credential objects (UPPER_SNAKE_CASE keys)                                                                     |
| `env_prefix`   | string | No       | Optional prefix for this project's env vars (overrides global). Example: `APP_`                                                           |

**Constraints:**

- `project_slug`: Must match `^[a-z0-9]([a-z0-9_-]*[a-z0-9])?$` (1-64 chars, start/end with alphanumeric)
- `credentials` keys: Must match `^[A-Z_][A-Z0-9_]*$` (UPPER_SNAKE_CASE env var names)

### Size Limits & DoS Protection

**Added in v1.0.0 (v0.2.19 hardening):** The schema enforces defensive size limits to prevent Denial-of-Service attacks and respect OS constraints:

| Field                                         | Limit             | Rationale                                               |
| --------------------------------------------- | ----------------- | ------------------------------------------------------- |
| **Credential values** (`value`)               | 65,536 chars      | 64KB max, UTF-8 encoded; generous for keys/certs/tokens |
| **External refs** (`ref`)                     | 2,048 chars       | Printable ASCII only; covers vault URIs, ARNs           |
| **Descriptions** (all levels)                 | 4,096 chars       | File, project, and credential descriptions              |
| **Credential key names** (env vars)           | 255 chars         | OS environment variable name limit (POSIX standard)     |
| **Projects per file** (`maxItems`)            | 256 projects      | Monorepo support without unbounded growth               |
| **Credentials per project** (`maxProperties`) | 1,024 credentials | Enterprise-scale projects; prevents accidental bloat    |

**Design Philosophy:**

- **Generous defaults:** Limits are 10-100x typical use cases to avoid legitimate workflows
- **DoS prevention:** Caps prevent malicious/accidental resource exhaustion
- **OS alignment:** Env var name limit matches POSIX/Linux/macOS standards
- **Audit-friendly:** Large descriptions support compliance documentation

**Override Strategy:**

If your use case exceeds these limits (rare):

1. Split large files into multiple project-scoped files (recommended)
2. Use external references (`ref`) for oversized values (e.g., large certificates → vault)
3. Contact Crucible maintainers if limits are blocking legitimate enterprise use

### Credential Object Structure

Each credential in the `credentials` map is an object with:

| Field         | Type   | Required | Description                                                                           |
| ------------- | ------ | -------- | ------------------------------------------------------------------------------------- |
| `type`        | enum   | Yes      | Credential type: `api_key`, `password`, or `token` (determines masking)               |
| `value`       | string | No\*     | Plaintext credential value (mutually exclusive with `ref`)                            |
| `ref`         | string | No\*     | External reference (e.g., `vault://secrets/db-url`) (mutually exclusive with `value`) |
| `description` | string | No       | Human-readable description of the credential's purpose                                |
| `metadata`    | object | No       | Lifecycle metadata (created, expires, purpose, tags, owner)                           |
| `rotation`    | object | No       | Rotation policy (interval, method)                                                    |

**\*Note:** Exactly one of `value` or `ref` must be provided (mutually exclusive).

#### Credential Types & Masking

| Type       | Use Case                        | Masking Behavior                 | Example                         |
| ---------- | ------------------------------- | -------------------------------- | ------------------------------- |
| `api_key`  | External service API keys       | Show prefix: `sk_live_...xyz`    | Stripe, GitHub, AWS keys        |
| `token`    | Auth tokens, JWTs               | Show suffix: `...1234`           | Bearer tokens, JWT secrets      |
| `password` | Passwords, DB URLs, passphrases | Full redaction: `***REDACTED***` | Database passwords, passphrases |

#### Metadata Object (Optional)

Lifecycle and ownership metadata for credential tracking, rotation planning, and compliance auditing.

**Core Fields:**

| Field            | Type              | Description                                                           |
| ---------------- | ----------------- | --------------------------------------------------------------------- |
| `created`        | string (ISO 8601) | Timestamp when credential was created                                 |
| `expires`        | string (ISO 8601) | Timestamp when credential expires                                     |
| `last_rotated`   | string (ISO 8601) | Timestamp when credential was last rotated                            |
| `next_rotation`  | string (ISO 8601) | Timestamp when credential should next be rotated                      |
| `rotation_count` | integer           | Number of times this credential has been rotated (minimum: 0)         |
| `purpose`        | string            | Purpose slug for categorization (e.g., `payment-processing`)          |
| `tags`           | array[string]     | Categorization tags for filtering (e.g., `["critical", "payment"]`)   |
| `owner`          | string            | Owner/team identifier (e.g., `platform-team`, `security@example.com`) |

**Service Integration Fields:**

| Field            | Type         | Description                                                       |
| ---------------- | ------------ | ----------------------------------------------------------------- |
| `service_name`   | string       | Name of external service (e.g., `Stripe`, `GitHub API`, `AWS S3`) |
| `service_url`    | string (URI) | Service URL (e.g., `https://api.stripe.com`)                      |
| `required_scope` | string       | OAuth/API scope (e.g., `repo:write`, `payments:read`)             |

**Compliance & Environment Fields:**

| Field             | Type          | Description                      | Allowed Values                                               |
| ----------------- | ------------- | -------------------------------- | ------------------------------------------------------------ |
| `compliance_tags` | array[string] | Compliance framework tags        | `pci-dss`, `soc2`, `hipaa`, etc. (max 20 items)              |
| `environment`     | string (enum) | Deployment environment           | `production`, `staging`, `development`, `test`, `qa`, `demo` |
| `tier`            | string (enum) | Service tier / criticality level | `critical`, `high`, `medium`, `low`                          |

#### Rotation Policy Object (Optional)

| Field      | Type   | Description                                    |
| ---------- | ------ | ---------------------------------------------- |
| `interval` | string | Rotation interval (e.g., `30d`, `90d`, `180d`) |
| `method`   | enum   | Rotation method: `auto` or `manual`            |

### Encryption Metadata

When file is encrypted, the `encryption` object contains:

| Field          | Type   | Required | Description                                                           |
| -------------- | ------ | -------- | --------------------------------------------------------------------- |
| `method`       | enum   | Yes      | Encryption method: `gpg`, `age`, or `passphrase`                      |
| `encrypted_at` | string | Yes      | ISO 8601 timestamp when encrypted                                     |
| `key_id`       | string | No       | GPG fingerprint or age public key (omit for passphrase)               |
| `cipher`       | string | No       | Cipher algorithm (e.g., `AES-256-GCM`) - recommended for auditability |

### Policy Configuration

| Field                 | Type    | Default | Description                                                                           |
| --------------------- | ------- | ------- | ------------------------------------------------------------------------------------- |
| `allow_plain_secrets` | boolean | `true`  | Whether to allow plaintext secrets. Set `false` for production to enforce encryption. |

---

## Usage Patterns

### Pattern 1: Development (Plaintext - Minimal)

**File:** `secrets.yaml`

```yaml
schema_version: v1.0.0
env_prefix: APP_

projects:
  - project_slug: myapp-dev
    credentials:
      DATABASE_URL:
        type: password
        value: postgres://localhost:5432/myapp_dev
      API_KEY:
        type: api_key
        value: dev_key_12345
      LOG_LEVEL:
        type: password
        value: debug

policies:
  allow_plain_secrets: true
```

**Usage:**

```bash
# Run app with dev secrets
fulsecrets exec -p myapp-dev -- npm run dev

# Or use specific file
fulsecrets exec -p myapp-dev -f secrets.yaml -- python app.py
```

**Security Note:** Plaintext files are OK for local development but should NEVER contain real production credentials.

### Pattern 1b: Development with Full Credential Metadata

**File:** `secrets-prod.yaml` (production example with rich metadata)

```yaml
schema_version: v1.0.0
description: |
  Production secrets for MyApp
  Contact: platform-team@example.com
env_prefix: PROD_
projects:
  - project_slug: backend_api
    description: Backend API services
    credentials:
      STRIPE_API_KEY:
        type: api_key
        value: sk_live_abc123def456
        description: Live Stripe API key for payment processing
        metadata:
          created: "2025-01-15T10:00:00Z"
          expires: "2026-01-15T10:00:00Z"
          purpose: payment-processing
          tags: [payment, critical, pci-scope]
          owner: payments-team
        rotation:
          interval: 90d
          method: auto
      DATABASE_PASSWORD:
        type: password
        value: super_secure_password
        description: Primary PostgreSQL database password
        metadata:
          expires: "2025-12-31T23:59:59Z"
          purpose: primary-database
          tags: [database, critical]
          owner: platform-team
        rotation:
          interval: 30d
          method: manual
      JWT_SECRET:
        type: token
        value: bearer_token_abc123
        description: JWT signing secret
```

**Benefits:**

- **Type-aware masking**: `api_key` shows prefix, `password` fully redacts
- **Expiry tracking**: Monitor credential lifecycle with `metadata.expires`
- **Rotation policy**: Document rotation requirements with `rotation.interval`
- **Ownership**: Track responsible teams with `metadata.owner`
- **Categorization**: Use `metadata.tags` for monitoring and alerting

### Pattern 2: Production (Encrypted - GPG)

**Workflow:**

```bash
# 1. Create plaintext file with real secrets (using credential objects)
cat > secrets-plain.yaml <<EOF
schema_version: v1.0.0
projects:
  - project_slug: myapp-prod
    credentials:
      DATABASE_URL:
        type: password
        value: postgres://prod.example.com/myapp
      API_KEY:
        type: api_key
        value: sk_live_real_key_xyz
        metadata:
          expires: "2026-01-01T00:00:00Z"
        rotation:
          interval: 90d
          method: auto
EOF

# 2. Encrypt with GPG
fulsecrets encrypt secrets-plain.yaml --gpg-key team@example.com -o secrets.yaml

# 3. Verify encryption
fulsecrets validate secrets.yaml
cat secrets.yaml  # Shows encryption metadata, not secrets

# 4. Clean up plaintext
rm secrets-plain.yaml

# 5. Commit encrypted file
git add secrets.yaml
git commit -m "Add production secrets (encrypted)"
```

**Encrypted File:** `secrets.yaml`

```yaml
schema_version: v1.0.0

encryption:
  method: gpg
  key_id: 7A8B9C0D1E2F3A4B
  encrypted_at: "2025-11-15T10:00:00Z"
  cipher: AES-256-GCM

# The 'projects' array with credential objects is encrypted inside ciphertext
ciphertext: |
  -----BEGIN PGP MESSAGE-----
  hQIMA3xQvBF2g8NSAQ//ZQWJRW8F...
  -----END PGP MESSAGE-----

policies:
  allow_plain_secrets: false
```

**Usage:**

```bash
# Run app with encrypted secrets (prompts for GPG passphrase)
fulsecrets exec -p myapp-prod -- npm start

# Decrypt to stdout for inspection (careful!)
fulsecrets decrypt secrets.yaml
```

### Pattern 3: Multi-Project with External References

**File:** `secrets.yaml` (plaintext for example)

```yaml
schema_version: v1.0.0

projects:
  - project_slug: api-staging
    credentials:
      DATABASE_URL:
        type: password
        ref: vault://secrets/staging/db-url # External reference
        description: Database URL from Vault
      JWT_SECRET:
        type: token
        value: staging_jwt_secret_abc

  - project_slug: worker-staging
    credentials:
      DATABASE_URL:
        type: password
        ref: vault://secrets/staging/db-url # Shared reference
      QUEUE_URL:
        type: password
        value: amqp://staging-queue.internal:5672

  - project_slug: frontend_staging
    env_prefix: VITE_ # Override global prefix
    credentials:
      API_URL:
        type: password
        value: https://api-staging.example.com
      ANALYTICS_KEY:
        type: api_key
        value: staging_analytics_key
```

**Usage:**

```bash
# Run specific project
fulsecrets exec -p api-staging -- node api-server.js
fulsecrets exec -p worker-staging -- python worker.py
fulsecrets exec -p frontend_staging -- npm run build

# Future: Multi-project merge (v0.2.0+)
fulsecrets exec -p api-staging -p worker-staging -- ./deploy.sh
# Merges both projects' secrets into single env (last wins on conflicts)
```

**External References:**

- Use `ref` field for credentials stored in external secret managers (Vault, AWS Secrets Manager, etc.)
- Tools implementing this schema should resolve `ref` URIs at runtime
- Reference formats: `vault://path/to/secret`, `aws-secrets://secret-name`, etc.

### Pattern 4: CI/CD Integration

**GitHub Actions Example:**

```yaml
# .github/workflows/deploy.yml
jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      # Install fulsecrets
      - name: Install fulsecrets
        run: |
          curl -L https://github.com/fulmenhq/fulmen-secrets/releases/download/v0.1.0/fulsecrets-linux-amd64 -o /usr/local/bin/fulsecrets
          chmod +x /usr/local/bin/fulsecrets

      # Deploy with secrets (passphrase from GitHub Secrets)
      - name: Deploy production
        env:
          FULSECRETS_PASSPHRASE: ${{ secrets.FULSECRETS_PASSPHRASE }}
        run: |
          # No prompt - reads passphrase from env var
          fulsecrets exec -p myapp-prod -f secrets.yaml --passphrase -- ./deploy.sh
```

**Security:** Passphrase stored in GitHub Secrets, never logged or exposed.

---

## Encryption Methods

### GPG (GNU Privacy Guard)

**When to use:** Teams with existing GPG infrastructure, multi-recipient encryption needed

**Setup:**

```bash
# Generate GPG key (if needed)
gpg --full-generate-key

# Encrypt secrets
fulsecrets encrypt secrets.yaml --gpg-key your-email@example.com

# Multi-recipient (future v0.2.0)
fulsecrets encrypt secrets.yaml --gpg-key alice@example.com --gpg-key bob@example.com
```

**Pros:**

- Industry standard, widely supported
- Multi-recipient encryption (team access)
- Key rotation workflows established

**Cons:**

- Complex key management (web of trust)
- Larger ciphertext
- Requires GPG tooling

### Age (Modern Alternative)

**When to use:** Greenfield projects, simpler key management desired

**Setup:**

```bash
# Generate age keypair
age-keygen -o ~/.config/fulsecrets/key.txt
# Public key: age1ql3z7hjy54pw3hyww5ayyfg7zqgvc7w3j2elw8zmrj2kg5sfn9aqmcac8p

# Encrypt secrets
fulsecrets encrypt secrets.yaml --age-recipient age1ql3z...

# Decrypt with private key
fulsecrets exec -p myapp-prod --age-identity ~/.config/fulsecrets/key.txt -- npm start
```

**Pros:**

- Modern, simple design
- Smaller ciphertext
- Easy key management

**Cons:**

- Newer standard (less ecosystem support)
- Requires age tooling

### Passphrase (Symmetric)

**When to use:** Solo developers, simplest setup, no key management

**Setup:**

```bash
# Encrypt with passphrase
fulsecrets encrypt secrets.yaml --passphrase
# Prompts: Enter passphrase: ********

# Use with passphrase
fulsecrets exec -p myapp-prod --passphrase -- npm start
# Prompts: Enter passphrase: ********
```

**Pros:**

- Simplest setup (no keys)
- Works everywhere (no external tools)

**Cons:**

- Passphrase must be shared (no per-user access)
- Password manager required for strong passphrase
- No key rotation (must re-encrypt with new passphrase)

---

## Security Recommendations

### 1. Never Commit Plaintext Production Secrets

```bash
# ❌ BAD - plaintext production secrets in git
git add secrets-prod.yaml  # Contains real credentials

# ✅ GOOD - encrypted secrets in git
fulsecrets encrypt secrets-prod.yaml --gpg-key team@example.com
git add secrets-prod.yaml  # Contains ciphertext only
```

**Enforcement:** Use `.gitignore` patterns:

```
*-plain.yaml
*-decrypted.yaml
secrets-*.txt
```

### 2. Use Policy Enforcement for Production

```yaml
# Production secrets file
policies:
  allow_plain_secrets: false  # Reject plaintext

# Validation fails if file is not encrypted
fulsecrets validate secrets-prod.yaml --strict
# Error: Policy violation: allow_plain_secrets is false but file is not encrypted
```

### 3. Subprocess Wrapping (Not Shell Export)

```bash
# ❌ DANGEROUS - secrets exposed in shell
eval $(fulsecrets load myapp-prod)  # Secrets in history, ps aux
export DATABASE_URL="postgres://..."
npm start

# ✅ SAFE - secrets isolated to subprocess
fulsecrets exec -p myapp-prod -- npm start
# Secrets only in npm process memory, never touch shell
```

### 4. Secure Temporary Files

When editing encrypted files:

```bash
# fulsecrets edit workflow:
# 1. Decrypts to /dev/shm (RAM disk) or temp dir with 0600 perms
# 2. Opens $EDITOR
# 3. Re-encrypts on save
# 4. Shreds temp file (overwrite + delete)

fulsecrets edit secrets.yaml
```

**Manual workflow:**

```bash
fulsecrets decrypt secrets.yaml -o /tmp/secrets-plain.yaml
chmod 600 /tmp/secrets-plain.yaml
vim /tmp/secrets-plain.yaml
fulsecrets encrypt /tmp/secrets-plain.yaml -o secrets.yaml
shred -u /tmp/secrets-plain.yaml  # Secure delete
```

### 5. CI/CD Best Practices

```yaml
# GitHub Actions
- name: Deploy
  env:
    # Store passphrase in GitHub Secrets
    FULSECRETS_PASSPHRASE: ${{ secrets.FULSECRETS_PASSPHRASE }}
  run: |
    # Never log/echo secrets
    set +x  # Disable command echoing
    fulsecrets exec -p myapp-prod --passphrase -- ./deploy.sh
```

**Never:**

- Echo secrets in CI logs
- Use `--dry-run` output in CI (contains plaintext secrets)
- Commit decrypted files in CI workspace

### 6. Key Rotation

```bash
# Scenario: Developer leaves team, rotate GPG key

# 1. Decrypt with old key
fulsecrets decrypt secrets.yaml --gpg-key old-dev@example.com -o plain.yaml

# 2. Re-encrypt with new key
fulsecrets encrypt plain.yaml --gpg-key new-team-key@example.com -o secrets.yaml

# 3. Verify and clean up
fulsecrets validate secrets.yaml
rm plain.yaml

# 4. Commit rotation
git add secrets.yaml
git commit -m "Rotate GPG key to new-team-key"
```

**Audit Trail:** `encryption.encrypted_at` updates automatically, providing rotation timestamp.

---

## CLI Tool Integration

### fulmen-secrets (fulsecrets)

**Installation:**

```bash
# macOS / Linux
curl -L https://github.com/fulmenhq/fulmen-secrets/releases/download/v0.1.0/fulsecrets-$(uname -s)-$(uname -m) -o /usr/local/bin/fulsecrets
chmod +x /usr/local/bin/fulsecrets

# Verify installation
fulsecrets --version
```

**Common Commands:**

```bash
# Initialize new secrets file
fulsecrets init secrets.yaml

# Validate schema compliance
fulsecrets validate secrets.yaml
fulsecrets validate secrets.yaml --strict  # Fail on warnings

# Encrypt file
fulsecrets encrypt secrets.yaml --gpg-key team@example.com
fulsecrets encrypt secrets.yaml --age-recipient age1ql3z...
fulsecrets encrypt secrets.yaml --passphrase

# Decrypt to stdout
fulsecrets decrypt secrets.yaml

# Execute command with secrets
fulsecrets exec -p myapp-prod -- npm start
fulsecrets exec -p api-prod -f secrets.yaml --passphrase -- python api.py

# List projects (doesn't show secret values)
fulsecrets list secrets.yaml

# Interactive edit (decrypt → edit → re-encrypt)
fulsecrets edit secrets.yaml
```

### L'Orage Central Integration

**Future integration (v0.3.0+):**

```yaml
# L'Orage Central can reference secrets files for credential loading
credentials:
  source: fulsecrets
  file: config/secrets.yaml
  project_slug: lorage-prod
```

### Etknow Microtool Integration

**Future integration:**

```bash
# Etknow can use secrets schema for credential management
etknow load-credentials --from-fulsecrets secrets.yaml -p etknow-prod
```

---

## Schema Evolution

### Current Version: v1.0.0 (Updated 2025-11-17)

**Included Features:**

- ✅ **Structured credential objects** with types (`api_key`, `password`, `token`)
- ✅ **Smart masking** based on credential type
- ✅ **Lifecycle metadata** (`created`, `expires`, `purpose`, `tags`, `owner`)
- ✅ **Rotation policies** (`interval`, `method: auto|manual`)
- ✅ **External references** (`ref` field for vault integration)
- ✅ **Project scoping** via `project_slug` (supports hyphens and underscores)
- ✅ **Global and project-level environment prefixes** (`env_prefix`)
- ✅ **Multi-line descriptions** (top-level and project-level)
- ✅ **Encryption metadata** (method, key_id, encrypted_at, cipher)
- ✅ **Dual-mode schema** (plaintext OR encrypted with oneOf enforcement)
- ✅ **Policy enforcement** (`allow_plain_secrets: false` for FIPS mode)
- ✅ **UTF-8 support** for descriptions and metadata

**Schema Properties:**

- Credential objects replace flat string values for rich metadata
- `value` XOR `ref` enforcement via oneOf constraint
- Project slugs: `^[a-z0-9]([a-z0-9_-]*[a-z0-9])?$` (allows underscores)
- Environment variable names: `^[A-Z_][A-Z0-9_]*$` (UPPER_SNAKE_CASE)
- Rotation intervals: `^[0-9]+[dwmy]$` (days/weeks/months/years)

### Future: v1.1.0+ (Planned)

**Potential Additions:**

- **Reference resolution**: Native vault:// URI handler in fulsecrets
- **Automated rotation triggers**: Integration with secret rotation APIs
- **Secret templates**: Interpolation support (e.g., `${DATABASE_URL}/api`)
- **Telemetry integration**: Decrypt events to L'Orage Central
- **Multi-environment fields**: Explicit environment tagging
- **Credential hierarchy**: Inherit credentials from parent projects

**Note:** Most enterprise features are already available in v1.0.0. Future versions will focus on automation and integration rather than data model changes.

**Migration:** Schema is backward-incompatible with pre-v1.0.0 flat format. Tools should detect old format and prompt migration.

---

## Compliance & FIPS Mode

### Production Enforcement

```yaml
# Production secrets MUST be encrypted
schema_version: v1.0.0

policies:
  allow_plain_secrets: false # Reject plaintext

encryption:
  method: gpg
  key_id: FIPS_VALIDATED_KEY_ID
  cipher: AES-256-GCM
ciphertext: "..."
```

**Validation:**

```bash
# Strict validation fails on policy violations
fulsecrets validate secrets-prod.yaml --strict
# ✅ Pass: File is encrypted, policy satisfied

# If someone commits plaintext with policy=false
fulsecrets validate secrets-bad.yaml --strict
# ❌ Error: Policy violation: allow_plain_secrets is false but file is not encrypted
```

### Pre-Commit Hook Example

```bash
#!/bin/bash
# .git/hooks/pre-commit

# Validate all secrets files
for file in $(git diff --cached --name-only | grep 'secrets.*\.yaml$'); do
  # Validate schema compliance
  if ! fulsecrets validate "$file" --strict; then
    echo "❌ Secrets validation failed: $file"
    exit 1
  fi

  # Ensure production files are encrypted
  if [[ "$file" =~ prod ]]; then
    if ! grep -q "^encryption:" "$file"; then
      echo "❌ Production secrets must be encrypted: $file"
      exit 1
    fi
  fi
done

echo "✅ Secrets validation passed"
```

---

## Examples

### Example 1: Solo Developer (Passphrase)

```yaml
schema_version: v1.0.0

encryption:
  method: passphrase
  encrypted_at: "2025-11-15T16:00:00Z"
  cipher: AES-256-GCM

ciphertext: "U2FsdGVkX1+abcd..."
```

**Workflow:**

```bash
# Day 1: Create + encrypt
fulsecrets init secrets.yaml
vim secrets.yaml  # Add secrets
fulsecrets encrypt secrets.yaml --passphrase
git add secrets.yaml && git commit -m "Add secrets"

# Day 2: Use secrets
fulsecrets exec -p myapp-prod --passphrase -- npm start
# Prompts once for passphrase, runs app
```

### Example 2: Team with GPG

```yaml
schema_version: v1.0.0

encryption:
  method: gpg
  key_id: 7A8B9C0D1E2F3A4B # Team GPG key
  encrypted_at: "2025-11-15T10:00:00Z"
  cipher: AES-256-GCM

ciphertext: "-----BEGIN PGP MESSAGE-----\n..."

policies:
  allow_plain_secrets: false
```

**Team Workflow:**

```bash
# Alice (admin): Create encrypted file
fulsecrets encrypt secrets.yaml --gpg-key team@example.com
git add secrets.yaml && git commit -m "Add prod secrets"

# Bob (dev): Clone and use
git clone repo && cd repo
fulsecrets exec -p myapp-prod -- npm start
# Uses Bob's GPG key (if added to team key), prompts for passphrase

# Charlie (new dev): Key rotation
fulsecrets decrypt secrets.yaml --gpg-key team@example.com -o plain.yaml
fulsecrets encrypt plain.yaml --gpg-key new-team@example.com -o secrets.yaml
rm plain.yaml
git add secrets.yaml && git commit -m "Rotate team GPG key"
```

### Example 3: Monorepo with Multiple Services

```yaml
schema_version: v1.0.0

projects:
  - project_slug: api-prod
    credentials:
      DATABASE_URL:
        type: password
        value: postgres://prod-db/api
      REDIS_URL:
        type: password
        value: redis://prod-cache:6379
      JWT_SECRET:
        type: token
        value: prod_jwt_abc

  - project_slug: worker-prod
    credentials:
      DATABASE_URL:
        type: password
        value: postgres://prod-db/api # Shared credential
      QUEUE_URL:
        type: password
        value: amqp://prod-queue:5672
      WORKER_TOKEN:
        type: token
        value: prod_worker_xyz

  - project_slug: frontend-prod
    env_prefix: VITE_
    credentials:
      API_URL:
        type: password
        value: https://api.example.com
```

**Deployment:**

```bash
# Deploy API
fulsecrets exec -p api-prod -- node api-server.js

# Deploy worker
fulsecrets exec -p worker-prod -- python worker.py

# Build frontend (injects VITE_API_URL)
fulsecrets exec -p frontend-prod -- npm run build
```

---

## Troubleshooting

### Issue: "Policy violation: allow_plain_secrets is false"

**Cause:** Attempting to use plaintext file with `allow_plain_secrets: false`

**Solution:**

```bash
# Encrypt the file
fulsecrets encrypt secrets.yaml --gpg-key your-key@example.com
```

### Issue: "GPG decryption failed: No secret key"

**Cause:** GPG key used for encryption is not in your keyring

**Solution:**

```bash
# Import the correct key
gpg --import team-key.asc

# Or decrypt with specific key
fulsecrets decrypt secrets.yaml --gpg-key other-dev@example.com
```

### Issue: "Schema validation failed: project_slug must be lowercase"

**Cause:** Project slug contains uppercase letters or invalid characters

**Solution:**

```yaml
# ❌ Invalid
projects:
  - project_slug: MyApp-Prod  # Uppercase

# ✅ Valid
projects:
  - project_slug: myapp-prod  # Lowercase, hyphens only
```

### Issue: "Encrypted file but no encryption method in CI"

**Cause:** CI doesn't have passphrase/key to decrypt

**Solution:**

```yaml
# Add to GitHub Secrets / GitLab CI Variables
FULSECRETS_PASSPHRASE: your-passphrase

# Use in workflow
env:
  FULSECRETS_PASSPHRASE: ${{ secrets.FULSECRETS_PASSPHRASE }}
run: fulsecrets exec -p myapp-prod --passphrase -- ./deploy.sh
```

---

## References

- **Schema:** [`schemas/devsecops/secrets/v1.0.0/secrets.schema.json`](../../../schemas/devsecops/secrets/v1.0.0/secrets.schema.json)
- **Config Examples:** [`config/devsecops/secrets/v1.0.0/defaults.yaml`](../../../config/devsecops/secrets/v1.0.0/defaults.yaml)
- **fulsecrets Tool:** [github.com/fulmenhq/fulmen-secrets](https://github.com/fulmenhq/fulmen-secrets)
- **Implementation Notes:** [`.plans/memos/fulmenhq/20251115-notes-on-fulsecrets.md`](../.plans/memos/fulmenhq/20251115-notes-on-fulsecrets.md) (Crucible repo)

---

**Maintained by:** Schema Cartographer
**Issues:** [github.com/fulmenhq/crucible/issues](https://github.com/fulmenhq/crucible/issues)
