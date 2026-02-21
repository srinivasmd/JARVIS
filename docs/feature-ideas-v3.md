# Secure LightClaw v3 Feature Ideas

This backlog proposes practical next features after v2, prioritized to maximize security, usability, and real-world deployment readiness.

## Priority 0 (High Impact / Near-Term)

### 1) Real Provider Integrations (OpenAI / Anthropic / OpenRouter / Local vLLM)
- Implement provider adapters behind the existing provider abstraction.
- Add per-provider auth config and request timeout/retry settings.
- Keep safe fallback chain behavior and provider health checks.

**Why:** Current provider path is scaffolded and ready; this unlocks immediate production use.

### 2) Telegram Transport Adapter (Polling + Webhook)
- Build `TelegramAdapter` for inbound messages and outbound replies.
- Add optional long polling mode for local development.
- Add webhook mode for cloud deployments.
- Reuse existing Telegram env config already validated in startup.

**Why:** Telegram is already modeled in config but not connected to message transport.

### 3) Durable Memory Backend
- Replace toy encrypted line format with structured SQLite persistence.
- Add memory namespaces (user/session/system), TTL/retention, and basic search.
- Add migration/versioning support.

**Why:** Reliable long-term memory is central to assistant quality.

### 4) Stronger Plugin Security
- Add manifest schema versioning and compatibility checks.
- Add plugin package checksum/signature verification at load time.
- Add explicit policy approval gates before first plugin activation.

**Why:** Complements the existing source-vetting controls with stronger runtime trust.

## Priority 1 (Product Maturity)

### 5) Conversation Management
- Add session IDs, multi-turn context windows, and context truncation strategy.
- Add token budgeting and prompt-template controls.

### 6) Web UI Upgrade
- Convert single-request server into persistent interactive chat UI.
- Add conversation list, settings pane, and audit trace view.

### 7) Observability
- Add structured logs (JSON), correlation IDs, and log levels.
- Add metrics endpoint (latency, error rates, provider success/fallback counts).

### 8) Policy Profiles
- Add predefined policy profiles: `safe`, `balanced`, `power-user`.
- Add profile override at runtime via env/CLI.

## Priority 2 (Scale + Ecosystem)

### 9) Plugin Registry API Service
- Expose registry publish/fetch over HTTP with auth.
- Add signatures, provenance metadata, and trust policies.
- Add moderation/vetting status workflow.

### 10) Multi-tenant Runtime
- Add tenant-scoped config, memory, and plugin isolation.
- Add per-tenant quotas and rate limiting.

### 11) Workflow/Automation Engine
- Add declarative task workflows (trigger → actions → policy checks).
- Integrate scheduler, tools, and channel notifications.

### 12) Deployment Hardening
- Add container image profiles (dev/prod/minimal).
- Add Helm chart and Kubernetes manifests.
- Add CI security scans and SBOM generation.

## Suggested Execution Sequence

1. Provider integrations + Telegram adapter
2. Durable memory backend + improved plugin verification
3. Web UI and observability improvements
4. Registry API + multi-tenant controls
5. Deployment hardening and ecosystem tooling

## Definition of Done for v3

- At least 2 real providers fully integrated with retries and fallback.
- Telegram works in both polling and webhook modes.
- Memory persistence uses SQLite with retention and namespace support.
- Plugin verification checks signature + checksum before activation.
- Web UI supports multi-turn chat sessions.
- Metrics/logging are production-usable.
