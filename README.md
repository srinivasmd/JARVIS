# JARVIS

A lightweight personal AI assistant project focused on secure-by-default architecture.

## Secure LightClaw v2 Status

Implementation coverage versus `docs/secure-lightclaw-plan.md` is currently **100% (25/25 roadmap items)** with executable scaffolds across all phases.

- Phase 1 (Core MVP): ✅
- Phase 2 (Security Hardening): ✅
- Phase 3 (Multi-Platform Release): ✅
- Phase 4 (Plugin Marketplace): ✅
- Phase 5 (Performance Optimization): ✅

See detailed mapping in [docs/implementation-status.md](docs/implementation-status.md).

## Runtime Capabilities

- Modular agent core with prompt loop and channel message handling.
- Policy-based permissions and deny-by-default tool sandbox.
- Provider abstraction with router/fallback behavior.
- Plugin manifest vetting, source-aware registration, and registry publish/verify service.
- In-memory context with encrypted file export/import.
- Audit log events for sensitive operations.
- Scheduler heartbeat for recurring tasks.
- CLI mode, web UI mode, benchmark mode, and plugin publish mode.

## Run

```bash
cargo run -- chat "hello jarvis"
```

## Serve Web UI (single request)

```bash
cargo run -- serve-web
```

## Publish Example Plugin

```bash
cargo run -- publish-plugin examples/plugin-manifest.json
```

## Benchmark Startup

```bash
cargo run -- bench
```

## Test

```bash
cargo test
```

## Build Release

```bash
./scripts/build-release.sh
```

## Development Plan

See the full Secure LightClaw roadmap and architecture notes in:

- [docs/secure-lightclaw-plan.md](docs/secure-lightclaw-plan.md)
