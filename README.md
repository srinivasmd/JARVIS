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
- Telegram polling (single cycle):
  ```bash
  cargo run -- telegram-poll-once
  ```
- Telegram webhook listener (single request lifecycle):
  ```bash
  cargo run -- telegram-webhook-once
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
| `JARVIS_LLM_PROVIDER` | `echo-local` | Provider selector (`echo-local`, `openai`, `openai-compatible`, `anthropic`). |
| `JARVIS_LLM_MODEL` | `mock-1` | Model identifier passed to provider adapters. |
| `JARVIS_LLM_API_BASE` | unset | Base URL for provider API (for example `http://127.0.0.1:11434/v1`). |
| `JARVIS_LLM_API_KEY` | unset | API key used in provider auth headers. |

Example (OpenAI-compatible):

```bash
export JARVIS_LLM_PROVIDER=openai-compatible
export JARVIS_LLM_MODEL=gpt-4.1-mini
export JARVIS_LLM_API_BASE=http://127.0.0.1:4000/v1
export JARVIS_LLM_API_KEY=local-dev-key
cargo run -- chat "summarize this"
```

Example (Anthropic-style endpoint):

```bash
export JARVIS_LLM_PROVIDER=anthropic
export JARVIS_LLM_MODEL=claude-3-5-sonnet-20240620
export JARVIS_LLM_API_BASE=http://127.0.0.1:5000/v1
export JARVIS_LLM_API_KEY=local-dev-key
cargo run -- chat "explain this"
```

> Note: current adapters use stdlib HTTP transport and support `http://` endpoints (local proxy/dev gateways) out of the box.

### 2) Telegram Settings (Polling + Webhook)

| Variable | Default | Description |
| --- | --- | --- |
| `JARVIS_TELEGRAM_ENABLED` | `false` | Enables Telegram startup validation and command use. |
| `JARVIS_TELEGRAM_BOT_TOKEN` | unset | Telegram bot token (required when enabled). |
| `JARVIS_TELEGRAM_CHAT_ID` | unset | Default outbound chat ID (required when enabled). |
| `JARVIS_TELEGRAM_API_BASE` | unset | Telegram API base URL (defaults internally to `http://127.0.0.1:8081` for local adapter testing). |
| `JARVIS_TELEGRAM_WEBHOOK_URL` | unset | Bind address for `telegram-webhook-once` (example: `127.0.0.1:9090`). |
| `JARVIS_TELEGRAM_POLL_INTERVAL_MS` | `1000` | Poll interval config value for polling workflows. |

Polling example:

```bash
export JARVIS_TELEGRAM_ENABLED=true
export JARVIS_TELEGRAM_BOT_TOKEN=123456:ABCDEF
export JARVIS_TELEGRAM_CHAT_ID=987654321
export JARVIS_TELEGRAM_API_BASE=http://127.0.0.1:8081
cargo run -- telegram-poll-once
```

Webhook example:

```bash
export JARVIS_TELEGRAM_ENABLED=true
export JARVIS_TELEGRAM_BOT_TOKEN=123456:ABCDEF
export JARVIS_TELEGRAM_CHAT_ID=987654321
export JARVIS_TELEGRAM_WEBHOOK_URL=127.0.0.1:9090
cargo run -- telegram-webhook-once
```

### 3) Runtime / App Settings

| Variable | Default | Description |
| --- | --- | --- |
| `JARVIS_WEB_BIND` | `127.0.0.1:7878` | Bind address for `serve-web`. |
| `JARVIS_DEFAULT_MODE` | `chat` | Command used when running `cargo run --` without an explicit command. |
| `JARVIS_REGISTRY_PATH` | `registry` | Local directory for published plugin artifacts/signatures. |
| `JARVIS_REGISTRY_SECRET` | `local-dev-secret` | Secret used for deterministic registry signing/verification. |
| `JARVIS_BENCH_ITERATIONS` | `100` | Iteration count used by benchmark mode. |

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
