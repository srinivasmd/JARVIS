# Contributing

## Development setup
1. Install Rust stable toolchain.
2. Clone repository.
3. Run format and tests:
```bash
cargo fmt
cargo test
```

## Architecture notes
- `src/core.rs` contains orchestration, memory, and scheduler wiring.
- `src/sandbox.rs` handles WASM execution and permission checks.
- `src/provider.rs` holds provider abstraction and OpenAI implementation.
- `src/api.rs` serves HTTP endpoints.
- `src/adapters/telegram.rs` manages Telegram integration.

## Pull requests
- Add tests for behavior changes.
- Keep security model deny-by-default.
- Document new plugin permissions.
