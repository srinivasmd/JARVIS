# JARVIS

A lightweight personal AI assistant project focused on secure-by-default architecture.

## Secure LightClaw v2 Status

Implementation coverage versus `docs/secure-lightclaw-plan.md` is currently **100% (25/25 roadmap items)** with executable scaffolds across all phases.

## Quick Start

```bash
cargo run -- chat "hello jarvis"
```

## Commands

- Chat mode:
  ```bash
  cargo run -- chat "hello jarvis"
  ```
- Web UI mode (single request lifecycle):
  ```bash
  cargo run -- serve-web
  ```
- Publish example plugin to local registry path:
  ```bash
  cargo run -- publish-plugin examples/plugin-manifest.json
  ```
- Startup benchmark:
  ```bash
  cargo run -- bench
  ```
- Show effective config (redacted):
  ```bash
  cargo run -- show-config
  ```

## Configuration

Configuration is environment-variable driven.

### 1) LLM Provider + Model

| Variable | Default | Description |
| --- | --- | --- |
| `JARVIS_LLM_PROVIDER` | `echo-local` | Logical provider name used by runtime provider factory. |
| `JARVIS_LLM_MODEL` | `mock-1` | Model identifier attached to responses and logs. |
| `JARVIS_LLM_API_BASE` | unset | Optional API base URL for HTTP-based provider integrations. |
| `JARVIS_LLM_API_KEY` | unset | Optional API key for external providers. |

Example:

```bash
export JARVIS_LLM_PROVIDER=openai
export JARVIS_LLM_MODEL=gpt-4.1-mini
export JARVIS_LLM_API_BASE=https://api.openai.com/v1
export JARVIS_LLM_API_KEY=sk-...
cargo run -- chat "summarize this"
```

> Current implementation is a safe configurable echo-provider scaffold; provider/model values are still fully configurable and ready for concrete API integrations.

### 2) Telegram Settings

| Variable | Default | Description |
| --- | --- | --- |
| `JARVIS_TELEGRAM_ENABLED` | `false` | Enables Telegram config validation flow. |
| `JARVIS_TELEGRAM_BOT_TOKEN` | unset | Telegram bot token (required when enabled). |
| `JARVIS_TELEGRAM_CHAT_ID` | unset | Target chat/user id (required when enabled). |
| `JARVIS_TELEGRAM_WEBHOOK_URL` | unset | Optional webhook endpoint for telegram delivery. |

Example:

```bash
export JARVIS_TELEGRAM_ENABLED=true
export JARVIS_TELEGRAM_BOT_TOKEN=123456:ABCDEF
export JARVIS_TELEGRAM_CHAT_ID=987654321
export JARVIS_TELEGRAM_WEBHOOK_URL=https://example.com/jarvis/telegram
cargo run -- show-config
```

If Telegram is enabled but token/chat-id is missing, startup fails fast with a config error.

### 3) Runtime / App Settings

| Variable | Default | Description |
| --- | --- | --- |
| `JARVIS_WEB_BIND` | `127.0.0.1:7878` | Bind address for `serve-web`. |
| `JARVIS_DEFAULT_MODE` | `chat` | Command used when you run `cargo run --` with no explicit command. |
| `JARVIS_REGISTRY_PATH` | `registry` | Local directory for published plugin artifacts/signatures. |
| `JARVIS_REGISTRY_SECRET` | `local-dev-secret` | Secret used for deterministic registry signing/verification. |
| `JARVIS_BENCH_ITERATIONS` | `100` | Iteration count used by benchmark mode. |

Example:

```bash
export JARVIS_WEB_BIND=0.0.0.0:8080
export JARVIS_DEFAULT_MODE=chat
export JARVIS_REGISTRY_PATH=/tmp/jarvis-registry
export JARVIS_REGISTRY_SECRET=super-secret
export JARVIS_BENCH_ITERATIONS=500
cargo run -- bench
```

## Helper Scripts

```bash
./scripts/start-local-api.sh
./scripts/publish-example-plugin.sh
./scripts/run-bench.sh
./scripts/build-release.sh
```

## Test

```bash
cargo test
```

## Development Plan

- [docs/secure-lightclaw-plan.md](docs/secure-lightclaw-plan.md)
- [docs/implementation-status.md](docs/implementation-status.md)
- [docs/feature-ideas-v3.md](docs/feature-ideas-v3.md)
