# Security Model

Secure LightClaw is secure-by-default.

## Sandboxing
- Plugins run as WebAssembly modules through Wasmtime.
- The framework only calls the plugin's exported `run` function.
- Plugins are gated by an explicit manifest permission set.

## Permission policy
- Permission checks are deny-by-default.
- Each plugin invocation requires a named capability (for example `scheduler.run`).
- Missing permissions fail closed.

## Memory protection
- Context memory is stored in SQLite.
- Values are encrypted before storage using ChaCha20-Poly1305 envelope encryption.
- Encryption keys are derived from the configured secret; rotate this secret in production.

## Operational guidance
- Do not commit real API keys.
- Use isolated deployment secrets per environment.
- Review plugin manifests before enabling scheduled execution.
