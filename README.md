# Secure LightClaw

Secure LightClaw is a lightweight, secure AI assistant framework written in Rust.

## Features
- Modular core with pluggable LLM provider abstraction.
- WASM plugin sandbox with explicit manifest permissions.
- Channels: CLI, HTTP API, Telegram polling adapter.
- Encrypted persistent memory using SQLite + AEAD envelope encryption.
- Cron-like scheduler for plugin automation.

## Build
```bash
cargo build
```

## Run
1. Initialize config:
```bash
cargo run -- init
```
2. Edit `lightclaw.toml` with API keys.
3. Chat from CLI:
```bash
cargo run -- chat "hello"
```
4. Start API:
```bash
cargo run -- start
```

## Add Plugins
1. Build a WASM plugin exporting `run() -> i32`.
2. Create a plugin manifest JSON with `permissions`.
3. Reference that manifest in `config/scheduler.toml`.
4. Execute scheduler once:
```bash
cargo run -- run-scheduler-once
```

## Telegram Adapter
Set these values in `lightclaw.toml`:
- `telegram_bot_token`
- `telegram_chat_id`

Then run:
```bash
cargo run -- telegram-poll-once
```

## LLM Provider Setup
Set provider fields in `lightclaw.toml`:
- Primary: `llm_primary_provider`, `llm_primary_api_key`, `llm_primary_model`, `llm_primary_api_base`
- Fallback: `llm_fallback_provider`, `llm_fallback_api_key`, `llm_fallback_model`, `llm_fallback_api_base`

The runtime now tries the primary provider first and automatically falls back when the primary call fails.

Quick test:
```bash
cargo run -- chat "Summarize secure plugin isolation in one sentence"
```

## Tests
```bash
cargo test
```
